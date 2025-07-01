use std::{cell::RefCell, collections::HashMap, fmt::format};

use wasm_bindgen::JsValue;

use crate::{
    fetch::fetch_api::PROXY_URL,
    http_request::{InitTunnelResult, init_tunnel},
};

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `InitTunnelResult`. // TODO: adding a client for each InitTunnelResult
    pub(crate) static NETWORK_STATE: RefCell<HashMap<String, (InitTunnelResult, reqwest::Client)>> = RefCell::new(HashMap::new());
}

pub async fn check_state_is_initialized(provider_url: &str) -> Result<(), JsValue> {
    if NETWORK_STATE.with_borrow(|state| state.contains_key(provider_url)) {
        // if the provider is already initialized, return Ok
        return Ok(());
    }

    let client = reqwest::Client::new();

    // try todo tunnel initialization
    let val = init_tunnel(format!("{}/init_tunnel", PROXY_URL)).await?;

    // store the result in the NETWORK_STATE
    NETWORK_STATE.with_borrow_mut(|state| {
        state.insert(provider_url.to_string(), (val.clone(), client));
    });

    Ok(())
}
