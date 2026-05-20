use std::convert::TryInto;
use l8_intercept::types::http_caller::{HttpCaller, HttpCallerResponse};
use ntor::common::InitSessionMessage;
use reqwest::{Error, RequestBuilder};
use serde::Deserialize;
use serde_json::json;
use wasm_bindgen::UnwrapThrowExt;

/// A mock implementation of `HttpCaller` for testing purposes, which returns a predefined response.
#[derive(Clone)]
pub struct MockInitTunnelHttpCaller {
    pub data: Vec<u8>,
    pub init: bool,
}

impl HttpCaller for MockInitTunnelHttpCaller {
    async fn send(self, req_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        if self.init {
            let req = req_builder.build()?;
            let pub_key: [u8; 32] = {
                #[derive(Deserialize)]
                struct ExpectedRequest {
                    public_key: Vec<u8>,
                }

                let json_body = serde_json::from_slice::<ExpectedRequest>(
                    req.body()
                        .expect_throw("Request body should be set")
                        .as_bytes()
                        .expect_throw("we expect the body to be bytes"),
                )
                .expect_throw("Failed to deserialize request body to ExpectedRequest struct");

                json_body
                    .public_key
                    .try_into()
                    .expect_throw("Failed to convert to [u8; 32]")
            };

            let server_id = "server123".to_string();
            let ntor_secret = [1, 2]
                .repeat(16)
                .as_slice()
                .try_into()
                .expect_throw("Failed to convert to [u8; 32]");

            let mut ntor_server =
                ntor::server::NTorServer::new_with_secret(server_id.clone(), ntor_secret);

            let init_session_response = {
                // Client initializes session with the server
                let init_session_msg = InitSessionMessage::from(pub_key.to_vec());
                ntor_server.accept_init_session_request(&init_session_msg)
            };

            let cert = ntor_server.get_certificate();

            let response = json!({
                "ephemeral_public_key": init_session_response.public_key(),
                "t_b_hash": init_session_response.t_b_hash(),
                "static_public_key": cert.public_key(),
                "server_id": server_id,
                "int_rp_jwt": "test_jwt1",
                "int_fp_jwt": "test_jwt2",
            });

            return Ok(HttpCallerResponse::Raw(
                serde_json::to_vec(&response).expect_throw("Failed to serialize response to JSON"),
            ));
        }

        Ok(HttpCallerResponse::Raw(self.data))
    }
}