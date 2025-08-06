use std::fmt::Debug;

use ntor::client::NTorClient;
use ntor::common::{InitSessionResponse, NTorCertificate, NTorParty};
use serde::Deserialize;
use serde_json::json;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::console;

#[derive(Clone)]
pub(crate) struct InitTunnelResult {
    pub client: NTorClient,
    pub int_rp_jwt: String,
    pub int_fp_jwt: String,
}

impl Debug for InitTunnelResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InitTunnelResult {{ ntor_session_id: `not debuggable`, client: `not debuggable` }}", // TODO: implement Debug for NTorClient
        )
    }
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

pub async fn init_tunnel(backend_url: String, dev_flag: bool) -> Result<InitTunnelResult, JsValue> {
    let mut ntor_client = NTorClient::new();

    let init_session_msg = ntor_client.initialise_session();
    let request_body = json!({
        "public_key": init_session_msg.public_key()
    });

    let response = reqwest::Client::new()
        .post(backend_url)
        .header("Content-Length", "application/json")
        .body(request_body.to_string())
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

    let response_bytes = match response.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(err) => {
            if dev_flag {
                console::error_1(&format!("Cannot read response body: {}", err).into());
            }

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

    let shared_secret_verification =
        ntor_client.handle_response_from_server(&server_certificate, &init_msg_response);

    if !shared_secret_verification {
        return Err(JsValue::from_str("Failed to create nTor Client"));
    };

    if dev_flag {
        console::log_1(
            &format!(
                "NTor shared secret: {:?}",
                ntor_client.get_shared_secret().unwrap_throw()
            )
            .into(),
        );
    }

    let result = InitTunnelResult {
        client: ntor_client,
        int_rp_jwt: response_body.int_rp_jwt,
        int_fp_jwt: response_body.int_fp_jwt,
    };

    Ok(result)
}
