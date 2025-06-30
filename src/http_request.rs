use std::io::Read;
use bytes::Bytes;
use reqwest::header::HeaderMap;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;
use serde::{Deserialize, Serialize};
use crate::ntor::client::{WasmEncryptedMessage};
use crate::utils::{js_map_to_headers, jsvalue_to_vec_u8, map_serialize};
use ntor::common::{InitSessionResponse, NTorCertificate, NTorParty};
use ntor::client::NTorClient;
use crate::utils;

#[wasm_bindgen(getter_with_clone)]
pub struct HttpRequestOptions {
    pub headers: js_sys::Map,
}

#[wasm_bindgen]
impl HttpRequestOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        return HttpRequestOptions {
            headers: js_sys::Map::new()
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct WrappedUserRequest {
    method: String,
    uri: String,
    headers: String,
    body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WrappedBackendResponse {
    status: u16,
    headers: String,
    body: Vec<u8>,
}

#[wasm_bindgen(getter_with_clone)]
struct WasmResponse {
    pub status: u16,
    pub headers: js_sys::Map,
    pub body: JsValue,
}

/// Deprecated
#[wasm_bindgen]
pub async fn http_get(url: String, options: Option<HttpRequestOptions>) -> Result<JsValue, JsValue> {
    let mut header_map = HeaderMap::new();
    if let Some(opts) = options {
        header_map = js_map_to_headers(&opts.headers);
        console::log_1(&format!("Headers: {}", map_serialize(&opts.headers)).into());
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

fn wrap_request(
    uri: String,
    body: JsValue,
    options: Option<HttpRequestOptions>
) -> Result<Vec<u8>, JsValue> {

    let mut serialized_header = "[]".to_string();
    if let Some(opts) = options {
        console::log_1(&format!("Serialized headers: {}", map_serialize(&opts.headers)).into());
        serialized_header = map_serialize(&opts.headers);
    }

    let serialized_body = match jsvalue_to_vec_u8(&body) {
        Ok(vec) => vec,
        Err(e) => {
            console::error_1((&e).into());
            return Err(e);
        }
    };

    let wrapped_request = WrappedUserRequest {
        method: "POST".to_string(),
        uri,
        headers: serialized_header,
        body: serialized_body,
    };
    console::log_1(&format!("WrappedUserRequest: {:?}", wrapped_request).into());

    utils::struct_to_vec(&wrapped_request)
}

#[wasm_bindgen]
pub async fn http_post(
    ntor_result: InitTunnelResult,
    host: String,
    uri: String,
    body: JsValue,
    options: Option<HttpRequestOptions>
) -> Result<WasmResponse, JsValue> {

    // wrap user request to WrappedUserRequest - the Interceptor's `/proxy` request body
    let wrapped_request_bytes = match wrap_request(uri, body, options) {
        Ok(bytes) => bytes,
        Err(e) => {
            console::error_1((&e).into());
            return Err(e);
        }
    };

    // Encrypt the request body using nTor shared secret
    let encrypted_request = match ntor_result.client.wasm_encrypt(wrapped_request_bytes) {
        Ok(encrypted) => encrypted,
        Err(e) => {
            console::error_1(&format!("Encryption error: {}", e.to_string()).into());
            return Err(e.into());
        }
    };
    console::log_1(&format!("EncryptedRequest: {:?}", encrypted_request).into());

    // Send the encrypted request to the FP via `/proxy` endpoint
    let response = reqwest::Client::new()
        .post(format!("{}/proxy", host))
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Headers", "Content-Length")
        .header("ntor-session-id", ntor_result.ntor_session_id)
        .body(serde_json::to_string(&encrypted_request).unwrap_throw())
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

    console::log_1(&format!("Response headers: {:?}", response.headers()).into());

    // parse the response body to WasmEncryptedMessage
    let encrypted_response: WasmEncryptedMessage = match response.bytes().await {
        Ok(bytes) => {
            console::log_1(&format!("Encrypted response body: {}", utils::vec_to_string(bytes.to_vec())).into());

            utils::vec_to_struct(bytes.to_vec())
                .map_err(|e| {
                    console::error_1((&e).into());
                    e
                })?
        },
        Err(e) => {
            console::error_1(&format!("Cannot read response body: {}", e).into());
            return Err(e.into());
        }
    };

    // Decrypt the response body using nTor shared secret
    let decrypted_response = match ntor_result.client.wasm_decrypt(encrypted_response.nonce.to_vec(), encrypted_response.data) {
        Ok(bytes) => {
            console::log_1(&format!("Decrypted response body: {}", utils::vec_to_string(bytes.clone())).into());

            utils::vec_to_struct::<WrappedBackendResponse>(bytes)
                .map_err(|e| {
                    console::error_1((&e).into());
                    e
                })?
        }
        Err(e) => {
            console::error_1(&format!("Decryption error: {}", e.to_string()).into());
            return Err(e.into());
        }
    };

    // Reconstruct the response
    let beHeaders = utils::map_deserialize(&decrypted_response.headers);
    let body: serde_json::Value = utils::vec_to_struct(decrypted_response.body).unwrap_throw();
    let beBody = serde_wasm_bindgen::to_value(&body).unwrap_throw();

    let beResponse = WasmResponse {
        status: decrypted_response.status,
        headers: beHeaders,
        body: beBody,
    };

    return Ok(beResponse)
}

#[wasm_bindgen(getter_with_clone)]
pub struct InitTunnelResult {
    client: ntor::client::NTorClient,
    ntor_session_id: String,
}

#[wasm_bindgen]
pub async fn init_tunnel(backend_url: String) -> Result<InitTunnelResult, JsValue> {
    let mut client = NTorClient::new();

    let init_session_msg = client.initialise_session();

    #[derive(Serialize)]
    struct InitTunnelRequest {
        pub public_key: Vec<u8>,
    }

    #[derive(Deserialize)]
    struct InitTunnelResponse {
        ephemeral_public_key: Vec<u8>,
        t_b_hash: Vec<u8>,
        server_id: String,
        static_public_key: Vec<u8>,
        session_id: String,
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
            return Err(JsValue::from_str(&format!("Cannot read response body: {:?}", err)));
        }
    };

    let response_body = serde_json::from_slice::<InitTunnelResponse>(&response_bytes).unwrap_throw();

    let init_msg_response = InitSessionResponse::new(response_body.ephemeral_public_key, response_body.t_b_hash);

    let server_certificate = NTorCertificate::new(response_body.static_public_key, response_body.server_id);

    let flag = client.handle_response_from_server(&server_certificate, &init_msg_response);

    if !flag {
        return Err(JsValue::from_str("Failed to create nTor Client"));
    };

    console::log_1(&format!("NTor shared secret: {:?}", client.get_shared_secret().unwrap_throw()).into());

    let result = InitTunnelResult {
        client: client,
        ntor_session_id: response_body.session_id,
    };

    Ok(result)
}