use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};

use ntor::common::{EncryptedMessage, NTorParty};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, throw_str};
use wasm_streams::ReadableStream;
use web_sys::{AbortSignal, Request, RequestInit, ResponseInit, console};

use crate::fetch::{formdata::parse_form_data_to_array, req_properties::add_properties_to_request};
use crate::http_request::InitTunnelResult;
use crate::network_state::{NETWORK_STATE, check_state_is_initialized};

/// A JSON serializable wrapper for a request that can be sent using the Fetch API.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct L8RequestObject {
    pub uri: String,
    pub method: String,
    pub headers: HashMap<String, serde_json::Value>,
    pub body: Vec<u8>,

    // User agent configurations
    #[serde(skip)]
    pub body_used: bool,
    #[serde(skip)]
    pub cache: String,
    #[serde(skip)]
    pub credentials: String,
    #[serde(skip)]
    pub destination: String,
    #[serde(skip)]
    pub integrity: String,
    #[serde(skip)]
    pub is_history_navigation: bool,
    #[serde(skip)]
    pub keep_alive: Option<bool>,
    #[serde(skip)]
    pub mode: Option<Mode>,
    #[serde(skip)]
    pub redirect: Option<String>,
    #[serde(skip)]
    pub signal: Option<AbortSignal>,
}

#[derive(Deserialize, Debug)]
pub struct L8ResponseObject {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub const PROXY_URL: &str = "http://localhost:6191"; // TODO: make dynamic

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Mode {
    // Disallows cross-origin requests. If a request is made to another origin with this mode set, the result is an error.
    SameOrigin = 0,
    // Disables CORS for cross-origin requests. The response is opaque, meaning that its headers and body are not available to JavaScript.
    NoCors = 1,
    // If the request is cross-origin then it will use the Cross-Origin Resource Sharing (CORS) mechanism.
    // Using the Request() constructor, the value of the mode property for that Request is set to cors.
    Cors = 2,
    // A mode for supporting navigation. The navigate value is intended to be used only by HTML navigation.
    // A navigate request is created only while navigating between documents.
    Navigate = 3,
}

impl L8RequestObject {
    pub async fn new(
        backend_url: String,
        resource: JsValue,
        options: Option<RequestInit>,
    ) -> Result<Self, JsValue> {
        // retrieve the uri
        let url = url::Url::parse(&backend_url)
            .map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

        let mut uri = url.path().to_string();
        if let Some(query) = url.query() {
            uri.push_str(&format!("?{}", query));
        }

        console::log_1(&format!("Request URI: {}", uri).into());

        // using the Request object to fetch the resource
        if let Some(req) = resource.dyn_ref::<Request>() {
            let mut req_wrapper = L8RequestObject {
                method: req.method().to_string().trim().to_uppercase(),
                uri,
                ..Default::default()
            };

            if let Some(readable_stream) = req.body() {
                req_wrapper.body = readable_stream_to_bytes(readable_stream)
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to read stream: {:?}", e)))?
                    .into();
            };

            req_wrapper.headers = headers_to_reqwest_headers(JsValue::from(req.headers()))?;
            req_wrapper.mode = Some(Mode::Cors); // Default mode for Request objects
            return Ok(req_wrapper);
        }

        let options = match options {
            Some(opts) => opts,
            None => {
                // using only the URL to fetch the resource, with assumed GET method
                return Ok(L8RequestObject {
                    uri,
                    method: String::from("GET"),
                    ..Default::default()
                });
            }
        };

        // Using the resource URL and options object to fetch the resource
        let mut req_wrapper = L8RequestObject {
            uri,
            ..Default::default()
        };

        req_wrapper.method = match options.get_method() {
            Some(val) => val.trim().to_uppercase(),
            None => String::from("GET"),
        };

        let body = options.get_body();
        if !body.is_undefined() && !body.is_null() {
            let body = parse_js_request_body(body).await.map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to parse request body: {}",
                    e.as_string().unwrap_or_else(|| "Unknown error".to_string())
                ))
            })?;

            match body {
                Body::Bytes(bytes) => req_wrapper.body = bytes,

                Body::Params(params) => {
                    let query = params
                        .iter()
                        .map(|(key, value)| format!("{}={}", key, value))
                        .collect::<Vec<String>>()
                        .join("&");

                    // reconstruct the uri
                    let mut uri = url.path().to_string();
                    uri.push_str(&format!("?{}", query));

                    req_wrapper.uri = uri;
                }

                Body::FormData(form_data) => {
                    let boundary = uuid::Uuid::new_v4().to_string();
                    let data = parse_form_data_to_array(form_data, boundary.clone()).await?;

                    // set content type for multipart/form-data
                    req_wrapper.headers.insert(
                        "Content-Type".to_string(),
                        serde_json::from_str(&format!(
                            "multipart/form-data; boundary={}",
                            boundary
                        ))
                        .expect_throw("a valid string is JSON serializable"),
                    );

                    req_wrapper.body = data;
                }

                Body::File(file) => {
                    // Fixme: find out if behavior is a byte array or we should use form data for the request
                    // Ref: <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_a_body>
                    // Convert File to a byte array
                    let file_bytes = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                        .await
                        .expect_throw("Failed to convert File to ArrayBuffer");
                    let uint8_array = js_sys::Uint8Array::new(&file_bytes);

                    req_wrapper.body = uint8_array.to_vec();
                }

                Body::Stream(stream) => {
                    // Convert ReadableStream to bytes
                    let bytes = readable_stream_to_bytes(stream.into_raw()).await?;
                    req_wrapper.body = bytes;
                }
            }
        }

        let raw_headers = options.get_headers();
        if !raw_headers.is_undefined() && !raw_headers.is_null() {
            let headers = headers_to_reqwest_headers(raw_headers)?;
            req_wrapper.headers.extend(headers);
        }

        // add properties to the request object
        add_properties_to_request(&mut req_wrapper, &options);

        Ok(req_wrapper)
    }

    pub async fn send(
        self,
        client: &reqwest::Client,
        init_tunnel: &InitTunnelResult,
    ) -> Result<web_sys::Response, JsValue> {
        let data = serde_json::to_vec(&self).expect_throw(
            "we expect the L8requestObject to be asserted as json serializable at compile time",
        );

        let msg = {
            let encrypted = init_tunnel.client.encrypt(data).map_err(|e| {
                JsValue::from_str(&format!("Failed to encrypt request data: {}", e))
            })?;

            serde_json::to_vec(&encrypted).map_err(|e| {
                JsValue::from_str(&format!("Failed to serialize encrypted message: {}", e))
            })?
        };

        let mut req_builder = client
            .post(format!("{}/proxy", PROXY_URL))
            .header("content-type", "application/json")
            .header("ntor-session-id", init_tunnel.ntor_session_id.clone())
            .body(msg);

        if self.body.is_empty() {
            req_builder = req_builder.header("x-empty-body", "true");
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to send request: {}", e)))?;

        // Just checking for an ok status
        if response.status() != reqwest::StatusCode::OK {
            return Err(JsValue::from_str(&format!(
                "Request failed with status: {}",
                response.status()
            )));
        }

        let body = &response.bytes().await.map_err(|e| {
            JsValue::from_str(&format!("Failed to read response body: {}", e.to_string()))
        })?;

        let encrypted_data = serde_json::from_slice::<EncryptedMessage>(&body).map_err(|e| {
            JsValue::from_str(&format!(
                "Failed to deserialize EncryptedMessage body: {}",
                e
            ))
        })?;

        let decrypted_response = init_tunnel
            .client
            .decrypt(encrypted_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to decrypt response data: {}", e)))?;

        let response = serde_json::from_slice::<L8ResponseObject>(&decrypted_response)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize response: {}", e)))?;

        console::log_1(&format!("Response: {:?}", response).into());

        // convert L8ResponseObject to web_sys::Response
        let resp_init = ResponseInit::new();
        resp_init.set_status(response.status);
        resp_init.set_status_text(&response.status_text);
        let js_headers = web_sys::Headers::new().expect_throw("Failed to create Headers object");
        for (key, value) in response.headers {
            let value = serde_json::to_string(&value).expect_throw(
                "we expect the header value to be serializable as a JSON string at compile time",
            );

            js_headers
                .append(&key, &value)
                .expect_throw("Failed to append header to Headers object");
        }
        resp_init.set_headers(&js_headers);
        let array = js_sys::Uint8Array::new_with_length(response.body.len() as u32);
        array.copy_from(&response.body);
        match web_sys::Response::new_with_opt_js_u8_array_and_init(Some(&array), &resp_init) {
            Ok(response) => Ok(response),
            Err(err) => {
                throw_str(&format!(
                    "Failed to construct JS Response: {:?}",
                    err.as_string()
                ));
            }
        }
    }

    /// Sends the request parts using the provided reqwest client. Not as a serialized object, but the parts of the request
    /// destructured into method, url, body, headers and params.
    #[deprecated = "Use `L8RequestObject::send` instead that encrypts the data and sends to the proxy server."]
    async fn send_request_parts(
        self,
        client: reqwest::Client,
    ) -> Result<web_sys::Response, JsValue> {
        let method = Method::from_str(&self.method)
            .map_err(|e| JsValue::from_str(&format!("Invalid HTTP method: {}", e)))?;
        let mut req_builder = client.request(method, self.uri);

        // set the body if it exists
        req_builder = req_builder.body(self.body);

        // set the headers if they exist
        if !self.headers.is_empty() {
            for (header_name, header_value) in &self.headers {
                req_builder = req_builder.header(header_name, serde_json::to_string(header_value).expect_throw(
                        "we expect the header value to be serializable as a JSON string at compile time",
                    ));
            }
        }

        // set the no-cors mode if it exists
        if let Some(mode) = self.mode {
            if mode as usize == Mode::NoCors as usize {
                req_builder = req_builder.fetch_mode_no_cors();
            }
        }

        let resp_result = req_builder.send().await;

        let resp = match resp_result {
            Ok(response) => response,
            Err(err) => {
                if let Some(abort_signal) = &self.signal {
                    // if there was an abort signal, we log the error add return that instead
                    console::warn_1(
                        &format!("Request failed with error: {}", err.to_string()).into(),
                    );

                    if abort_signal.aborted() {
                        console::warn_1(&"Request was aborted".into());
                        return Err(format!(
                            "Request was aborted, reason: {}",
                            abort_signal
                                .reason()
                                .as_string()
                                .unwrap_or("Unknown reason".to_string())
                        )
                        .into());
                    }
                }

                // If the request fails, we throw an error with the details.
                return Err(JsValue::from_str(&format!(
                    "Failed to send request: {}",
                    err.to_string()
                )));
            }
        };

        // Constructing a web_sys::Response from the reqwest::Response
        Ok(construct_js_response(resp).await)
    }
}

/// This API is expected to be a 1:1 mapping of the Fetch API.
/// Arguments:
/// - `resource`: The resource to fetch, which can be a string, a URL object or a Request object.
/// - `options`: Optional configuration for the fetch request, which can include headers, method, body, etc.
#[wasm_bindgen]
pub async fn fetch(
    resource: JsValue,
    options: Option<RequestInit>,
) -> Result<web_sys::Response, JsValue> {
    // before wrapping the request let's make sure we have the tunnel initialized
    let backend_url = retrieve_resource_url(&resource)?;
    let base_url = {
        let url = url::Url::parse(&backend_url)
            .map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

        // get without query or path fragments
        let mut base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
        if url.port().is_some() {
            base_url = format!("{}:{}", base_url, url.port().unwrap())
        }

        base_url
    };

    check_state_is_initialized(&base_url).await?;
    let network_state = NETWORK_STATE.with_borrow(|cache| {
        let state = match cache.get(&base_url) {
            Some(state) => Arc::clone(state), // This is a reference clone; cannot do interior mutability
            None => {
                // unreachable since we are calling `check_state_is_initialized` before this
                return Err(JsValue::from_str("Network state is not initialized"));
            }
        };

        Ok(state)
    })?;

    let req_object = L8RequestObject::new(backend_url, resource, options).await?;
    let resp = req_object
        .send(&network_state.client, &network_state.keychain)
        .await?;
    Ok(resp)
}

async fn construct_js_response(resp: reqwest::Response) -> web_sys::Response {
    let resp_init = ResponseInit::new();
    {
        // status
        resp_init.set_status(resp.status().as_u16());

        // status text
        resp_init.set_status_text(resp.status().canonical_reason().unwrap_or("OK"));

        // headers
        let js_headers =
            web_sys::Headers::new().expect_throw("Failed to create a new Headers object");
        for (key, value) in resp.headers().iter() {
            js_headers
                .append(
                    key.as_str(),
                    value
                        .to_str()
                        .expect_throw("Expected header value to be a valid UTF-8 string"),
                )
                .expect_throw("Failed to append header to Headers object");
        }

        // logging headers
        console::log_1(&format!("Response Headers: {:?}", resp.headers()).into());

        resp_init.set_headers(&js_headers);
    }

    let body = resp
        .bytes()
        .await
        .expect_throw("Failed to read response body as bytes");
    let array = js_sys::Uint8Array::new_with_length(body.len() as u32);
    array.copy_from(&body);
    match web_sys::Response::new_with_opt_js_u8_array_and_init(Some(&array), &resp_init) {
        Ok(response) => response,
        Err(err) => {
            throw_str(&format!(
                "Failed to construct JS Response: {:?}",
                err.as_string()
            ));
        }
    }
}

// returns the URL of the resource to be fetched
fn retrieve_resource_url(resource: &JsValue) -> Result<String, JsValue> {
    let mut resource_url = String::new();
    if resource.is_string() {
        resource_url = resource
            .as_string()
            .expect_throw("Expected resource to be a string");
    }

    // If the resource is a URL object, we return it stringified.
    if resource.is_instance_of::<web_sys::Url>() {
        return Ok(String::from(
            resource
                .dyn_ref::<web_sys::Url>()
                .expect_throw("Expected resource to be a web_sys::Url")
                .to_string(),
        ));
    }

    if resource.is_instance_of::<web_sys::Request>() {
        resource_url = resource
            .dyn_ref::<web_sys::Request>()
            .expect_throw("Expected resource to be a web_sys::Request")
            .url();
    }

    if resource_url.is_empty() {
        return Err(JsValue::from_str(&format!(
            "Invalid resource type for fetch. Expected a string, URL object, or Request object. Got: {:?}",
            resource.js_typeof(),
        )));
    }

    // validate the URL from string and Request object
    if let Err(err) = web_sys::Url::new(&resource_url) {
        // If the URL is invalid, we throw an error with the details.
        return Err(JsValue::from_str(&format!(
            "Invalid URL: {}. Error: {}",
            resource_url,
            err.as_string()
                .unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    Ok(resource_url)
}

// Ref <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_headers>
// we expect the headers to be either Headers or an Object
fn headers_to_reqwest_headers(
    js_headers: JsValue,
) -> Result<HashMap<String, serde_json::Value>, JsValue> {
    // If the headers are undefined or null, we return an empty HeaderMap
    if js_headers.is_null() || js_headers.is_undefined() {
        return Ok(HashMap::new());
    }

    // We first check if the headers are an instance of web_sys::Headers
    if let Some(headers) = js_headers.dyn_ref::<web_sys::Headers>() {
        return js_headers_to_reqwest_headers(headers);
    }

    console::log_1(&format!("Headers typeof: {:?}", js_headers.js_typeof()).into());

    // we can then check if the headers are an instance of js_sys::Object
    if !js_headers.is_object() {
        return Err(JsValue::from_str(
            "Invalid headers type. Expected Headers or Object.",
        ));
    }

    let headers = js_headers
        .dyn_ref::<js_sys::Object>()
        .expect_throw("Expected headers to be a js_sys::Object");

    // In some cases the headers might be a web_sys::Headers object; this is the case for Request objects.
    if let Some(headers) = headers.dyn_ref::<web_sys::Headers>() {
        // If the headers are a web_sys::Headers object, we can convert them directly
        return js_headers_to_reqwest_headers(headers);
    }

    // [key, value] item array
    let entries = js_sys::Object::entries(headers);
    let mut reqwest_headers = HashMap::new();
    for entry in entries.iter() {
        // [key, value] item array
        let key_value_entry = js_sys::Array::from(&entry);
        let key = key_value_entry.get(0);
        let value = key_value_entry.get(1);
        if key.is_null() || key.is_undefined() || !key.is_string() {
            continue;
        }

        // Convert the key and value to reqwest's HeaderName and HeaderValue
        let header_name = key
            .as_string()
            .expect_throw("Expected header name to be a string");

        let header_value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert header value: {}", e)))?;

        reqwest_headers.insert(header_name, header_value);
    }

    Ok(reqwest_headers)
}

fn js_headers_to_reqwest_headers(
    headers: &web_sys::Headers,
) -> Result<HashMap<String, serde_json::Value>, JsValue> {
    let mut reqwest_headers = HashMap::new();
    for entry in headers.entries() {
        // [key, value] item array
        let key_value_entry = js_sys::Array::from(&entry?);
        let key = key_value_entry.get(0);
        let value = key_value_entry.get(1);

        // Convert the key and value to reqwest's HeaderName and HeaderValue
        let header_name = key
            .as_string()
            .expect_throw("Expected header name to be a string");

        let header_value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert header value: {}", e)))?;

        reqwest_headers.insert(header_name, header_value);
    }

    Ok(reqwest_headers)
}

enum Body {
    Bytes(Vec<u8>),
    Stream(wasm_streams::ReadableStream),
    Params(HashMap<String, String>),
    FormData(web_sys::FormData),
    #[allow(dead_code)]
    File(web_sys::File),
}

// Converts a Javascript request body to a reqwest Body type.
// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_a_body>
async fn parse_js_request_body(body: JsValue) -> Result<Body, JsValue> {
    // You can supply the body as an instance of any of the following types:
    // a string
    // ArrayBuffer
    // TypedArray
    // DataView
    // Blob
    // File
    // URLSearchParams
    // FormData
    // ReadableStream

    // a string
    if body.is_string() {
        return Ok(Body::Bytes(
            body.as_string()
                .expect_throw("Expected body to be a string")
                .into_bytes(),
        ));
    }

    // ArrayBuffer
    if let Some(val) = body.dyn_ref::<js_sys::ArrayBuffer>() {
        let uint8_array = js_sys::Uint8Array::new(val);
        return Ok(Body::Bytes(uint8_array.to_vec()));
    }

    // *TypedArray, todo

    // DataView
    if let Some(val) = body.dyn_ref::<js_sys::DataView>() {
        let uint8_array = js_sys::Uint8Array::new(&val.buffer());
        return Ok(Body::Bytes(uint8_array.to_vec()));
    }

    // Blob
    if let Some(val) = body.dyn_ref::<web_sys::Blob>() {
        let readable_stream = val.stream();
        let body = ReadableStream::from_raw(readable_stream);
        return Ok(Body::Stream(body));
    }

    // File
    if body.is_instance_of::<web_sys::File>() {
        let val = body
            .dyn_into::<web_sys::File>()
            .expect_throw("Expected body to be a web_sys::File");
        let readable_stream = val.stream();
        let body = ReadableStream::from_raw(readable_stream);
        return Ok(Body::Stream(body));
    }

    // URLSearchParams
    if let Some(val) = body.dyn_ref::<web_sys::UrlSearchParams>() {
        let mut params = HashMap::new();
        for entry in val.entries() {
            // [key, value] item array
            let key_value_entry = js_sys::Array::from(
                &entry.expect_throw("Expected entry to be a valid URLSearchParams entry"),
            );
            let key = key_value_entry
                .get(0)
                .as_string()
                .expect_throw("Expected key in URLSearchParams key entry to be a string");
            let value = key_value_entry
                .get(1)
                .as_string()
                .expect_throw("Expected value in URLSearchParams value entry to be a string");
            params.insert(key, value);
        }
        return Ok(Body::Params(params));
    }

    // FormData
    if body.is_instance_of::<web_sys::FormData>() {
        let val = body
            .dyn_into::<web_sys::FormData>()
            .expect_throw("Expected body to be a web_sys::FormData");

        return Ok(Body::FormData(val));
    }

    // ReadableStream
    if body.is_instance_of::<web_sys::ReadableStream>() {
        let readable_stream = body
            .dyn_into::<web_sys::ReadableStream>()
            .expect_throw("Expected body to be a web_sys::ReadableStream");
        let body = ReadableStream::from_raw(readable_stream);
        return Ok(Body::Stream(body));
    }

    // Other objects are converted to strings using their toString() method.
    if let Some(val) = body.dyn_ref::<js_sys::Object>() {
        let val = js_sys::Object::to_string(val)
            .as_string()
            .expect_throw("Expected body to be a string representation of an object");
        return Ok(Body::Bytes(val.into_bytes()));
    }

    Err(JsValue::from_str(
        "Invalid body type for fetch. Expected a string, ArrayBuffer, TypedArray, DataView, Blob, File, URLSearchParams, FormData, or ReadableStream.",
    ))
}

// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read#example_1_-_simple_example>
async fn readable_stream_to_bytes(stream: web_sys::ReadableStream) -> Result<Vec<u8>, JsValue> {
    let reader = stream.get_reader();
    let reader = reader
        .dyn_ref::<web_sys::ReadableStreamDefaultReader>()
        .expect_throw("Expected ReadableStreamDefaultReader, already checked");

    let mut data = Vec::new();
    loop {
        // { done, value }
        // done  - true if the stream has already given you all its data.
        // value - some data. Always undefined when done is true.
        let object = wasm_bindgen_futures::JsFuture::from(reader.read()).await?;

        let done = js_sys::Reflect::get(&object, &"done".into())
            .expect_throw("Expected 'done' property in ReadableStreamDefaultReader.read() result")
            .as_bool()
            .expect_throw(
                "Expected 'done' property to be a boolean in ReadableStreamDefaultReader.read() result",
            );

        if done {
            // If done, we break from the loop and return the accumulated data.
            console::log_1(&format!("Stream read completed with {} bytes", data.len()).into());
            break;
        }

        // value for fetch streams is a Uint8Array
        let value = js_sys::Reflect::get(&object, &"value".into())
            .expect_throw(
                "Expected 'value' property in ReadableStreamDefaultReader.read() result",
            )
            .dyn_into::<js_sys::Uint8Array>()
            .expect_throw(
                "Expected 'value' property to be a Uint8Array in ReadableStreamDefaultReader.read() result",
            )
            .to_vec();

        data.extend_from_slice(&value);
    }

    // Release the reader lock
    reader.release_lock();
    Ok(data)
}
