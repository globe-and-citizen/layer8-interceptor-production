use std::{cell::RefCell, collections::HashMap, sync::Arc};

use wasm_bindgen::prelude::*;

use crate::http_request::{InitTunnelResult, init_tunnel};

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    pub(crate) static NETWORK_STATE: RefCell<HashMap<String, Arc<NetworkState>>> = RefCell::new(HashMap::new());
}

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
pub async fn init_encrypted_tunnel(
    forward_proxy_url: String,
    service_providers: Vec<ServiceProvider>,
    _dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    for service_provider in service_providers {
        let base_url = base_url(&service_provider.url)?;
        if NETWORK_STATE.with_borrow(|cache| cache.contains_key(&base_url)) {
            // if the provider is already initialized, skip
            continue;
        }

        let init_tunnel_result = init_tunnel(format!(
            "{}/init-tunnel?backend_url={}",
            forward_proxy_url, base_url
        ))
        .await?;

        let state = NetworkState {
            http_client: reqwest::Client::new(),
            init_tunnel_result,
            forward_proxy_url: forward_proxy_url.clone(),
            _dev_flag,
        };

        // store the result in the NETWORK_STATE
        NETWORK_STATE.with_borrow_mut(|cache| {
            cache.insert(base_url, Arc::new(state));
        });
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
