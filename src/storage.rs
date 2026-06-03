use crate::constants::FETCH_RETRY_SLEEP_DELAY;
use crate::types::network_state::{NetworkState, NetworkStateOpen};
use crate::utils;

use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::console;

thread_local! {
    /// This is the cache for all the InitTunnelResult present. It is the single source of truth for the state of the system.
    ///
    /// It maps a provider name (e.g., "https://provider.com") to its corresponding `NetworkState`.
    static NETWORK_STATE_MAP: RefCell<HashMap<String, Rc<NetworkState>>> = RefCell::new(HashMap::new());

    /// This is a flag to indicate if the dev mode is enabled. It is used to enable or disable the dev mode features like logging.
    static DEV_FLAG: RefCell<bool> = const { RefCell::new(false) };
}

/// In-memory cache for managing network states and feature flags.
///
/// `InMemoryCache` provides a thread-local, static interface for storing and retrieving
/// [`NetworkState`] entries keyed by provider URL, as well as a developer mode flag.
///
/// # Design
/// All state is stored in `thread_local!` statics, making this type zero-sized with
/// no instances needed — all methods are associated functions (no `self`).
pub struct InMemoryCache {}

impl InMemoryCache {
    /// Retrieves the [`NetworkStateOpen`] for the given `provider_url`, waiting if the
    /// tunnel is still in the `CONNECTING` state.
    ///
    /// # Behavior
    /// - If the state is [`NetworkState::OPEN`], returns the inner [`NetworkStateOpen`] immediately.
    /// - If the state is [`NetworkState::CONNECTING`], sleeps for [`FETCH_RETRY_SLEEP_DELAY`]
    ///   milliseconds and retries in a loop.
    /// - If the state is [`NetworkState::ERRORED`], returns the stored [`JsValue`] error.
    /// - If no entry exists for `provider_url`, returns an error instructing the caller to
    ///   invoke `layer8.initEncryptedTunnel(..)` first.
    ///
    /// # Errors
    /// Returns a [`JsValue`] error string when the network state is missing or errored.
    ///
    /// # Note
    /// This method is intended for **internal use** and public for testing only. It is called by `fetch` and
    /// `L8RequestObject::l8_send` to resolve the current network state prior to making
    /// requests. It transparently handles waiting for tunnel initialization and propagates
    /// any errors encountered. If no initialization process is started, this method will
    /// loop indefinitely.
    pub async fn get_network_state(provider_url: &str) -> Result<NetworkStateOpen, JsValue> {
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
                NetworkState::OPEN(state) => return Ok(state.clone()),
                NetworkState::ERRORED(err) => return Err(err.clone()),
                NetworkState::CONNECTING => {
                    if dev_flag {
                        console::log_1(
                            &format!("Waiting for network state to be OPEN for {}", provider_url)
                                .into(),
                        );
                    }

                    utils::sleep(FETCH_RETRY_SLEEP_DELAY).await; // wait before checking
                    continue;
                }
            }
        }
    }

    /// Inserts or overwrites the network state for `provider_url` with [`NetworkState::CONNECTING`].
    ///
    /// This should be called before initiating an encrypted tunnel handshake so that
    /// concurrent fetch calls know to wait rather than fail immediately.
    pub fn set_connecting_network_state(provider_url: &str) {
        NETWORK_STATE_MAP.with_borrow_mut(|cache| {
            cache.insert(provider_url.to_string(), Rc::new(NetworkState::CONNECTING));
        });
    }

    /// Inserts or overwrites the network state for `provider_url` with [`NetworkState::OPEN`],
    /// storing the fully initialized `state`.
    ///
    /// Call this once the tunnel handshake completes successfully.
    pub fn set_open_network_state(provider_url: &str, state: NetworkStateOpen) {
        NETWORK_STATE_MAP.with_borrow_mut(|cache| {
            cache.insert(provider_url.to_string(), Rc::new(NetworkState::OPEN(state)));
        });
    }

    /// Inserts or overwrites the network state for `provider_url` with [`NetworkState::ERRORED`],
    /// storing the originating `err`.
    ///
    /// Subsequent calls to [`InMemoryCache::get_network_state`] for the same URL will
    /// propagate this error to the caller immediately.
    pub fn set_errored_network_state(provider_url: &str, err: JsValue) {
        NETWORK_STATE_MAP.with_borrow_mut(|cache| {
            cache.insert(
                provider_url.to_string(),
                Rc::new(NetworkState::ERRORED(err)),
            );
        });
    }

    /// Enables dev mode when `flag` is `Some(true)`, logging a confirmation message to the
    /// browser console.
    ///
    /// Dev mode currently activates additional console logging (e.g., tunnel connection wait
    /// messages in [`InMemoryCache::get_network_state`]).
    ///
    /// # Returns
    /// `true` if dev mode was enabled, `false` otherwise.
    pub fn set_dev_flag(flag: Option<bool>) -> bool {
        if let Some(val) = flag {
            if val {
                DEV_FLAG.with_borrow_mut(|dev_flag| *dev_flag = true);
                console::log_1(&"Dev mode enabled".into());
                return true;
            }
        }
        false
    }

    /// Returns the current value of the dev mode flag.
    ///
    /// `true` means dev mode is active; `false` means it is disabled (the default).
    pub fn get_dev_flag() -> bool {
        DEV_FLAG.with_borrow(|dev_flag| *dev_flag)
    }
}
