use wasm_bindgen::prelude::*;
use web_sys::{RequestInit, console};

use crate::init_tunnel::init_tunnel;
use crate::storage::InMemoryCache;
use crate::types::{
    http_caller::ActualHttpCaller,
    network_state::{NetworkState, NetworkStateOpen, NetworkStateResponse},
    request::L8RequestObject,
};
use crate::{constants, utils};

/// This API is expected to be a 1:1 mapping of the Fetch API.
/// Arguments:
/// - `resource`: The resource to fetch, which can be a string, a URL object or a Request object.
/// - `options`: Optional configuration for the fetch request, which can include headers, method, body, etc.
#[wasm_bindgen]
pub async fn fetch(
    resource: JsValue,
    options: Option<RequestInit>,
) -> Result<web_sys::Response, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();
    let backend_url = utils::retrieve_resource_url(&resource)?;
    let backend_base_url = utils::get_base_url(&backend_url)?;

    let req_object = L8RequestObject::new(backend_url, resource, options).await?;

    // we can limit the reinitializations to 2 per fetch call and +1 for the initial request
    let mut attempts = constants::FETCH_RETRY_ATTEMPTS;
    loop {
        let network_state = InMemoryCache::get_network_state(&backend_base_url).await?;

        let network_state_open = match network_state.as_ref() {
            NetworkState::OPEN(state) => state,
            _ => {
                // we expect the network state to be open or to have errored out when calling `get_network_state`, report as bug
                return Err(JsValue::from_str(&format!(
                    "Network state for {} is not open. Please report bug to l8 team.",
                    backend_base_url
                )));
            }
        };

        let resp = req_object.l8_send(network_state_open, attempts > 0).await?;

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
                let backend_url = format!(
                    "{}/init-tunnel?backend_url={}",
                    network_state_open.forward_proxy_url, backend_base_url
                );

                if dev_flag {
                    console::log_1(
                        &format!("Reinitializing network state for {}", backend_url).into(),
                    );
                }

                // creating a new NetworkState and overwriting the existing one
                let val = init_tunnel(backend_url, ActualHttpCaller).await?;
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
