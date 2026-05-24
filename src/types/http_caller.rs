use {
    bytes::Bytes,
    hyper::{HeaderMap, StatusCode},
    serde::de::DeserializeOwned,
    std::future::Future,
};

#[derive(Debug)]
pub struct MockHttpResponse {
    pub status: StatusCode,
    pub status_str: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub url: url::Url,
}

#[derive(Debug)]
pub struct MockHttpError {
    pub msg: String,
}

/// Represents the response from an HTTP call, which can either be a `reqwest::Response` or raw data.
#[derive(Debug)]
pub enum HttpCallerResponse {
    Reqwest(reqwest::Response),
    Mock(MockHttpResponse),
    Raw(Vec<u8>),
    MockErr(MockHttpError),
}

/// A trait that defines the behavior of an HTTP caller, allowing for different implementations
/// such as actual HTTP requests or mock responses for testing.
pub trait HttpCaller: Clone {
    fn send(
        self,
        request_builder: reqwest::RequestBuilder,
    ) -> impl Future<Output = Result<HttpCallerResponse, reqwest::Error>>;
}

/// An marker implementation of `HttpCaller` that uses `reqwest::Client` to send requests.
#[derive(Clone)]
pub struct ActualHttpCaller; //in-mem there's no allocation

impl HttpCaller for ActualHttpCaller {
    async fn send(
        self,
        request_builder: reqwest::RequestBuilder,
    ) -> Result<HttpCallerResponse, reqwest::Error> {
        Ok(HttpCallerResponse::Reqwest(request_builder.send().await?))
    }
}

impl HttpCallerResponse {
    #[inline]
    pub fn status(&self) -> StatusCode {
        match self {
            HttpCallerResponse::Reqwest(response) => response.status(),
            HttpCallerResponse::Raw(_) => StatusCode::OK,
            HttpCallerResponse::Mock(response) => response.status,
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        match self {
            HttpCallerResponse::Reqwest(response) => response.headers(),
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
            HttpCallerResponse::Mock(response) => &response.headers,
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        match self {
            HttpCallerResponse::Reqwest(response) => response.headers_mut(),
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
            HttpCallerResponse::Mock(response) => &mut response.headers,
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub fn content_length(&self) -> Option<u64> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.content_length(),
            HttpCallerResponse::Raw(data) => Some(data.len() as u64),
            HttpCallerResponse::Mock(response) => response.body.len().try_into().ok(),
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub fn url(&self) -> &url::Url {
        match self {
            HttpCallerResponse::Reqwest(response) => response.url(),
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
            HttpCallerResponse::Mock(response) => &response.url,
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub async fn json<T: DeserializeOwned>(self) -> reqwest::Result<T> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.json().await,
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
            HttpCallerResponse::Mock(response) => Ok(serde_json::from_slice(&response.body)
                .expect("failed to deserialize mock response body as JSON")),
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub async fn text(self) -> reqwest::Result<String> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.text().await,
            HttpCallerResponse::Raw(_) => unimplemented!("not implemented for tests"),
            HttpCallerResponse::Mock(response) => {
                Ok(String::from_utf8_lossy(&response.body).into_owned())
            }
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }

    #[inline]
    pub async fn bytes(self) -> reqwest::Result<Bytes> {
        match self {
            HttpCallerResponse::Reqwest(response) => response.bytes().await,
            HttpCallerResponse::Raw(data) => Ok(data.clone().into()),
            HttpCallerResponse::Mock(response) => Ok(Bytes::from(response.body)),
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
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
            HttpCallerResponse::Mock(response) => {
                if response.status.is_client_error() || response.status.is_server_error() {
                    panic!("mock response returned error status: {}", response.status);
                }
                Ok(HttpCallerResponse::Mock(response))
            }
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
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
            HttpCallerResponse::Mock(response) => {
                if response.status.is_client_error() || response.status.is_server_error() {
                    panic!("mock response returned error status: {}", response.status);
                }
                Ok(self)
            }
            HttpCallerResponse::MockErr(_) => unimplemented!("not implemented for tests"),
        }
    }
}
