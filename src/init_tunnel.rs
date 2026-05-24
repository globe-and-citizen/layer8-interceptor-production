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

/// Holds the result of a successfully completed `init-tunnel` handshake.
///
/// After the NTor key exchange finishes, an `InitTunnelResult` is stored in the global
/// network state and reused for every subsequent encrypted request to the associated
/// service provider.
///
/// # Fields
/// * `ntor_client` – The `NTorClient` that has completed the handshake and holds the shared
///   secret used to encrypt/decrypt payloads.
/// * `int_rp_jwt` – A signed JWT issued by the backend (reverse-proxy) that authorizes
///   the current session on the forward-proxy side.
/// * `int_fp_jwt` – A signed JWT issued by the forward-proxy that the client attaches to
///   subsequent requests so the proxy can verify the session.
#[derive(Clone)]
pub struct InitTunnelResult {
    pub ntor_client: NTorClient,
    pub int_rp_jwt: String,
    pub int_fp_jwt: String,
}

impl InitTunnelResult {
    fn new() -> Self {
        InitTunnelResult {
            ntor_client: NTorClient::new(),
            int_rp_jwt: String::new(),
            int_fp_jwt: String::new(),
        }
    }

    fn generate_ntor_client_public_key(&mut self) -> Vec<u8> {
        let init_session_msg = self.ntor_client.initialise_session();
        init_session_msg.public_key()
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

/// Represents the response payload returned by the forward-proxy during the `init-tunnel` handshake.
///
/// This struct is deserialized from the JSON body of the forward-proxy's response and contains
/// all the data required to complete the NTor key exchange and establish an encrypted session.
///
/// # Fields
/// * `ephemeral_public_key` – The server's ephemeral Curve25519 public key generated for this
///   session. Used together with `t_b_hash` to complete the NTor handshake on the client side.
/// * `t_b_hash` – The NTor authentication hash (`T_B`) produced by the server, binding the
///   session to the server's identity and ephemeral key.
/// * `int_rp_jwt` – A signed JWT issued by the backend (reverse-proxy) that authorizes the
///   current session on the forward-proxy side.
/// * `int_fp_jwt` – A signed JWT issued by the forward-proxy that the client attaches to
///   subsequent requests for session verification.
/// * `server_id` – A string identifier for the server, used to look up the server's static
///   public key during certificate validation in the NTor handshake.
/// * `static_public_key` – The server's long-term Curve25519 public key. Combined with
///   `server_id` it forms the `NTorCertificate` used to authenticate the server's identity.
#[derive(Deserialize, Serialize, Debug)]
pub struct InitTunnelResponse {
    pub ephemeral_public_key: Vec<u8>,
    pub t_b_hash: Vec<u8>,
    pub int_rp_jwt: String,
    pub int_fp_jwt: String,
    pub server_id: String,
    pub static_public_key: Vec<u8>,
}

impl InitTunnelResponse {
    /// Deserializes an `InitTunnelResponse` from a raw byte slice.
    ///
    /// Expects `bytes` to contain a valid UTF-8 JSON payload that matches the
    /// `InitTunnelResponse` schema. Panics (via `expect_throw`) if deserialization fails,
    /// which propagates the error to the JavaScript caller as an exception.
    ///
    /// # Parameters
    /// * `bytes` – Raw response body bytes received from the forward-proxy.
    ///
    /// # Panics
    /// Panics with the message `"Failed to deserialize bytes to InitTunnelResponse"` if
    /// `bytes` is not valid JSON or does not match the expected structure.
    fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).expect_throw("Failed to deserialize bytes to InitTunnelResponse")
    }

    /// Completes the NTor handshake on the client side using the server's response data.
    ///
    /// Constructs an [`InitSessionResponse`] from the server's ephemeral public key and
    /// authentication hash, and an [`NTorCertificate`] from the server's static public key
    /// and server identifier. These are then passed to [`NTorClient::handle_response_from_server`]
    /// to derive the shared secret and verify the server's identity.
    ///
    /// # Parameters
    /// * `client` – A mutable reference to the `NTorClient` that sent the initial message.
    ///
    /// # Returns
    /// `true` if the handshake succeeded and a shared secret has been established;
    /// `false` if authentication or key derivation failed.
    fn compute_ntor_handshake(&self, client: &mut NTorClient) -> bool {
        let init_msg_response =
            InitSessionResponse::new(self.ephemeral_public_key.clone(), self.t_b_hash.clone());

        let server_certificate =
            NTorCertificate::new(self.static_public_key.clone(), self.server_id.clone());

        client.handle_response_from_server(&server_certificate, &init_msg_response)
    }
}

/// Establishes an `init-tunnel` request with the forward-proxy, performs the NTor key exchange,
/// and returns an `InitTunnelResult` containing the negotiated `NTorClient` and JWT tokens.
///
/// # Parameters
/// * `backend_url` - The `init-tunnel` endpoint on the forward-proxy. The forward-proxy expects a
///   `backend_url` query parameter which points to the backend (e.g.
///   `https://fp.example.com/init-tunnel?backend_url=https://backend.example.com`).
/// * `http_caller` - An implementation of the `HttpCaller` trait used to send the HTTP request
///   (for example `ActualHttpCaller` in production or a mock in tests).
///
/// # Behavior
/// * Generates the NTor client initialization message and sends it as JSON `{ "public_key": ... }`.
/// * Retries the request up to `INIT_TUNNEL_RETRY_ATTEMPTS` with
///   `INIT_TUNNEL_RETRY_SLEEP_DELAY` between attempts on failure.
/// * Deserializes the response into `InitTunnelResponse` and completes the NTor handshake.
/// * On success returns an `InitTunnelResult` with the established `NTorClient` and JWTs.
///
/// # Returns
/// `Result<InitTunnelResult, JsValue>` - `Ok` on success; `Err(JsValue)` when:
/// * sending the request failed after retries;
/// * the response body could not be read or deserialized;
/// * the NTor handshake failed.
///
/// # Notes
/// When dev logging is enabled (`InMemoryCache::get_dev_flag()`), errors and the NTor shared secret
/// are logged to the web console for debugging.
pub async fn init_tunnel(
    request_url: String,
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
            .post(request_url.clone())
            .header("Content-Type", "application/json")
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
        Ok(bytes) => InitTunnelResponse::from_bytes(&bytes),
        Err(err) => {
            if dev_flag {
                console::error_1(&format!("Cannot read response body: {}", err).into());
            }

            return Err(JsValue::from_str(&format!("Cannot read response body: {:?}", err)));
        }
    };

    // 4. Complete NTor handshake
    if !response_body.compute_ntor_handshake(&mut init_tunnel_result.ntor_client) {
        return Err(JsValue::from_str("Failed to create nTor Client"));
    };

    if dev_flag {
        console::log_1(
            &format!(
                "NTor shared secret: {:?}",
                init_tunnel_result.ntor_client.get_shared_secret().expect_throw(
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

/// Initializes encrypted tunnels for the given service providers.
///
/// Spawns background tasks that perform the `init-tunnel` HTTP handshake with the forward-proxy,
/// complete the NTor key exchange, and update the global network state for each provider.
///
/// # Parameters:
/// * `forward_proxy_url` - URL of the forward proxy (e.g. `https://fp.example.com`).
/// * `service_providers` - Vector of `ServiceProvider` entries to initialize tunnels for.
/// * `dev_flag` - Optional boolean to enable verbose/dev logging.
///
/// # Behavior:
/// * Sets the network state to `connecting` for each provider before scheduling work.
/// * Constructs the `init-tunnel` backend URL and spawns a local async task to call `init_tunnel`.
/// * On success, stores a `NetworkStateOpen` containing an HTTP client, the NTor result and the forward-proxy URL.
/// * On failure, stores an errored network state with the returned error.
///
/// # Returns:
/// * `Ok(())` when background tasks were scheduled successfully.
/// * `Err(JsValue)` if building the backend URL fails synchronously.
#[wasm_bindgen(js_name = "initEncryptedTunnel")]
pub fn init_encrypted_tunnels(
    forward_proxy_url: String,
    service_providers: Vec<ServiceProvider>,
    dev_flag: Option<bool>,
) -> Result<(), JsValue> {
    let dev_flag = InMemoryCache::set_dev_flag(dev_flag);

    for service_provider in service_providers {
        let base_url = utils::get_base_url(&service_provider.url)?;
        let backend_url = format!("{}/init-tunnel?backend_url={}", forward_proxy_url, base_url);
        let forward_proxy_url = forward_proxy_url.clone();

        // update the urls as connecting before scheduling the background task to initialize the tunnel
        InMemoryCache::set_connecting_network_state(&service_provider.url);

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
