use std::{cell::RefCell, collections::HashMap, future::Future, pin::Pin, sync::Arc, task::Poll};

use futures::FutureExt;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::http_request::{InitTunnelResult, init_tunnel};

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    pub(crate) static NETWORK_STATE: RefCell<HashMap<String, Arc<NetworkState>>> = RefCell::new(HashMap::new());
    static INIT_EVENT_QUEUE: RefCell<HashMap<String, InitEventItem>>= RefCell::new(HashMap::new());
}

// This event queue item is used to store the events that are waiting to be processed.
// Design:
// 1. An initialization call is queued in the event queue. This allows it to be polled later to make sure the tunnel initialization happened or errored out.
// 2. Any calls to the `fetch` API will first check if the tunnel is initialized.
//    - If it is initialized in the NETWORK_STATE, the call is made.
//    - If it is not initialized, the initialization call is polled in the INIT_EVENT_QUEUE to check if it is done.
//    - If the initialization call is done, the fetch call is made.
//    - If the initialization call is not done, the fetch call waits and retries to poll (after x duration?) until the initialization call is done.
// 3. If the initialization call failed in INIT_EVENT_QUEUE, the fetch call will return an error.
struct InitEventItem {
    init_event: Pin<Box<dyn Future<Output = Result<InitTunnelResult, JsValue>> + 'static>>,
    forward_proxy_url: String,
    version: Version,
    _dev_flag: Option<bool>,
}

// This is an alias value to track the version of the tunnel. It is incremented every time a new tunnel is initialized.
pub(crate) type Version = u16;

#[derive(Debug)]
pub(crate) struct NetworkState {
    pub http_client: reqwest::Client,
    pub init_tunnel_result: InitTunnelResult,
    pub forward_proxy_url: String,
    pub version: Version,
    pub _dev_flag: Option<bool>,
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ServiceProvider {
    url: String,
    _options: Option<js_sys::Object>, // for now, options is just any object including empty
}

#[wasm_bindgen]
impl ServiceProvider {
    pub fn new(url: String, _options: Option<js_sys::Object>) -> Self {
        ServiceProvider { url, _options }
    }
}

/// This function initializes the encrypted tunnel for the given service providers.
/// It checks if the provider already has an initialized tunnel, if not it initializes a new tunnel
/// and stores the result.
#[wasm_bindgen(js_name = "initEncryptedTunnel")]
pub fn init_encrypted_tunnel(
    forward_proxy_url: String,
    service_providers: Vec<ServiceProvider>,
    _dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    for service_provider in service_providers {
        let base_url = base_url(&service_provider.url)?;
        schedule_init_event(&base_url, 1, forward_proxy_url.clone(), _dev_flag)?;
    }

    Ok(())
}

pub(crate) fn schedule_init_event(
    base_url: &str,
    expected_next_version: Version,
    forward_proxy_url: String,
    _dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    // version is already connecting or connected, return early
    let current_version = NetworkReadyState::ready_state(base_url)?.version();
    if current_version >= expected_next_version {
        return Ok(());
    }

    let backend_url = format!("{}/init-tunnel?backend_url={}", forward_proxy_url, base_url);
    let init_event = InitEventItem {
        forward_proxy_url,
        version: current_version + 1,
        _dev_flag,
        init_event: Box::pin(init_tunnel(backend_url)),
    };

    INIT_EVENT_QUEUE.with_borrow_mut(|queue| queue.insert(base_url.to_string(), init_event));
    Ok(())
}

pub fn base_url(url: &str) -> Result<String, JsValue> {
    let url =
        url::Url::parse(url).map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    // get without query or path fragments
    let mut base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
    if let Some(port) = url.port() {
        base_url = format!("{}:{}", base_url, port);
    }

    Ok(base_url)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NetworkReadyState {
    CONNECTING(Version),
    OPEN(Version),
    CLOSED,
}

impl NetworkReadyState {
    /// This function checks the current state of the network for the given base URL. It will only return the state of the latest version
    /// if there are multiple versions of the network state.
    pub fn ready_state(base_url: &str) -> Result<NetworkReadyState, JsValue> {
        let mut versions = Vec::new();
        if let Some(version) =
            NETWORK_STATE.with_borrow(|cache| cache.get(base_url).map(|val| val.version))
        {
            versions.push(NetworkReadyState::OPEN(version));
        }

        // check if there's a version in the INIT_EVENT_QUEUE
        let init_queue_item: Option<Result<NetworkReadyState, JsValue>> = INIT_EVENT_QUEUE
            .with_borrow_mut(|queue| match queue.get_mut(base_url) {
                Some(fut) => pool_op(base_url, fut),
                None => None,
            });

        match init_queue_item {
            Some(val) => {
                let state = match val {
                    Ok(val) => val,
                    Err(err) => {
                        // We failed to initialize, we;re popping this item from the queue
                        INIT_EVENT_QUEUE.with_borrow_mut(|queue| {
                            queue.remove(base_url);
                        });

                        return Err(err);
                    }
                };

                if let NetworkReadyState::OPEN(..) = state {
                    INIT_EVENT_QUEUE.with_borrow_mut(|queue| {
                        queue.remove(base_url);
                    });
                }

                versions.push(state);
            }
            None => {
                // If the base URL is not in the cache or event queue, it means it was never initialized.
                console::warn_1(
                    &format!(
                        "No init event found for URL: {}. Assuming it is already open.",
                        base_url
                    )
                    .into(),
                );

                if versions.is_empty() {
                    return Ok(NetworkReadyState::CLOSED);
                }
            }
        }

        versions.sort_by_key(|a| a.version());

        let latest = versions
            .last()
            .cloned()
            .unwrap_or(NetworkReadyState::CLOSED);

        console::log_1(&format!("URL: {}. Network state version: {:?}", base_url, latest).into());

        Ok(latest)
    }

    pub fn version(&self) -> Version {
        match self {
            NetworkReadyState::CONNECTING(ver) => *ver,
            NetworkReadyState::OPEN(ver) => *ver,
            NetworkReadyState::CLOSED => 0, // No version for closed state
        }
    }
}

// This function polls the future returning the result of the tunnel initialization if it is ready.
fn pool_op(base_url: &str, fut: &mut InitEventItem) -> Option<Result<NetworkReadyState, JsValue>> {
    let noop_waker = futures::task::noop_waker_ref();
    let mut ctx = futures::task::Context::from_waker(&noop_waker);

    match fut.init_event.poll_unpin(&mut ctx) {
        Poll::Ready(val) => match val {
            Ok(init_tunnel_result) => {
                console::log_1(
                    &format!("Tunnel initialized successfully for base URL: {}", base_url).into(),
                );

                // add the result to the cache
                let network_state = NetworkState {
                    http_client: reqwest::Client::new(),
                    init_tunnel_result,
                    forward_proxy_url: fut.forward_proxy_url.clone(),
                    version: fut.version,
                    _dev_flag: fut._dev_flag,
                };

                NETWORK_STATE.with_borrow_mut(|cache| {
                    cache.insert(base_url.to_string(), Arc::new(network_state));
                });

                Some(Ok(NetworkReadyState::OPEN(fut.version)))
            }

            Err(err) => {
                console::error_1(
                    &format!(
                        "Error initializing tunnel for base URL: {}. Error: {:?}",
                        base_url, err
                    )
                    .into(),
                );
                Some(Err(err))
            }
        },

        Poll::Pending => {
            console::log_1(
                &format!("Network is still initializing for base URL: {}", base_url).into(),
            );

            Some(Ok(NetworkReadyState::CONNECTING(fut.version)))
        }
    }
}
