use wasm_bindgen::prelude::*;
use crate::init_tunnel::InitTunnelResult;

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

