use {
    bytes::Bytes,
    hyper::{HeaderMap, StatusCode},
    ntor::common::InitSessionMessage,
    reqwest::{Error, RequestBuilder, Response},
    serde::{Deserialize, de::DeserializeOwned},
    serde_json::json,
    wasm_bindgen::UnwrapThrowExt,
};

/// Represents the response from an HTTP call, which can either be a `reqwest::Response` or raw data.
#[derive(Debug)]
pub enum HttpCallerResponse {
    Reqwest(Response),
    Raw(Vec<u8>),
}

/// A trait that defines the behavior of an HTTP caller, allowing for different implementations
/// such as actual HTTP requests or mock responses for testing.
pub trait HttpCaller: Clone {
    fn send(
        self,
        request_builder: RequestBuilder,
    ) -> impl Future<Output = Result<HttpCallerResponse, Error>>;
}

/// An marker implementation of `HttpCaller` that uses `reqwest::Client` to send requests.
#[derive(Clone)]
pub struct ActualHttpCaller;

impl HttpCaller for ActualHttpCaller {
    async fn send(self, request_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        Ok(HttpCallerResponse::Reqwest(request_builder.send().await?))
    }
}

/// A mock implementation of `HttpCaller` for testing purposes, which returns a predefined response.
#[derive(Clone)]
pub struct MockHttpCaller {
    pub data: Vec<u8>,
    pub init: bool,
}

impl HttpCaller for MockHttpCaller {
    async fn send(self, req_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        let req = req_builder.build()?;
        if self.init {
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
                "public_key": cert.public_key(),
                "server_id": server_id,
                "jwt1": "test_jwt1",
                "jwt2": "test_jwt2",
            });

            return Ok(HttpCallerResponse::Raw(
                serde_json::to_vec(&response).expect_throw("Failed to serialize response to JSON"),
            ));
        }

        Ok(HttpCallerResponse::Raw(self.data))
    }
}

impl HttpCallerResponse {
    #[inline]
    pub fn status(&self) -> StatusCode {
        match self {
            HttpCallerResponse::Reqwest(response) => response.status(),
            HttpCallerResponse::Raw(_) => StatusCode::OK,
        }
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        match self {
            HttpCallerResponse::Reqwest(response) => response.headers(),
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        match self {
            HttpCallerResponse::Reqwest(response) => response.headers_mut(),
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub fn content_length(&self) -> Option<u64> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.content_length(),
            HttpCallerResponse::Raw(data) => Some(data.len() as u64),
        }
    }

    #[inline]
    pub fn url(&self) -> &url::Url {
        match self {
            HttpCallerResponse::Reqwest(response) => response.url(),
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub async fn json<T: DeserializeOwned>(self) -> reqwest::Result<T> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.json().await,
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub async fn text(self) -> reqwest::Result<String> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.text().await,
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub async fn bytes(self) -> reqwest::Result<Bytes> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.bytes().await,
            HttpCallerResponse::Raw(data) => Ok(data.clone().into()),
        }
    }

    #[inline]
    pub fn error_for_status(self) -> reqwest::Result<Self> {
        match self {
            HttpCallerResponse::Reqwest(response) => {
                response.error_for_status_ref()?;
                Ok(HttpCallerResponse::Reqwest(response))
            }
            HttpCallerResponse::Raw(data) => Ok(HttpCallerResponse::Raw(data)),
        }
    }

    #[inline]
    pub fn error_for_status_ref(&self) -> reqwest::Result<&Self> {
        match self {
            HttpCallerResponse::Reqwest(response) => {
                response.error_for_status_ref()?;
                Ok(self)
            }
            HttpCallerResponse::Raw(_) => Ok(self),
        }
    }
}
