use std::{collections::HashMap, str::FromStr};

use js_sys::Object;
use reqwest::{Method, RequestBuilder, header::HeaderValue};
use wasm_bindgen::{prelude::*, throw_str};
use web_sys::{ReadableStreamDefaultReader, Request, RequestInit, ResponseInit, console};

/// This API is expected to be a 1:1 mapping of the Fetch API.
/// Arguments:
/// - `resource`: The resource to fetch, which can be a string, a URL object or a Request object.
/// - `options`: Optional configuration for the fetch request, which can include headers, method, body, etc.
#[wasm_bindgen]
pub async fn fetch(
    resource: JsValue,
    options: Option<RequestInit>,
) -> Result<web_sys::Response, JsValue> {
    console::log_1(&format!("Fetching resource: {:?}", resource).into());

    let client = reqwest::Client::new();
    let url = retrieve_resource_url(&resource)?;

    // using the Request object to fetch the resource
    if let Some(req) = resource.dyn_ref::<Request>() {
        let method = Method::from_str(&req.method().trim().to_uppercase())
            .map_err(|e| JsValue::from_str(&format!("Invalid HTTP method: {}", e)))?;

        let data = match req.body() {
            Some(val) => readable_stream_to_bytes(val).await?,
            None => Vec::new(),
        };

        let headers = js_headers_to_reqwest_headers(JsValue::from(req.headers()))?;
        let resp = client
            .request(method, url)
            .headers(headers)
            .body(data)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to fetch resource: {}", e)))?;

        return Ok(construct_js_response(resp).await);
    }

    // Using the resource URL and options object to fetch the resource
    if let Some(options) = options {
        let method = match options.get_method() {
            Some(val) => Method::from_str(&val.trim().to_uppercase()).unwrap_throw(),
            None => Method::GET,
        };

        let mut req_builder = client.request(method, &url);

        let body = options.get_body();
        if body != JsValue::UNDEFINED || body != JsValue::NULL {
            let body = parse_js_request_body(body).await.map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to parse request body: {}",
                    e.as_string().unwrap_or_else(|| "Unknown error".to_string())
                ))
            })?;

            match body {
                Body::Bytes(bytes) => req_builder = req_builder.body(bytes),

                Body::Params(params) => {
                    let encoded_params =
                        params
                            .into_iter()
                            .map(|(k, v)| (k, v))
                            .collect::<Vec<(String, String)>>();

                    req_builder = req_builder.query(encoded_params.as_slice());
                }

                Body::FormData(form_data) => {
                    // req_builder = parse_form_data(req_builder, form_data).await?;
                    // FIXME: Convert FormData to a byte array, we are missing out on the file upload parts
                    let mut form = HashMap::new();
                    for entry in form_data.entries() {
                        let key_value_entry = js_sys::Array::from(&entry.unwrap_throw());
                        let key = key_value_entry.get(0).as_string().unwrap_throw();
                        let value = key_value_entry.get(1).as_string().unwrap_throw();

                        form.insert(key, value);
                    }

                    req_builder = req_builder.form(&form);
                }

                Body::File(file) => {
                    // Fixme: find out if behavior is a byte array or we should use formdata for the request
                    // Ref: <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_a_body>
                    // Convert File to a byte array
                    let file_bytes = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                        .await
                        .unwrap_throw();
                    let uint8_array = js_sys::Uint8Array::new(&file_bytes);
                    req_builder = req_builder.body(uint8_array.to_vec());
                }
            }
        }

        if options.get_headers() != JsValue::UNDEFINED || options.get_headers() != JsValue::NULL {
            let headers = js_headers_to_reqwest_headers(options.get_headers())?;
            req_builder = req_builder.headers(headers);
        }

        let resp = req_builder
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to fetch resource: {}", e)))?;

        console::log_1(&format!("Response: {:?}", resp).into());

        return Ok(construct_js_response(resp).await);
    }

    // using only the URL to fetch the resource, with assumed GET method
    match client.get(url).send().await {
        Ok(resp) => Ok(construct_js_response(resp).await),
        Err(err) => {
            throw_str(&format!("Failed to fetch resource: {}", err));
        }
    }
}

async fn construct_js_response(resp: reqwest::Response) -> web_sys::Response {
    // TODO: only transfers status, status_text, headers and body ref <https://stackoverflow.com/a/76425824/10020745>
    // This approach misses out on properties like `Response.bodyUsed`... <https://developer.mozilla.org/en-US/docs/Web/API/Response#instance_properties>
    let resp_init = ResponseInit::new();
    {
        // status
        resp_init.set_status(resp.status().as_u16());

        // status text
        resp_init.set_status_text(resp.status().canonical_reason().unwrap_or("OK"));

        // headers
        let js_headers = web_sys::Headers::new().unwrap_throw();
        for (key, value) in resp.headers().iter() {
            js_headers
                .append(key.as_str(), value.to_str().unwrap_throw())
                .unwrap_throw();
        }

        // logging headers
        console::log_1(&format!("Response Headers: {:?}", resp.headers()).into());

        resp_init.set_headers(&js_headers);
    }

    let body = resp.bytes().await.unwrap_throw();
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
        resource_url = resource.as_string().unwrap_throw();
    }

    // If the resource is a URL object, we return it stringified.
    if resource.is_instance_of::<web_sys::Url>() {
        return Ok(String::from(
            resource
                .dyn_ref::<web_sys::Url>()
                .unwrap_throw()
                .to_string(),
        ));
    }

    if resource.is_instance_of::<web_sys::Request>() {
        resource_url = resource.dyn_ref::<web_sys::Request>().unwrap_throw().url();
    }

    if resource_url.is_empty() {
        return Err(JsValue::from_str(&format!(
            "Invalid resource type for fetch. Expected a string, URL object, or Request object. Got: {:?}",
            resource.js_typeof(),
        )));
    }

    // validate the URL from string and Request object
    if !web_sys::Url::new(&resource_url).is_ok() {
        Err(JsValue::from_str(&format!("Invalid URL: {}", resource_url)))?;
    }

    Ok(resource_url)
}

// Ref <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_headers>
// we expect the headers to be either Headers or an Object
fn js_headers_to_reqwest_headers(
    js_headers: JsValue,
) -> Result<reqwest::header::HeaderMap<HeaderValue>, JsValue> {
    // If the headers are undefined or null, we return an empty HeaderMap
    if js_headers.is_null() || js_headers.is_undefined() {
        return Ok(reqwest::header::HeaderMap::new());
    }

    // We first check if the headers are an instance of web_sys::Headers
    if let Some(headers) = js_headers.dyn_ref::<web_sys::Headers>() {
        let mut reqwest_headers = reqwest::header::HeaderMap::new();
        for entry in headers.entries() {
            // [key, value] item array
            let key_value_entry = js_sys::Array::from(&entry?);
            let key = key_value_entry.get(0);
            let value = key_value_entry.get(1);

            // Convert the key and value to reqwest's HeaderName and HeaderValue
            let header_name = reqwest::header::HeaderName::from_str(
                &key.as_string().unwrap_throw(),
            )
            .map_err(|_| {
                JsValue::from_str(&format!(
                    "Invalid header name: {}",
                    key.as_string().unwrap_throw()
                ))
            })?;
            let header_value = reqwest::header::HeaderValue::from_str(
                &value.as_string().unwrap_throw(),
            )
            .map_err(|_| {
                JsValue::from_str(&format!(
                    "Invalid header value: {}",
                    value.as_string().unwrap_throw()
                ))
            })?;

            reqwest_headers.insert(header_name, header_value);
        }
    }

    // we can then check if the headers are an instance of js_sys::Object
    if let Some(headers) = js_headers.dyn_ref::<Object>() {
        // [key, value] item array
        let entries = js_sys::Object::entries(&headers);

        let mut reqwest_headers = reqwest::header::HeaderMap::new();
        for entry in entries.iter() {
            // [key, value] item array
            let key_value_entry = js_sys::Array::from(&entry);
            let key = key_value_entry.get(0);
            let value = key_value_entry.get(1);
            if key.is_null() || key.is_undefined() || !key.is_string() {
                continue;
            }

            // Convert the key and value to reqwest's HeaderName and HeaderValue
            let header_name = reqwest::header::HeaderName::from_str(
                &key.as_string().unwrap_throw(),
            )
            .map_err(|_| {
                JsValue::from_str(&format!(
                    "Invalid header name: {}",
                    key.as_string().unwrap_throw()
                ))
            })?;

            let header_value = reqwest::header::HeaderValue::from_str(
                &value.as_string().unwrap_throw(),
            )
            .map_err(|_| {
                JsValue::from_str(&format!(
                    "Invalid header value: {}",
                    value.as_string().unwrap_throw()
                ))
            })?;

            reqwest_headers.insert(header_name, header_value);
        }

        return Ok(reqwest_headers);
    }

    Err(JsValue::from_str(
        "Invalid headers type. Expected Headers or Object.",
    ))
}

enum Body {
    Bytes(Vec<u8>),
    Params(HashMap<String, String>),
    FormData(web_sys::FormData),
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
        return Ok(Body::Bytes(body.as_string().unwrap_throw().into_bytes()));
    }

    // ArrayBuffer
    if let Some(val) = body.dyn_ref::<js_sys::ArrayBuffer>() {
        let uint8_array = js_sys::Uint8Array::new(&val);
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
        let array_buffer = wasm_bindgen_futures::JsFuture::from(val.array_buffer()).await?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        return Ok(Body::Bytes(uint8_array.to_vec()));
    }

    // File
    if body.is_instance_of::<web_sys::File>() {
        let val = body.dyn_into::<web_sys::File>().unwrap_throw();
        return Ok(Body::File(val));
    }

    // URLSearchParams
    if let Some(val) = body.dyn_ref::<web_sys::UrlSearchParams>() {
        let mut params = HashMap::new();
        for entry in val.entries() {
            // [key, value] item array
            let key_value_entry = js_sys::Array::from(&entry.unwrap_throw());
            let key = key_value_entry.get(0).as_string().unwrap_throw();
            let value = key_value_entry.get(1).as_string().unwrap_throw();
            params.insert(key, value);
        }
        return Ok(Body::Params(params));
    }

    // FormData
    if body.is_instance_of::<web_sys::FormData>() {
        let val = body.dyn_into::<web_sys::FormData>().unwrap_throw();
        return Ok(Body::FormData(val));
    }

    // ReadableStream
    if body.is_instance_of::<web_sys::ReadableStream>() {
        let stream = body.dyn_into::<web_sys::ReadableStream>().unwrap_throw();
        let bytes = readable_stream_to_bytes(stream).await?;
        return Ok(Body::Bytes(bytes));
    }

    // Other objects are converted to strings using their toString() method.
    if let Some(val) = body.dyn_ref::<js_sys::Object>() {
        let val = js_sys::Object::to_string(val).as_string().unwrap_throw();
        return Ok(Body::Bytes(val.into_bytes()));
    }

    return Err(JsValue::from_str(
        "Invalid body type for fetch. Expected a string, ArrayBuffer, TypedArray, DataView, Blob, File, URLSearchParams, FormData, or ReadableStream.",
    ));
}

// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read#example_1_-_simple_example>
async fn readable_stream_to_bytes(stream: web_sys::ReadableStream) -> Result<Vec<u8>, JsValue> {
    let reader = stream.get_reader();
    let reader = reader
        .dyn_ref::<ReadableStreamDefaultReader>()
        .unwrap_throw();

    let mut data = Vec::new();
    loop {
        // { done, value }
        // done  - true if the stream has already given you all its data.
        // value - some data. Always undefined when done is true.
        let object = wasm_bindgen_futures::JsFuture::from(reader.read()).await?;

        let done = js_sys::Reflect::get(&object, &"done".into())
            .unwrap_throw()
            .as_bool()
            .unwrap_throw();

        if done {
            // If done, we break from the loop and return the accumulated data.
            console::log_1(&format!("Stream read completed with {} bytes", data.len()).into());
            break;
        }

        // value for fetch streams is a Uint8Array
        let value = js_sys::Reflect::get(&object, &"value".into())
            .unwrap_throw()
            .dyn_into::<js_sys::Uint8Array>()
            .unwrap_throw()
            .to_vec();

        data.extend_from_slice(&value);
    }

    Ok(data)
}

#[allow(dead_code)]
async fn parse_form_data(
    _: RequestBuilder,
    _: web_sys::FormData,
) -> Result<RequestBuilder, JsValue> {
    unimplemented!(
        "Please check <https://github.com/globe-and-citizen/layer8-interceptor-rs/blob/main/src/js_glue/formdata_polyfill.ts> for a polyfill of FormData in JS"
    );
}
