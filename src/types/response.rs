use crate::storage::InMemoryCache;
use crate::types::http_caller::HttpCallerResponse;
use crate::types::network_state::{NetworkStateOpen, NetworkStateResponse};
use crate::utils;
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::{JsValue, throw_str};
use web_sys::{ResponseInit, console};

/// Represents a decrypted HTTP response received from the Layer8 proxy.
///
/// This struct is deserialized from the JSON payload returned by the `/proxy` endpoint
/// after decryption. It is then reconstructed into a browser-native [`web_sys::Response`].
#[derive(Deserialize, Debug, Clone)]
pub struct L8ResponseObject {
    /// HTTP status code (e.g. `200`, `401`, `500`, etc.).
    pub status: u16,

    /// HTTP status text (e.g. `"OK"`, `"Unauthorized"`, etc.).
    pub status_text: String,

    /// Response headers as a map of header name to JSON value.
    pub headers: HashMap<String, serde_json::Value>,

    /// Raw response body bytes.
    pub body: Vec<u8>,

    /* Below fields are present but not used because ResponseInit does not support */
    /// Whether the response was successful (`status` in the range 200–299).
    /// Not forwarded to the reconstructed response — `ResponseInit` has no such field.
    #[allow(dead_code)]
    pub ok: bool,

    /// The final URL of the response after any redirects.
    /// Not forwarded — `ResponseInit` has no such field.
    #[allow(dead_code)]
    pub url: String,

    /// Whether the request was redirected before reaching the final URL.
    /// Not forwarded — `ResponseInit` has no such field.
    #[allow(dead_code)]
    pub redirected: bool,
    /* Other fields are ignored because rust and wasm do not support */
}

impl L8ResponseObject {
    /// Reconstructs a browser-native [`web_sys::Response`] from this object.
    ///
    /// Sets `status`, `status_text`, and `headers` via [`ResponseInit`], then attaches
    /// `body` as a [`js_sys::Uint8Array`].
    ///
    /// # Panics (via `throw_str`)
    /// Panics with a JS exception if the underlying `Response` constructor fails.
    ///
    /// # Errors
    /// Returns `Err(JsValue)` if header conversion fails.
    ///
    /// # Notes
    /// - `Set-Cookie` headers are silently dropped; the browser's `Headers` API does
    ///   not allow setting them from JavaScript.
    pub fn reconstruct_js_response(&self) -> Result<web_sys::Response, JsValue> {
        let resp_init = ResponseInit::new();
        resp_init.set_status(self.status);
        resp_init.set_status_text(&self.status_text);

        let js_headers = utils::hashmap_to_js_headers(&self.headers)?;
        resp_init.set_headers(&js_headers);

        let mut body = None;
        let array = js_sys::Uint8Array::new_with_length(self.body.len() as u32);
        if !self.body.is_empty() {
            array.copy_from(&self.body);
            body = Some(&array);
        };

        // we lost "Set-Cookie" header here
        match web_sys::Response::new_with_opt_js_u8_array_and_init(body, &resp_init) {
            Ok(response) => Ok(response),
            Err(err) => {
                throw_str(&format!("Failed to construct JS Response: {:?}", err));
            }
        }
    }
}

/// Processes a raw HTTP response from the internal caller and maps it to a
/// [`NetworkStateResponse`].
///
/// # Flow
///
/// **Error responses (`status >= 400`):**
/// - If `reinitialize_attempt` is `true`, returns [`NetworkStateResponse::Reinitialize`]
///   so the caller can re-establish the tunnel.
/// - Otherwise returns [`NetworkStateResponse::ProxyError`] with status and body details.
///
/// **Success responses:**
/// 1. Reads the raw bytes from the response body.
/// 2. Decrypts them with the current ntor session via [`NetworkStateOpen::ntor_decrypt`].
/// 3. Deserializes the plaintext JSON into an [`L8ResponseObject`].
/// 4. Reconstructs and returns a [`web_sys::Response`] wrapped in
///    [`NetworkStateResponse::ProviderResponse`].
///
/// # Errors
/// Returns `Err(JsValue)` if any of the following fail:
/// - Reading the response body.
/// - Ntor decryption.
/// - JSON deserialization.
/// - JS `Response` reconstruction.
pub async fn handle_response(
    network_state_open: &NetworkStateOpen,
    reinitialize_attempt: bool,
    proxy_response: HttpCallerResponse,
) -> Result<NetworkStateResponse, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();

    if dev_flag {
        console::log_1(&format!("Proxy response: {:?}", proxy_response).into());
    }

    // status >= 400
    if proxy_response.status() >= reqwest::StatusCode::BAD_REQUEST {
        if dev_flag {
            console::log_1(
                &format!(
                    "Received error response from the proxy server: {}",
                    proxy_response.status()
                )
                .into(),
            );
        }

        // we can reinitialize the network state
        if reinitialize_attempt {
            return Ok(NetworkStateResponse::Reinitialize);
        }

        return Ok(NetworkStateResponse::ProxyError(JsValue::from_str(
            &format!(
                "Unexpected response from the proxy server: {}; With body: {}",
                proxy_response.status(),
                proxy_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No response body".to_string())
            ),
        )));
    }

    let body = &proxy_response
        .bytes()
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to read response body: {}", e)))?;

    if dev_flag {
        console::log_1(&format!("Encrypted response body: {:?}", body.to_vec()).into());
    }

    let decrypted_response = network_state_open.ntor_decrypt(body)?;

    let l8_response = serde_json::from_slice::<L8ResponseObject>(&decrypted_response)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize response: {}", e)))?;

    if dev_flag {
        console::log_1(&format!("Response: {:?}", l8_response).into());
    }

    // convert L8ResponseObject to web_sys::Response
    let js_response = l8_response.reconstruct_js_response()?;
    Ok(NetworkStateResponse::ProviderResponse(js_response))
}
