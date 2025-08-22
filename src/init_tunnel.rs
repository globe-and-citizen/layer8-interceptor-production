use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::{JsValue, UnwrapThrowExt, prelude::wasm_bindgen};
use web_sys::console;

use ntor::client::NTorClient;
use ntor::common::{InitSessionResponse, NTorCertificate, NTorParty};

use crate::utils;
use crate::{
    http_call_indirection::{HttpCaller, HttpCallerResponse},
    network_state::DEV_FLAG,
};

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct InitTunnelResult {
    pub(crate) client: NTorClient,
    pub(crate) int_rp_jwt: String,
    pub(crate) int_fp_jwt: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InitTunnelResponse {
    pub ephemeral_public_key: Vec<u8>,
    pub t_b_hash: Vec<u8>,
    #[serde(rename = "jwt1")]
    pub int_rp_jwt: String,
    #[serde(rename = "jwt2")]
    pub int_fp_jwt: String,
    pub server_id: String,
    #[serde(rename = "public_key")]
    pub static_public_key: Vec<u8>,
}

impl Debug for InitTunnelResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InitTunnelResult {{ int_fp_jwt: {},\n int_rp_jwt: {},\n client: `not debuggable` }}", // TODO: implement Debug for NTorClient
            self.int_fp_jwt, self.int_rp_jwt
        )
    }
}

pub async fn init_tunnel(
    backend_url: String,
    http_caller: impl HttpCaller,
) -> Result<InitTunnelResult, JsValue> {
    let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);
    let mut client = NTorClient::new();

    let init_session_msg = client.initialise_session();
    let request_body = json!({
        "public_key": init_session_msg.public_key(),
    });

    let mut count = 0;
    let response: HttpCallerResponse;
    loop {
        count += 1;

        let req_builder = reqwest::Client::new()
            .post(backend_url.clone())
            .header("Content-Length", "application/json")
            .header("Retry-count", count)
            .body(request_body.to_string());

        match http_caller.clone().send(req_builder).await {
            Ok(res) => {
                response = res;
                break;
            }
            Err(err) => {
                console::error_1(&format!("Request failed: {}. Attempt: {}", err, count).into());

                if count >= 3 {
                    console::error_1(
                        &format!("Failed to initialize tunnel after {} attempts", count).into(),
                    );
                    return Err(JsValue::from_str(&format!(
                        "Failed to initialize tunnel after {} attempts: {}",
                        count, err
                    )));
                }
                // Wait for a short period (1s) before retrying
                utils::sleep(1000).await;
            }
        };
    }

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

    let response_body = serde_json::from_slice::<InitTunnelResponse>(&response_bytes)
        .expect_throw("Failed to deserialize response body to InitTunnelResponse");

    let init_msg_response =
        InitSessionResponse::new(response_body.ephemeral_public_key, response_body.t_b_hash);

    let server_certificate =
        NTorCertificate::new(response_body.static_public_key, response_body.server_id);

    let flag = client.handle_response_from_server(&server_certificate, &init_msg_response);

    if !flag {
        return Err(JsValue::from_str("Failed to create nTor Client"));
    };

    if dev_flag {
        console::log_1(
            &format!(
                "NTor shared secret: {:?}",
                client.get_shared_secret().expect_throw(
                    "Shared secret should be available after successful tunnel initialization"
                )
            )
            .into(),
        );
    }

    let result = InitTunnelResult {
        client,
        int_rp_jwt: response_body.int_rp_jwt,
        int_fp_jwt: response_body.int_fp_jwt,
    };

    Ok(result)
}
