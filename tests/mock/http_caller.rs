use crate::mock::data::{MockData, get_mock_data};
use l8_intercept::types::http_caller::{
    HttpCaller, HttpCallerResponse, MockHttpError, MockHttpResponse,
};
use l8_intercept::types::request::L8RequestObject;
use ntor::common::{EncryptedMessage, InitSessionMessage, NTorParty};
use ntor::server::NTorServer;
use reqwest::{Error, RequestBuilder};
use serde::Deserialize;
use serde_json::json;
use std::convert::TryInto;
use wasm_bindgen::UnwrapThrowExt;

/// A mock implementation of `HttpCaller` for testing purposes, which returns a predefined response.
#[derive(Clone)]
pub struct MockHttpCaller {
    pub data: Vec<u8>,
    pub mock_data: Option<MockData>,
    pub init: bool,
    pub is_proxy: bool,
}

impl HttpCaller for MockHttpCaller {
    async fn send(self, req_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        if self.init {
            return self.mock_init_tunnel_response(req_builder);
        };

        if self.is_proxy {
            return self.mock_proxy_response(req_builder);
        }

        Ok(HttpCallerResponse::Raw(self.data))
    }
}

impl MockHttpCaller {
    pub fn mock_init_tunnel_response(
        self,
        req_builder: RequestBuilder,
    ) -> Result<HttpCallerResponse, Error> {
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

        Ok(HttpCallerResponse::Raw(
            serde_json::to_vec(&response).expect_throw("Failed to serialize response to JSON"),
        ))
    }

    pub fn mock_proxy_response(
        self,
        req_builder: RequestBuilder,
    ) -> Result<HttpCallerResponse, Error> {
        let req = req_builder.build()?;

        let body_bytes = req
            .body()
            .expect_throw("Request body should be set")
            .as_bytes()
            .expect_throw("we expect the body to be bytes");

        let mock_data = self.mock_data.unwrap().clone();

        let encrypted_msg = EncryptedMessage::from_bytes(body_bytes)
            .expect_throw("Failed to deserialize request body to EncryptedMessage");

        let mut ntor_server = NTorServer::new("server_id".to_string());
        ntor_server.set_shared_secret(mock_data.shared_secret.to_vec());

        let decrypted_msg = ntor_server
            .decrypt(*encrypted_msg)
            .expect_throw("Failed to decrypt the request body with NTorServer");
        let l8_req = serde_json::from_slice::<L8RequestObject>(&decrypted_msg)
            .expect_throw("Failed to deserialize decrypted message to L8RequestObject");

        if (l8_req.uri, l8_req.method, l8_req.headers, l8_req.body)
            != (
                mock_data.l8_request_object.uri,
                mock_data.l8_request_object.method,
                mock_data.l8_request_object.headers,
                mock_data.l8_request_object.body,
            )
        {
            return Ok(HttpCallerResponse::Err(MockHttpError {
                msg: "Decrypted request body does not match expected mock data".to_string(),
            }));
        }

        Ok(HttpCallerResponse::Mock(MockHttpResponse {
            status: reqwest::StatusCode::OK,
            status_str: "OK".to_string(),
            headers: Default::default(),
            body: mock_data.response_encrypted_body,
            url: url::Url::parse("http://placeholder.net/").unwrap(),
        }))
    }
}
