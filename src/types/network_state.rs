use crate::init_tunnel::InitTunnelResult;
use wasm_bindgen::prelude::*;

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

// This enum is used to represent the response from the network state.
pub enum NetworkStateResponse {
    // This is an error in response to the unexpected response from the proxy server.
    ProxyError(JsValue),
    // This is a successful response from the proxy server.
    ProviderResponse(web_sys::Response),
    // This is an indicator that we are reinitializing the connection
    Reinitialize,
}
