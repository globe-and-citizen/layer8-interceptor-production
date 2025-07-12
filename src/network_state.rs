use std::{cell::RefCell, collections::HashMap, future::Future, pin::Pin, sync::Arc, task::Poll};

use futures::FutureExt;
use wasm_bindgen::prelude::*;

use crate::http_request::{InitTunnelResult, init_tunnel};

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    pub(crate) static NETWORK_STATE: RefCell<HashMap<String, Arc<NetworkState>>> = RefCell::new(HashMap::new());
    static INIT_EVENT_QUEUE: RefCell<InitEventQueue>= RefCell::new(HashMap::new());
}

/// This event queue is used to store the events that are waiting to be processed.
/// Design:
/// 1. An initialization call is queued in the event queue. This allows it to be polled later to make sure the tunnel initialization happened or errored out.
/// 2. Any calls to the `fetch` API will first check if the tunnel is initialized.
///    - If it is initialized in the NETWORK_STATE, the call is made.
///    - If it is not initialized, the initialization call is polled in the INIT_EVENT_QUEUE to check if it is done.
///    - If the initialization call is done, the fetch call is made.
///    - If the initialization call is not done, the fetch call waits and retries to poll (after x duration?) until the initialization call is done.
/// 3. If the initialization call failed in INIT_EVENT_QUEUE, the fetch call will return an error.
type InitEventQueue = HashMap<String, Pin<Box<dyn Future<Output = Result<(), JsValue>> + 'static>>>;

#[derive(Debug)]
pub(crate) struct NetworkState {
    pub http_client: reqwest::Client,
    pub init_tunnel_result: InitTunnelResult,
    pub forward_proxy_url: String,
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
///
/// Make sure this call is blocking (**is being awaited**) before making any requests to the service providers.,
#[wasm_bindgen(js_name = "initEncryptedTunnel")]
pub fn init_encrypted_tunnel(
    forward_proxy_url: String,
    service_providers: Vec<ServiceProvider>,
    _dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    for service_provider in service_providers {
        let base_url = base_url(&service_provider.url)?;
        match NetworkReadyState::ready_state(&base_url)? {
            NetworkReadyState::OPEN | NetworkReadyState::CONNECTING => {
                // If the network is connecting, we will handle it in the INIT_EVENT_QUEUE
                continue;
            }
            NetworkReadyState::CLOSED => {}
        }

        let init_event: Pin<Box<dyn Future<Output = Result<(), JsValue>> + 'static>> = {
            let backend_url = format!("{}/init-tunnel?backend_url={}", forward_proxy_url, base_url);
            let forward_proxy_url = forward_proxy_url.clone();
            let _dev_flag = _dev_flag.clone();
            let base_url = base_url.clone();
            Box::pin(async {
                let init_tunnel_result = init_tunnel(backend_url).await?;

                let state = NetworkState {
                    http_client: reqwest::Client::new(),
                    init_tunnel_result,
                    forward_proxy_url: forward_proxy_url,
                    _dev_flag: None, // TODO: cloning does not work, find out why
                };

                // store the result in the NETWORK_STATE
                NETWORK_STATE.with_borrow_mut(|cache| {
                    cache.insert(base_url, Arc::new(state));
                });

                Ok(())
            })
        };

        INIT_EVENT_QUEUE.with_borrow_mut(|queue| queue.insert(base_url.clone(), init_event));
    }

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

pub enum NetworkReadyState {
    CONNECTING,
    OPEN,
    CLOSED,
}

impl NetworkReadyState {
    pub fn ready_state(base_url: &str) -> Result<NetworkReadyState, JsValue> {
        if NETWORK_STATE.with_borrow(|cache| cache.get(base_url).is_some()) {
            return Ok(NetworkReadyState::OPEN);
        }

        // confirm if we have an entry in the InitEventQueue, poll if it is ready if not return CONNECTING
        let state: Option<Result<NetworkReadyState, JsValue>> =
            INIT_EVENT_QUEUE.with_borrow_mut(|queue| match queue.get_mut(base_url) {
                Some(fut) => {
                    let noop_waker = futures::task::noop_waker();
                    let mut ctx = futures::task::Context::from_waker(&noop_waker);

                    match fut.poll_unpin(&mut ctx) {
                        Poll::Ready(val) => match val {
                            Ok(_) => {
                                // remove the entry from the queue
                                queue.remove(base_url);
                                Some(Ok(NetworkReadyState::OPEN))
                            }

                            Err(err) => Some(Err(err)),
                        },

                        Poll::Pending => Some(Ok(NetworkReadyState::CONNECTING)),
                    }
                }
                None => None,
            });

        match state {
            Some(val) => Ok(val?),
            None => Ok(NetworkReadyState::CLOSED), // If the base URL is not in the cache, it means the network is not ready
        }
    }
}
