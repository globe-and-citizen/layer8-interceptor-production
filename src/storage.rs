use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::console;
use crate::constants::SLEEP_DELAY;
use crate::types::network_state::NetworkState;
use crate::utils;


thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    pub(crate) static NETWORK_STATE_MAP: RefCell<HashMap<String, Rc<NetworkState>>> = RefCell::new(HashMap::new());

    /// This is a flag to indicate if the dev mode is enabled. It is used to enable or disable the dev mode features like logging.
    pub(crate) static DEV_FLAG: RefCell<bool> = const { RefCell::new(false) };
}

pub(crate) async fn get_network_state(provider_url: &str) -> Result<Rc<NetworkState>, JsValue> {
    let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);
    loop {
        let network_state = NETWORK_STATE_MAP
            .with_borrow(|cache| cache.get(provider_url).map(Rc::clone))
            .ok_or_else(|| {
                JsValue::from_str(&format!(
                    "Network state for {} is not initialized. Please call `await layer8.initEncryptedTunnel(..)` first.",
                    provider_url
                ))
            })?;

        match network_state.as_ref() {
            NetworkState::OPEN { .. } => return Ok(network_state),
            NetworkState::ERRORED(err) => return Err(err.clone()),
            NetworkState::CONNECTING => {
                if dev_flag {
                    console::log_1(
                        &format!("Waiting for network state to be OPEN for {}", provider_url)
                            .into(),
                    );
                }

                utils::sleep(SLEEP_DELAY).await; // wait before checking
                continue;
            }
        }
    }
}
