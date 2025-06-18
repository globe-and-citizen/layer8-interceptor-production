use bytes::Bytes;
use reqwest::header::HeaderMap;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;
use serde::{Deserialize, Serialize};
use crate::ntor::client::{Client as nTorClient};
use crate::ntor::common::{InitSessionResponse, Certificate};
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

#[wasm_bindgen(getter_with_clone)]
pub struct InitTunnelResult {
    pub client: nTorClient,
    pub ntor_session_id: String
}

#[wasm_bindgen]
pub async fn init_tunnel(backend_url: String) -> Result<InitTunnelResult, JsValue> {
    let mut client = nTorClient::new();

    let init_session_msg = client.initialise_session();

    #[derive(Serialize)]
    struct InitTunnelRequest {
        pub public_key: Vec<u8>
    }

    #[derive(Deserialize)]
    struct InitTunnelResponse {
        public_key: Vec<u8>,
        t_hash: Vec<u8>,
        server_id: String,
        static_public_key: Vec<u8>,
        session_id: String
    }

    let request_body = InitTunnelRequest {
        public_key: init_session_msg.public_key(),
    };

    let response = reqwest::Client::new()
        .post(backend_url)
        .header("Content-Length", "application/json")
        .body(serde_json::to_string(&request_body).unwrap_throw())
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

    let response_bytes = match response.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(err) => {
            console::error_1(&format!("Cannot read response body: {}", err).into());
            return Err(err.into())
        }
    };

    let response_body = serde_json::from_slice::<InitTunnelResponse>(&response_bytes).unwrap_throw();

    let init_msg_response = InitSessionResponse::new(response_body.public_key, response_body.t_hash);

    let server_certificate = Certificate::new(response_body.static_public_key, response_body.server_id);

    let flag = client.handle_response_from_server(&server_certificate, &init_msg_response);

    if !flag {
        return Err(JsValue::from_str("Failed to create nTor Client"))
    };

    let result = InitTunnelResult {
        client: client,
        ntor_session_id: response_body.session_id
    };

    Ok(result)
}