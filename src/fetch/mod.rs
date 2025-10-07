use wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};

pub mod fetch_api;
mod request;
mod response;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Mode {
    // Disallows cross-origin requests. If a request is made to another origin with this mode set, the result is an error.
    SameOrigin = 0,
    // Disables CORS for cross-origin requests. The response is opaque, meaning that its headers and body are not available to JavaScript.
    NoCors = 1,
    // If the request is cross-origin then it will use the Cross-Origin Resource Sharing (CORS) mechanism.
    // Using the Request() constructor, the value of the mode property for that Request is set to cors.
    Cors = 2,
    // A mode for supporting navigation. The navigate value is intended to be used only by HTML navigation.
    // A navigate request is created only while navigating between documents.
    Navigate = 3,
}

// This enum is used to represent the response from the network state.
pub enum NetworkResponse {
    // This is an error in response to the unexpected response from the proxy server.
    ProxyError(JsValue),
    // This is a successful response from the proxy server.
    ProviderResponse(web_sys::Response),
    // This is an indicator that we are reinitializing the connection
    Reinitialize,
}
