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
use crate::constants::MAX_INIT_TUNNEL_ATTEMPTS;

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

/// Establishes `init-tunnel` request to the backend server and performs NTor key exchange.
/// # Arguments
/// * `backend_url` - The `init-tunnel` endpoint of the target server (forward-proxy) includes reverse-proxy's url as a param
/// (eg. https://fp.layer8.net/init-tunnel?backend_url=https://backendwithreverseproxy.layer8.net)
/// * `http_caller` - An implementation of the `HttpCaller` trait to send HTTP requests (either real http call or mock test).
/// # Returns
/// * `InitTunnelResult` if success - Contains the NTor Client and JWT tokens for further communication.
/// * Error if any step fails during the process:
///     - Sending request to backend failed (after MAX_INIT_TUNNEL_ATTEMPTS retries)
///     - Processing the response failed
///     - NTor handshake failed
pub async fn init_tunnel(
    backend_url: String,
    http_caller: impl HttpCaller,
) -> Result<InitTunnelResult, JsValue> {
    let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);

    // 1. Initialize NTor Client message
    let mut client = NTorClient::new();
    let init_session_msg = client.initialise_session();
    let request_body = json!({
        "public_key": init_session_msg.public_key(),
    });

    // 2. Try to send the request to the backend up to MAX_INIT_TUNNEL_ATTEMPTS times
    let mut init_tunnel_retry = 0;
    let response: HttpCallerResponse;
    loop {
        init_tunnel_retry += 1;

        let req_builder = reqwest::Client::new()
            .post(backend_url.clone())
            .header("Content-Length", "application/json")
            .header("Retry-count", init_tunnel_retry)
            .body(request_body.to_string());

        match http_caller.clone().send(req_builder).await {
            Ok(res) => {
                response = res;
                break;
            }
            // If it fails, log the error and retry after a short delay
            Err(err) => {
                if dev_flag {
                    console::error_1(&format!("Request attempt {} failed: {}", init_tunnel_retry, err).into());
                }

                if init_tunnel_retry >= MAX_INIT_TUNNEL_ATTEMPTS {
                    console::error_1(
                        &format!("Failed to initialize tunnel after {} attempts", init_tunnel_retry).into(),
                    );
                    return Err(JsValue::from_str(&format!(
                        "Failed to initialize tunnel after {} attempts: {}",
                        init_tunnel_retry, err
                    )));
                }

                // Wait for a short period (1s) before retrying
                utils::sleep(1000).await;
            }
        };
    }

    // 3. Process the response
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

    // 4. Complete NTor handshake
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
