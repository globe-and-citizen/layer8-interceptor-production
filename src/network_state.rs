use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::{
    constants::SLEEP_DELAY,
    http_call_indirection::ActualHttpCaller,
    init_tunnel::{InitTunnelResult, init_tunnel},
    utils,
};

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    pub(crate) static NETWORK_STATE: RefCell<HashMap<String, Rc<NetworkState>>> = RefCell::new(HashMap::new());

    /// This is a flag to indicate if the dev mode is enabled. It is used to enable or disable the dev mode features like logging.
    pub(crate) static DEV_FLAG: RefCell<bool> = const { RefCell::new(false) };
}

/// Represents the current state of the network connection for a service provider.
#[derive(Debug)]
pub(crate) enum NetworkState {
    /// The network is currently being established.
    CONNECTING,
    /// The network is open and ready for use.
    OPEN(NetworkStateOpen),
    /// An error occurred while trying to establish the network connection.
    ERRORED(JsValue),
}

/// This is the state of the network connection for a service provider when it has
/// completed key exchange and is ready to be used.
#[derive(Debug)]
pub(crate) struct NetworkStateOpen {
    pub http_client: reqwest::Client,
    pub init_tunnel_result: InitTunnelResult,
    pub forward_proxy_url: String,
}

/// Represents a service provider that can be used to request for resources.
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

/// This function initializes the encrypted tunnel for the given service providers using a background process, which updates
/// the `NETWORK_STATE` global static.
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
        // update the urls as connecting before scheduling the background task to initialize the tunnel
        NETWORK_STATE.with_borrow_mut(|cache| {
            cache.insert(
                service_provider.url.clone(),
                Rc::new(NetworkState::CONNECTING),
            );
        });

        let base_url = base_url(&service_provider.url)?;
        let backend_url = format!("{}/init-tunnel?backend_url={}", forward_proxy_url, base_url);
        let forward_proxy_url = forward_proxy_url.clone();

        // schedule the background task to initialize the tunnel
        wasm_bindgen_futures::spawn_local(async move {
            match init_tunnel(backend_url, ActualHttpCaller).await {
                Ok(val) => {
                    if dev_flag.unwrap_or(false) {
                        console::log_1(
                            &format!("Tunnel initialized for {}", service_provider.url).into(),
                        );
                    }

                    let state = NetworkStateOpen {
                        http_client: reqwest::Client::new(),
                        init_tunnel_result: val,
                        forward_proxy_url: forward_proxy_url.clone(),
                    };

                    NETWORK_STATE.with_borrow_mut(|cache| {
                        cache.insert(base_url, Rc::new(NetworkState::OPEN(state)));
                    });
                }

                Err(err) => {
                    NETWORK_STATE.with_borrow_mut(|cache| {
                        cache.insert(base_url, Rc::new(NetworkState::ERRORED(err)));
                    });
                }
            }
        });
    }

    Ok(())
}

pub(crate) async fn get_network_state(
    provider_url: &str,
    dev_flag: bool,
) -> Result<Rc<NetworkState>, JsValue> {
    loop {
        let network_state = NETWORK_STATE
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
