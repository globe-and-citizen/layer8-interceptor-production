use bytes::Bytes;
use reqwest::header::HeaderMap;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;
use crate::utils::js_map_to_headers;

#[wasm_bindgen(getter_with_clone)]
pub struct HttpRequestOptions {
    pub headers: Option<js_sys::Map>
}

#[wasm_bindgen]
impl HttpRequestOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        return HttpRequestOptions{
            headers: None
        }
    }
}

#[wasm_bindgen]
pub async fn http_get(url: String, options: Option<HttpRequestOptions>) -> Result<JsValue, JsValue> {

    let mut header_map = HeaderMap::new();
    if let Some(opts) = options {
        if let Some(headers) = opts.headers {
            header_map = js_map_to_headers(&headers);
        }
    }

    let response = reqwest::Client::new()
        .get(url)
        .headers(header_map)
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

    let body_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            console::error_1(&format!("Cannot read response body: {}", e).into());
            Bytes::from(vec![])
        }
    };
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_throw();
    Ok(serde_wasm_bindgen::to_value(&body).unwrap_throw())
}

#[wasm_bindgen]
pub async fn http_post(url: String, body: JsValue, options: Option<HttpRequestOptions>) -> Result<JsValue, JsValue> {

    let mut header_map = HeaderMap::new();
    if let Some(opts) = options {
        if let Some(headers) = opts.headers {
            header_map = js_map_to_headers(&headers);
        }
    }

    // convert body from JsValue to serde_json::Value
    let body: serde_json::Value = serde_wasm_bindgen::from_value(body.clone()).map_err(|e| JsValue::from_str(&format!("Body parse error: {}", e)))?;

    let response = reqwest::Client::new()
        .post(url)
        .headers(header_map)
        .body(serde_json::to_string(&body).unwrap_throw())
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

    let body_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            console::error_1(&format!("Cannot read response body: {}", e).into());
            Bytes::from(vec![])
        }
    };

    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_throw();
    Ok(serde_wasm_bindgen::to_value(&body).unwrap_throw())
}
