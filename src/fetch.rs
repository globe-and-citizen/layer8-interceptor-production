use wasm_bindgen::prelude::*;
use web_sys::{RequestInit, console};

use crate::init_tunnel::init_tunnel;
use crate::storage::InMemoryCache;
use crate::types::{
    http_caller::ActualHttpCaller,
    network_state::{NetworkStateOpen, NetworkStateResponse},
    request::L8RequestObject,
};
use crate::{constants, utils};
use crate::types::response::handle_response;

/// Performs an HTTP request compatible with the Web Fetch API, routed through an internal proxy/tunnel.
/// Behavior:
/// - Async function returning `Result<Response, JsValue>`.
/// - Uses `InMemoryCache` to store and retrieve network state (forward proxy URL and tunnel init result).
/// - Resolves the target URL and backend base URL via `utils`.
/// - Constructs an `L8RequestObject` containing request data (backend URL, original `resource`, `options`) and sends it using the current `NetworkStateOpen`.
/// - On network/proxy errors, attempts to reinitialize the tunnel and retry the request.
///
/// Retry and reinitialization:
/// - Performs up to `constants::FETCH_RETRY_ATTEMPTS` reinitialization attempts (plus the initial request).
/// - On send errors calls `init_tunnel` at `{forward_proxy_url}/init-tunnel?backend_url={backend_base_url}` and saves the result in `InMemoryCache`.
/// - If attempts are exhausted or the proxy returns an explicit error, returns `Err(JsValue)`.
///
/// Logging:
/// - When `dev_flag` is set, logs additional messages to the console (`web_sys::console`) during reinitialization and on errors.
///
/// Parameters:
/// - `resource`: `JsValue` — string, `Url` or `Request` specifying the request target.
/// - `options`: `Option<RequestInit>` — optional fetch init (method, headers, body, etc.).
///
/// Returns:
/// - `Ok(web_sys::Response)` on successful provider response.
/// - `Err(JsValue)` on network/proxy errors or failed tunnel initialization.
#[wasm_bindgen]
pub async fn fetch(
    resource: JsValue,
    options: Option<RequestInit>,
) -> Result<web_sys::Response, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();
    let backend_url = utils::retrieve_resource_url(&resource)?;
    let backend_base_url = utils::get_base_url(&backend_url)?;

    let req_object = L8RequestObject::new(backend_url, resource, options).await?;

    // we can limit the reinitialization to 2 per fetch call and +1 for the initial request
    let mut attempts = constants::FETCH_RETRY_ATTEMPTS;
    loop {
        let network_state_open = InMemoryCache::get_network_state(&backend_base_url).await?;

        let resp = match req_object.l8_send(&network_state_open, ActualHttpCaller).await {
            Ok(resp) => handle_response(&network_state_open, attempts > 0, resp).await?,
            Err(err) => {
                // we can reinitialize the network state
                if attempts <= 0 {
                    return Err(err)
                };

                NetworkStateResponse::Reinitialize
            }
        };

        // we decrement the attempts, incase we have reinitialized the network state
        attempts -= 1;
        match resp {
            NetworkStateResponse::ProviderResponse(response) => {
                // If the response is successful, we return it
                return Ok(response);
            }

            NetworkStateResponse::ProxyError(err) => {
                // If the response is an error, we have exhausted the reinitialization attempts
                if dev_flag {
                    console::error_1(&err);
                }

                return Err(err);
            }

            NetworkStateResponse::Reinitialize => {
                let request_url = format!(
                    "{}/init-tunnel?backend_url={}",
                    network_state_open.forward_proxy_url, backend_base_url
                );

                if dev_flag {
                    console::log_1(
                        &format!("Reinitializing network state for {}", request_url).into(),
                    );
                }

                // creating a new NetworkState and overwriting the existing one
                let val = init_tunnel(request_url, ActualHttpCaller).await?;
                let state = NetworkStateOpen {
                    http_client: reqwest::Client::new(),
                    init_tunnel_result: val.clone(),
                    forward_proxy_url: network_state_open.forward_proxy_url.clone(),
                };

                InMemoryCache::set_open_network_state(&backend_base_url, state);
            }
        }
    }
}
