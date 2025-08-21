use bytes::Bytes;
use reqwest::{Error, RequestBuilder, Response};
use wasm_bindgen::UnwrapThrowExt;

/// Represents the response from an HTTP call, which can either be a `reqwest::Response` or raw data.
#[derive(Debug)]
pub enum HttpCallerResponse {
    Reqwest(Response),
    Raw(Vec<u8>),
}

impl HttpCallerResponse {
    pub async fn bytes(self) -> Result<Bytes, Error> {
        match self {
            HttpCallerResponse::Reqwest(response) => Ok(response.bytes().await.unwrap()),
            HttpCallerResponse::Raw(data) => Ok(data.clone().into()),
        }
    }
}

/// A trait that defines the behavior of an HTTP caller, allowing for different implementations
/// such as actual HTTP requests or mock responses for testing.
pub trait HttpCaller {
    fn send(
        &self,
        request_builder: RequestBuilder,
    ) -> impl Future<Output = Result<HttpCallerResponse, Error>>;
}

/// An marker implementation of `HttpCaller` that uses `reqwest::Client` to send requests.
pub struct ActualHttpCaller;

impl HttpCaller for ActualHttpCaller {
    async fn send(&self, request_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        Ok(HttpCallerResponse::Reqwest(request_builder.send().await?))
    }
}

/// A mock implementation of `HttpCaller` for testing purposes, which returns a predefined response.
pub struct MockHttpCaller {
    pub data: Vec<u8>,
    pub init: bool,
}

impl HttpCaller for MockHttpCaller {
    async fn send(&self, req_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        let req = req_builder.build()?;
        if self.init {
            let pub_key = req
                .body()
                .expect_throw("Request body should be set")
                .as_bytes()
                .expect_throw("we expect the body to be bytes");

            let server_id = "server123".to_string();
            let ntor_secret = [1, 2]
                .repeat(16)
                .as_slice()
                .try_into()
                .expect_throw("Failed to convert to [u8; 32]");

            let mut ntor_server = ntor::server::NTorServer::new_with_secret(server_id, ntor_secret);
        }
        Ok(HttpCallerResponse::Raw(self.data.clone()))
    }
}
