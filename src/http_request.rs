use std::{collections::HashMap, fmt::Debug};

use bytes::Bytes;
use ntor::client::NTorClient;
use ntor::common::{InitSessionResponse, NTorCertificate, NTorParty};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::console;

use crate::utils::{js_map_to_http_header_map, js_map_to_string};

#[wasm_bindgen(getter_with_clone)]
pub struct HttpRequestOptions {
    pub headers: js_sys::Map,
}

#[wasm_bindgen]
impl HttpRequestOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        return HttpRequestOptions {
            headers: js_sys::Map::new(),
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Layer8RequestObject {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, serde_json::Value>,
    pub body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Layer8ResponseObject {
    pub status: u16,
    pub headers: HashMap<String, serde_json::Value>,
    pub body: Vec<u8>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmResponse {
    pub status: u16,
    pub headers: js_sys::Map,
    pub body: JsValue,
}

/// Deprecated
#[wasm_bindgen]
pub async fn http_get(
    url: String,
    options: Option<HttpRequestOptions>,
) -> Result<JsValue, JsValue> {
    let mut header_map = HeaderMap::new();
    if let Some(opts) = options {
        header_map = js_map_to_http_header_map(&opts.headers);
        console::log_1(&format!("Headers: {}", js_map_to_string(&opts.headers)).into());
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
    let body: serde_json::Value = serde_json::from_slice(&body_bytes)
        .expect_throw("Failed to deserialize response body as json");
    Ok(serde_wasm_bindgen::to_value(&body)
        .expect_throw("Failed to convert response body to JsValue"))
}
#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct InitTunnelResult {
    pub(crate) client: NTorClient,
    pub(crate) int_rp_jwt: String,
    pub(crate) int_fp_jwt: String,
}

impl Debug for InitTunnelResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InitTunnelResult {{ ntor_session_id: `not debuggable`, client: `not debuggable` }}", // TODO: implement Debug for NTorClient
        )
    }
}

// #[wasm_bindgen]
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
        #[serde(rename = "jwt1")]
        int_rp_jwt: String,
        #[serde(rename = "jwt2")]
        int_fp_jwt: String,
        server_id: String,
        #[serde(rename = "public_key")]
        static_public_key: Vec<u8>,
    }

    let request_body = InitTunnelRequest {
        public_key: init_session_msg.public_key(),
    };

    let response = reqwest::Client::new()
        .post(backend_url)
        .header("Content-Length", "application/json")
        .body(
            serde_json::to_string(&request_body)
                .expect_throw("Failed to serialize request body to JSON"),
        )
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

    let response_bytes = match response.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(err) => {
            console::error_1(&format!("Cannot read response body: {}", err).into());
            return Err(JsValue::from_str(&format!(
                "Cannot read response body: {:?}",
                err
            )));
        }
    };

    let response_body =
        serde_json::from_slice::<InitTunnelResponse>(&response_bytes).unwrap_throw();

    let init_msg_response =
        InitSessionResponse::new(response_body.ephemeral_public_key, response_body.t_b_hash);

    let server_certificate =
        NTorCertificate::new(response_body.static_public_key, response_body.server_id);

    let flag = client.handle_response_from_server(&server_certificate, &init_msg_response);

    if !flag {
        return Err(JsValue::from_str("Failed to create nTor Client"));
    };

    console::log_1(
        &format!(
            "NTor shared secret: {:?}",
            client.get_shared_secret().unwrap_throw()
        )
        .into(),
    );

    let result = InitTunnelResult {
        client,
        int_rp_jwt: response_body.int_rp_jwt,
        int_fp_jwt: response_body.int_fp_jwt,
    };

    Ok(result)
}
