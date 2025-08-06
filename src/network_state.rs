use std::{cell::RefCell, collections::HashMap, future::Future, pin::Pin, sync::Arc, task::Poll};

use futures::FutureExt;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::init_tunnel::{InitTunnelResult, init_tunnel};

thread_local! {
    // This is the cache for all the NetworkState present.
    // It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    static NETWORK_STATE: RefCell<HashMap<String, Arc<NetworkState>>> = RefCell::new(HashMap::new());

    // These events are lazily evaluated (async polling) when `fetch` is called, the evaluation is a
    // blocking operation that waits for the tunnel to be initialized.
    static INIT_EVENT_ITEMS: RefCell<HashMap<String, InitEventItem>>= RefCell::new(HashMap::new());

    // This is a flag to indicate if the dev mode is enabled. It is used to enable or disable the dev mode features like logging.
    pub(crate) static DEV_FLAG: RefCell<bool> = const { RefCell::new(false) };
}

// This event item is used to store the events that are waiting to be processed.
// Design:
// 1. An initialization call is pushed to the event items. This allows it to be polled later to make sure the tunnel initialization happened or errored out.
// 2. Any calls to the `fetch` API will first check if the tunnel is initialized.
//    - If it is initialized in the NETWORK_STATE, the call is made.
//    - If it is not initialized, the initialization call is polled in the INIT_EVENT_ITEMS to check if it is done.
//    - If the initialization call is done, the fetch call is made.
//    - If the initialization call is not done, the fetch call waits and retries to poll until the initialization call is done.
// 3. If the initialization call failed in INIT_EVENT_ITEMS, the fetch call will return an error.
struct InitEventItem {
    init_event: Pin<Box<dyn Future<Output = Result<InitTunnelResult, JsValue>> + 'static>>,
    forward_proxy_url: String,
    // This is marker to keep track of how many iterations of the network_state have been created.
    // Say it's 4, it means this is the 4th iteration of the network_state
    version: Version,
}

// This is an alias value to track the version of the tunnel. It is incremented every time a new tunnel is initialized.
pub(crate) type Version = u16;

#[derive(Debug)]
pub(crate) struct NetworkState {
    pub http_client: reqwest::Client,
    pub init_tunnel_result: InitTunnelResult,
    pub forward_proxy_url: String,
    pub base_url: String,
    // This is marker to keep track of how many iterations of the network_state have been created.
    // Say it's 4, it means this is the 4th iteration of the network_state
    pub version: Version,
}

/// This represents the service provider endpoint that the l8 network is connecting to.
#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ServiceProvider {
    url: String,
    #[allow(dead_code)]
    options: Option<js_sys::Object>, // for now, options is just any object including empty
}

#[wasm_bindgen]
impl ServiceProvider {
    /// Allows the creation of a service provider with optional configurable options.
    /// The options have not yet been implemented.
    pub fn new(url: String, options: Option<js_sys::Object>) -> Self {
        ServiceProvider { url, options }
    }
}

/// This function initializes the encrypted tunnel for the given service providers.
/// It checks if the provider already has an initialized tunnel, if not it initializes a new tunnel
/// and stores the result.
#[wasm_bindgen(js_name = "initEncryptedTunnel")]
pub fn init_encrypted_tunnel(
    forward_proxy_url: String,
    service_providers: Vec<ServiceProvider>,
    dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    if let Some(val) = dev_flag {
        if val {
            DEV_FLAG.with_borrow_mut(|flag| *flag = true);
            console::log_1(&"Dev mode enabled".into());
        }
    }

    for service_provider in service_providers {
        let base_url = base_url(&service_provider.url)?;
        schedule_init_event(
            &base_url,
            1,
            forward_proxy_url.clone(),
            dev_flag.unwrap_or(false),
        )?;
    }

    Ok(())
}

pub(crate) fn schedule_init_event(
    base_url: &str,
    expected_next_version: Version,
    forward_proxy_url: String,
    dev_flag: bool,
) -> Result<(), JsValue> {
    // if there's already a version in the event items or connected state that is higher than the one we want to create we
    // short-circuit
    let current_version = NetworkReadyState::ready_state(base_url, false, dev_flag)?.version();
    if current_version >= expected_next_version {
        return Ok(());
    }

    let backend_url = format!("{}/init-tunnel?backend_url={}", forward_proxy_url, base_url);
    let init_event = InitEventItem {
        forward_proxy_url,
        version: current_version + 1,
        init_event: Box::pin(init_tunnel(backend_url, dev_flag)),
    };

    INIT_EVENT_ITEMS.with_borrow_mut(|init_event_items| {
        init_event_items.insert(base_url.to_string(), init_event)
    });
    Ok(())
}

// This function extracts the base URL from a given URL string. Example input: "https://example.com/path?query=1#fragment" will return "https://example.com".
pub(crate) fn base_url(url: &str) -> Result<String, JsValue> {
    let url =
        url::Url::parse(url).map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    // get without query or path fragments
    let mut base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
    if let Some(port) = url.port() {
        base_url = format!("{}:{}", base_url, port);
    }

    Ok(base_url)
}

#[derive(Debug, Clone)]
pub(crate) enum NetworkReadyState {
    CONNECTING(Version),
    OPEN(Arc<NetworkState>),
    CLOSED,
}

impl NetworkReadyState {
    /// This function checks the current state of the network for the given base URL. It will only return the state of the latest version
    /// if there are multiple versions of the network state.
    ///
    /// If `is_lazy` is true and a version was found in connecting state, it will not poll the initialization event but instead report it as
    /// [`NetworkReadyState::CONNECTING`]
    pub(crate) fn ready_state(
        base_url: &str,
        poll_init_event: bool,
        dev_flag: bool,
    ) -> Result<NetworkReadyState, JsValue> {
        let mut versions = Vec::new();
        if let Some(state) =
            NETWORK_STATE.with_borrow(|cache| cache.get(base_url).map(|val| Arc::clone(val)))
        {
            versions.push(NetworkReadyState::OPEN(state));
        }

        // let's short-circuit since we don't need to evaluate the init event if we find it
        if !poll_init_event {
            let connecting_state = INIT_EVENT_ITEMS
                .with_borrow(|init_event_items| init_event_items.get(base_url).map(|v| v.version));

            if let Some(version) = connecting_state {
                versions.push(NetworkReadyState::CONNECTING(version));
            }

            versions.sort_by_key(|a| a.version());
            let latest = versions
                .last()
                .cloned()
                .unwrap_or(NetworkReadyState::CLOSED);

            if dev_flag {
                console::log_1(
                    &format!("Latest network state for URL {}: {:?}", base_url, latest).into(),
                );
            }

            return Ok(latest);
        }

        // check if there's a version in the INIT_EVENT_QUEUE
        let init_event_state: Option<Result<NetworkReadyState, JsValue>> = INIT_EVENT_ITEMS
            .with_borrow_mut(
                |init_event_items| match init_event_items.get_mut(base_url) {
                    Some(fut) => pool_op(base_url, fut, dev_flag),
                    None => None,
                },
            );

        match init_event_state {
            Some(val) => {
                let state = match val {
                    Ok(val) => val,
                    Err(err) => {
                        // we failed to initialize, we are removing the entry from the init event items
                        INIT_EVENT_ITEMS.with_borrow_mut(|init_event_items| {
                            init_event_items.remove(base_url);
                        });

                        return Err(err);
                    }
                };

                if let NetworkReadyState::OPEN(..) = state {
                    INIT_EVENT_ITEMS.with_borrow_mut(|init_event_items| {
                        init_event_items.remove(base_url);
                    });
                }

                versions.push(state);
            }
            None => {
                // If the base URL is not in the cache or init event items, it means it was never initialized.
                if dev_flag {
                    console::log_1(
                        &format!(
                            "No init event found for URL: {}. Assuming it is already open.",
                            base_url
                        )
                        .into(),
                    );
                }
            }
        }

        versions.sort_by_key(|a| a.version());
        let latest = versions
            .last()
            .cloned()
            .unwrap_or(NetworkReadyState::CLOSED);

        if dev_flag {
            console::log_1(
                &format!("Latest network state for URL {}: {:?}", base_url, latest).into(),
            );
        }

        Ok(latest)
    }

    pub fn version(&self) -> Version {
        match self {
            NetworkReadyState::CONNECTING(ver) => *ver,
            NetworkReadyState::OPEN(state) => state.version,
            NetworkReadyState::CLOSED => 0, // No version for closed state
        }
    }
}

// This function polls the future returning the result of the tunnel initialization if it is ready.
fn pool_op(
    base_url: &str,
    fut: &mut InitEventItem,
    dev_flag: bool,
) -> Option<Result<NetworkReadyState, JsValue>> {
    let noop_waker = futures::task::noop_waker_ref();
    let mut ctx = futures::task::Context::from_waker(&noop_waker);

    match fut.init_event.poll_unpin(&mut ctx) {
        Poll::Ready(val) => match val {
            Ok(init_tunnel_result) => {
                if dev_flag {
                    console::log_1(
                        &format!("Tunnel initialized successfully for base URL: {}", base_url)
                            .into(),
                    );
                }

                // add the result to the cache
                let network_state = Arc::new(NetworkState {
                    http_client: reqwest::Client::new(),
                    init_tunnel_result,
                    forward_proxy_url: fut.forward_proxy_url.clone(),
                    version: fut.version,
                    base_url: base_url.to_string(),
                });

                NETWORK_STATE.with_borrow_mut(|cache| {
                    cache.insert(base_url.to_string(), Arc::clone(&network_state));
                });

                Some(Ok(NetworkReadyState::OPEN(network_state)))
            }

            Err(err) => {
                if dev_flag {
                    console::error_1(
                        &format!(
                            "Failed to initialize tunnel for base URL: {}. Error: {:?}",
                            base_url, err
                        )
                        .into(),
                    );
                }

                Some(Err(err))
            }
        },

        Poll::Pending => {
            if dev_flag {
                console::log_1(
                    &format!(
                        "Tunnel initialization is still pending for base URL: {}",
                        base_url
                    )
                    .into(),
                );
            }

            Some(Ok(NetworkReadyState::CONNECTING(fut.version)))
        }
    }
}
