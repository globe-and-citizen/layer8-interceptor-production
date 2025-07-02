use std::{cell::RefCell, collections::HashMap, sync::Arc};

use wasm_bindgen::JsValue;

use crate::{
    fetch::fetch_api::PROXY_URL,
    http_request::{InitTunnelResult, init_tunnel},
};

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    pub(crate) static NETWORK_STATE: RefCell<HashMap<String, Arc<NetworkState>>> = RefCell::new(HashMap::new());
}

pub(crate) struct NetworkState {
    pub client: reqwest::Client,
    pub keychain: InitTunnelResult,
}

pub async fn check_state_is_initialized(provider_url: &str) -> Result<(), JsValue> {
    if NETWORK_STATE.with_borrow(|state| state.contains_key(provider_url)) {
        // if the provider is already initialized, return Ok
        return Ok(());
    }

    let keychain = init_tunnel(format!(
        "{}/init-tunnel?backend_url={}",
        PROXY_URL, provider_url
    ))
    .await?;

    let state = NetworkState {
        client: reqwest::Client::new(),
        keychain,
    };

    // store the result in the NETWORK_STATE
    NETWORK_STATE.with_borrow_mut(|cache| {
        cache.insert(provider_url.to_string(), Arc::new(state));
    });

    Ok(())
}
