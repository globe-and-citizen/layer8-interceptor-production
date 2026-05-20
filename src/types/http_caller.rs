use {
    bytes::Bytes,
    hyper::{HeaderMap, StatusCode},
    reqwest::{Error, RequestBuilder, Response},
    serde::de::DeserializeOwned,
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
pub struct ActualHttpCaller; //in-mem there's no allocation

impl HttpCaller for ActualHttpCaller {
    async fn send(self, request_builder: RequestBuilder) -> Result<HttpCallerResponse, Error> {
        Ok(HttpCallerResponse::Reqwest(request_builder.send().await?))
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
