use std::collections::HashMap;
use std::rc::Rc;

use ntor::common::NTorParty;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, throw_str};
use web_sys::{AbortSignal, console, Request, RequestInit, ResponseInit};

use crate::{constants, utils};
use crate::fetch::{
    req_properties::add_properties_to_request,
    WasmEncryptedMessage,
};
use crate::types::http_call_indirection::ActualHttpCaller;
use crate::init_tunnel::init_tunnel;
use crate::types::Body;
use crate::types::network_state::{
    DEV_FLAG, get_network_state, NETWORK_STATE, NetworkState, NetworkStateOpen,
};
use crate::utils::{get_base_url, parse_form_data_to_array};

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
    pub headers: HashMap<String, serde_json::Value>,
    pub body: Vec<u8>,

    /* Below fields are present but not used because ResponseInit does not support */
    #[allow(dead_code)]
    pub ok: bool,
    #[allow(dead_code)]
    pub url: String,
    #[allow(dead_code)]
    pub redirected: bool,
    /* Other fields are ignored because rust and wasm do not support */
}

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

// This enum is used to represent the response from the network state.
pub enum NetworkResponse {
    // This is an error in response to the unexpected response from the proxy server.
    ProxyError(JsValue),
    // This is a successful response from the proxy server.
    ProviderResponse(web_sys::Response),
    // This is an indicator that we are reinitializing the connection
    Reinitialize,
}

impl L8RequestObject {
    /// Creates a new L8RequestObject from the given resource or options.
    async fn new(
        backend_url: String,
        resource: JsValue,
        options: Option<RequestInit>,
    ) -> Result<Self, JsValue>
    {
        let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);

        let uri = utils::get_uri(&backend_url)?;

        if dev_flag {
            console::log_1(&format!("Resource URL: {}", uri).into());
        }

        // using the Request object to fetch the resource
        if let Some(req) = resource.dyn_ref::<Request>() {
            return Self::from_web_sys_request_object(uri.clone(), req).await;
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

        return Self::from_request_options(uri, options).await;
    }

    async fn from_request_options(mut uri: String, options: RequestInit) -> Result<Self, JsValue> {
        // Using the resource URL and options object to fetch the resource
        let mut req_wrapper = L8RequestObject {
            uri: uri.clone(),
            ..Default::default()
        };

        req_wrapper.method = match options.get_method() {
            Some(val) => val.trim().to_uppercase(),
            None => String::from("GET"),
        };

        let body = options.get_body();
        if !body.is_undefined() && !body.is_null() {
            let body = utils::parse_js_request_body(body).await.map_err(|e| {
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
                    uri.push_str(&format!("?{}", query));

                    req_wrapper.uri = uri.to_string();
                }

                Body::FormData(form_data) => {
                    let boundary = uuid::Uuid::new_v4().to_string();
                    let data = parse_form_data_to_array(form_data, &boundary).await?;

                    req_wrapper.headers.insert(
                        "Content-Type".to_string(),
                        serde_json::to_value(&format!(
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
                    let bytes = utils::readable_stream_to_bytes(stream.into_raw()).await?;
                    req_wrapper.body = bytes;
                }
            }
        }

        let raw_headers = options.get_headers();
        if !raw_headers.is_undefined() && !raw_headers.is_null() {
            let headers = utils::headers_to_reqwest_headers(raw_headers)?;
            req_wrapper.headers.extend(headers);
        }

        // add properties to the request object
        add_properties_to_request(&mut req_wrapper, &options);

        Ok(req_wrapper)
    }

    async fn from_web_sys_request_object(uri: String, req: &Request) -> Result<Self, JsValue> {
        let mut req_wrapper = L8RequestObject {
            method: req.method().to_string().trim().to_uppercase(),
            uri,
            ..Default::default()
        };

        // The body itself is always represented as a ReadableStream if present, not other types.
        if let Some(readable_stream) = req.body() {
            // Converting a ReadableStream to bytes is needed because HTTP request bodies
            // must be sent as raw data (e.g. Vec<u8>) rather than as a stream object.
            // This allows the request to be serialized, encrypted, or processed before transmission.
            // In Rust and WASM, you cannot directly use a JS ReadableStream as a request body;
            // you must read all its chunks and accumulate them into a byte array for further handling.
            req_wrapper.body = utils::readable_stream_to_bytes(readable_stream)
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to read stream: {:?}", e)))?;
        };

        req_wrapper.headers = utils::headers_to_reqwest_headers(JsValue::from(req.headers()))?;
        req_wrapper.mode = Some(Mode::Cors); // Default mode for Request objects
        return Ok(req_wrapper);
    }

    /// Sends the request using the Layer8 network state.
    /// This method can recurse only once to retry sending the request if it fails.
    /// If the request fails again, it will return an error.
    async fn l8_send(
        &self,
        network_state_open: &NetworkStateOpen,
        reinitialize_attempt: bool,
    ) -> Result<NetworkResponse, JsValue>
    {
        let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);
        let data = serde_json::to_vec(&self).expect_throw(
            "we expect the L8requestObject to be asserted as json serializable at compile time",
        );

        let msg = {
            let (nonce, encrypted) = network_state_open
                .init_tunnel_result
                .client
                .wasm_encrypt(data)
                .map_err(|e| {
                    JsValue::from_str(&format!("Failed to encrypt request data: {}", e))
                })?;

            serde_json::to_vec(&WasmEncryptedMessage {
                nonce: nonce.to_vec(),
                data: encrypted,
            })
                .map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize encrypted message: {}", e))
                })?
        };

        let mut req_builder = network_state_open
            .http_client
            .post(format!("{}/proxy", network_state_open.forward_proxy_url))
            .header("content-type", "application/json")
            .header(
                "int_rp_jwt",
                network_state_open.init_tunnel_result.int_rp_jwt.clone(),
            )
            .header(
                "int_fp_jwt",
                network_state_open.init_tunnel_result.int_fp_jwt.clone(),
            )
            .body(msg);

        if self.body.is_empty() {
            req_builder = req_builder.header("x-empty-body", "true");
        }

        let response_result = req_builder.send().await.inspect_err(|e| {
            if dev_flag {
                console::error_1(&format!("Request failed with error: {}", e).into());
            }
        });

        return match response_result {
            Ok(resp) => Self::handle_response(network_state_open, reinitialize_attempt, resp).await,
            Err(err) => {
                // we can reinitialize the network state
                if reinitialize_attempt {
                    return Ok(NetworkResponse::Reinitialize);
                }

                Err(JsValue::from_str(&format!(
                    "Failed to send request: {}",
                    err
                )))
            }
        };
    }

    async fn handle_response(
        network_state_open: &NetworkStateOpen,
        reinitialize_attempt: bool,
        response: reqwest::Response,
    ) -> Result<NetworkResponse, JsValue>
    {
        let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);

        // status >= 400
        if response.status() >= reqwest::StatusCode::BAD_REQUEST {
            if dev_flag {
                console::log_1(&format!("Received error response from the proxy server: {}", response.status()).into());
            }

            // we can reinitialize the network state
            if reinitialize_attempt {
                return Ok(NetworkResponse::Reinitialize);
            }

            return Ok(NetworkResponse::ProxyError(JsValue::from_str(&format!(
                "Unexpected response from the proxy server: {}; With body: {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No response body".to_string())
            ))));
        }

        let body = &response
            .bytes()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to read response body: {}", e)))?;

        let encrypted_data =
            serde_json::from_slice::<WasmEncryptedMessage>(&body).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to deserialize EncryptedMessage body: {}",
                    e
                ))
            })?;

        let decrypted_response = network_state_open
            .init_tunnel_result
            .client
            .wasm_decrypt(encrypted_data.nonce, encrypted_data.data)
            .map_err(|e| JsValue::from_str(&format!("Failed to decrypt response data: {}", e)))?;

        let l8_response = serde_json::from_slice::<L8ResponseObject>(&decrypted_response)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize response: {}", e)))?;

        if dev_flag {
            console::log_1(&format!("Response: {:?}", l8_response).into());
        }

        // convert L8ResponseObject to web_sys::Response
        let resp_init = ResponseInit::new();
        resp_init.set_status(l8_response.status);
        resp_init.set_status_text(&l8_response.status_text);

        let js_headers = web_sys::Headers::new().expect_throw("Failed to create Headers object");
        for (key, value) in l8_response.headers {
            let value = serde_json::to_string(&value).expect_throw(
                "we expect the header value to be serializable as a JSON string at compile time",
            );

            js_headers
                .append(&key, &value)
                .expect_throw("Failed to append header to Headers object");
        }
        resp_init.set_headers(&js_headers);

        let array = js_sys::Uint8Array::new_with_length(l8_response.body.len() as u32);
        array.copy_from(&l8_response.body);

        match web_sys::Response::new_with_opt_js_u8_array_and_init(Some(&array), &resp_init) {
            Ok(response) => Ok(NetworkResponse::ProviderResponse(response)),
            Err(err) => {
                throw_str(&format!(
                    "Failed to construct JS Response: {:?}",
                    err.as_string()
                ));
            }
        }
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
) -> Result<web_sys::Response, JsValue>
{
    let dev_flag = DEV_FLAG.with_borrow(|flag| *flag);
    let backend_url = utils::retrieve_resource_url(&resource)?;
    let backend_base_url = get_base_url(&backend_url)?;

    let req_object = L8RequestObject::new(backend_url, resource, options).await?;

    // we can limit the reinitializations to 2 per fetch call and +1 for the initial request
    let mut attempts = constants::REINIT_ATTEMPTS;
    loop {
        let network_state = get_network_state(&backend_base_url).await?;
        let network_state_open = match network_state.as_ref() {
            NetworkState::OPEN(state) => state,
            _ => {
                // we expect the network state to be open or to have errored out when calling `get_network_state`, report as bug
                return Err(JsValue::from_str(&format!(
                    "Network state for {} is not open. Please report bug to l8 team.",
                    backend_base_url
                )));
            }
        };

        let resp = req_object.l8_send(network_state_open, attempts > 0).await?;

        // we decrement the attempts, incase we have reinitialized the network state
        attempts -= 1;

        match resp {
            NetworkResponse::ProviderResponse(response) => {
                // If the response is successful, we return it
                return Ok(response);
            }

            NetworkResponse::ProxyError(err) => {
                // If the response is an error, we have exhausted the reinitialization attempts
                if dev_flag {
                    console::error_1(&err);
                }

                return Err(err);
            }

            NetworkResponse::Reinitialize => {
                let backend_url = format!(
                    "{}/init-tunnel?backend_url={}",
                    network_state_open.forward_proxy_url, backend_base_url
                );

                if dev_flag {
                    console::log_1(
                        &format!("Reinitializing network state for {}", backend_url).into(),
                    );
                }

                // creating a new NetworkState and overwriting the existing one
                let val = init_tunnel(backend_url, ActualHttpCaller).await?;
                let state = NetworkStateOpen {
                    http_client: reqwest::Client::new(),
                    init_tunnel_result: val.clone(),
                    forward_proxy_url: network_state_open.forward_proxy_url.clone(),
                };

                NETWORK_STATE.with_borrow_mut(|cache| {
                    cache.insert(backend_base_url.clone(), Rc::new(NetworkState::OPEN(state)));
                });
            }
        }
    }
}
