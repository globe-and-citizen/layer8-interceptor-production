use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::{JsValue, UnwrapThrowExt, prelude::wasm_bindgen};
use web_sys::console;

use ntor::client::NTorClient;
use ntor::common::{InitSessionResponse, NTorCertificate, NTorParty};

use crate::constants::{INIT_TUNNEL_RETRY_ATTEMPTS, INIT_TUNNEL_RETRY_SLEEP_DELAY};
use crate::storage::InMemoryCache;
use crate::types::{
    http_caller::{ActualHttpCaller, HttpCaller, HttpCallerResponse},
    network_state::NetworkStateOpen,
    service_provider::ServiceProvider,
};
use crate::utils;

#[derive(Clone)]
pub struct InitTunnelResult {
    pub(crate) client: NTorClient,
    pub(crate) int_rp_jwt: String,
    pub(crate) int_fp_jwt: String,
}

impl InitTunnelResult {
    fn new() -> Self {
        InitTunnelResult {
            client: NTorClient::new(),
            int_rp_jwt: String::new(),
            int_fp_jwt: String::new(),
        }
    }

    fn generate_ntor_client_public_key(&mut self) -> Vec<u8> {
        let init_session_msg = self.client.initialise_session();
        init_session_msg.public_key()
    }
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

impl InitTunnelResponse {
    fn compute_ntor_handshake(&self, client: &mut NTorClient) -> bool {
        let init_msg_response =
            InitSessionResponse::new(self.ephemeral_public_key.clone(), self.t_b_hash.clone());

        let server_certificate =
            NTorCertificate::new(self.static_public_key.clone(), self.server_id.clone());

        return client.handle_response_from_server(&server_certificate, &init_msg_response);
    }
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
///     - Sending request to backend failed (after INIT_TUNNEL_RETRY_ATTEMPTS retries)
///     - Processing the response failed
///     - NTor handshake failed
pub async fn init_tunnel(
    backend_url: String,
    http_caller: impl HttpCaller,
) -> Result<InitTunnelResult, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();

    // 1. Initialize NTor Client message
    let mut init_tunnel_result = InitTunnelResult::new();
    let request_body = json!({
        "public_key": init_tunnel_result.generate_ntor_client_public_key(),
    });

    // 2. Try to send the request to the backend up to INIT_TUNNEL_RETRY_ATTEMPTS times
    let mut retry_attempt = 0;
    let response: HttpCallerResponse;
    loop {
        retry_attempt += 1;

        let req_builder = reqwest::Client::new()
            .post(backend_url.clone())
            .header("Content-Length", "application/json")
            .header("Retry-count", retry_attempt)
            .body(request_body.to_string());

        match http_caller.clone().send(req_builder).await {
            Ok(res) => {
                response = res;
                break;
            }
            // If it fails, log the error and retry after a short delay
            Err(err) => {
                if dev_flag {
                    console::error_1(
                        &format!("Request attempt {} failed: {}", retry_attempt, err).into(),
                    );
                }

                if retry_attempt >= INIT_TUNNEL_RETRY_ATTEMPTS {
                    console::error_1(
                        &format!("Init-tunnel failed after {} attempts", retry_attempt).into(),
                    );

                    return Err(JsValue::from_str(&format!(
                        "Failed to initialize tunnel after {} attempts: {}",
                        retry_attempt, err
                    )));
                }

                // Wait for a short period before retrying
                utils::sleep(INIT_TUNNEL_RETRY_SLEEP_DELAY).await;
            }
        };
    }

    // 3. Parse the response
    let response_body = match response.bytes().await {
        Ok(bytes) => serde_json::from_slice::<InitTunnelResponse>(&bytes)
            .expect_throw("Failed to deserialize response body to InitTunnelResponse"),
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

    // 4. Complete NTor handshake
    if !response_body.compute_ntor_handshake(&mut init_tunnel_result.client) {
        return Err(JsValue::from_str("Failed to create nTor Client"));
    };

    if dev_flag {
        console::log_1(
            &format!(
                "NTor shared secret: {:?}",
                init_tunnel_result.client.get_shared_secret().expect_throw(
                    "Shared secret should be available after successful tunnel initialization"
                )
            )
            .into(),
        );
    }

    init_tunnel_result.int_rp_jwt = response_body.int_rp_jwt;
    init_tunnel_result.int_fp_jwt = response_body.int_fp_jwt;

    Ok(init_tunnel_result)
}

/// This function initializes the encrypted tunnel for the given service providers using a background process, which updates
/// the `NETWORK_STATE` global static.
#[wasm_bindgen(js_name = "initEncryptedTunnel")]
pub fn init_encrypted_tunnels(
    forward_proxy_url: String,
    service_providers: Vec<ServiceProvider>,
    dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    let dev_flag = InMemoryCache::set_dev_flag(dev_flag);

    for service_provider in service_providers {
        // update the urls as connecting before scheduling the background task to initialize the tunnel
        InMemoryCache::set_connecting_network_state(&service_provider.url);

        let base_url = utils::get_base_url(&service_provider.url)?;
        let backend_url = format!("{}/init-tunnel?backend_url={}", forward_proxy_url, base_url);
        let forward_proxy_url = forward_proxy_url.clone();

        // schedule the background task to initialize the tunnel
        wasm_bindgen_futures::spawn_local(async move {
            match init_tunnel(backend_url, ActualHttpCaller).await {
                Ok(val) => {
                    if dev_flag {
                        console::log_1(
                            &format!("Tunnel initialized for {}", service_provider.url).into(),
                        );
                    }

                    let state = NetworkStateOpen {
                        http_client: reqwest::Client::new(),
                        init_tunnel_result: val,
                        forward_proxy_url: forward_proxy_url.clone(),
                    };

                    InMemoryCache::set_open_network_state(&base_url, state);
                }
                Err(err) => {
                    InMemoryCache::set_errored_network_state(&base_url, err);
                }
            }
        });
    }

    Ok(())
}
