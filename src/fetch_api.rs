use std::str::FromStr;

use reqwest::{Method, header::HeaderValue};
use wasm_bindgen::{prelude::*, throw_str};
use web_sys::{ReadableStreamDefaultReader, Request, RequestInit, ResponseInit};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// This API is expected to be a 1:1 mapping of the Fetch API.
/// Arguments:
/// - `resource`: The resource to fetch, which can be a string, a URL object or a Request object.
/// - `options`: Optional configuration for the fetch request, which can include headers, method, body, etc.
#[wasm_bindgen]
pub async fn fetch(resource: JsValue, options: Option<RequestInit>) -> web_sys::Response {
    log(&format!("Fetching resource: {:?}", resource));

    let client = reqwest::Client::new();
    let url = retrieve_resource_url(&resource);

    // using the Request object to fetch the resource
    if let Some(req) = resource.dyn_ref::<Request>() {
        let method = Method::from_str(&req.method().to_uppercase()).unwrap_throw();
        let url = req.url();

        let mut data = Vec::new();
        if let Some(val) = req.body() {
            data = readable_stream_to_bytes(val).await.unwrap_throw();
        }

        let headers = js_headers_to_reqwest_headers(JsValue::from(req.headers())).unwrap_throw();
        let resp = client
            .request(method, url)
            .headers(headers)
            .body(data)
            .send()
            .await
            .unwrap_throw();

        return construct_js_response(resp).await;
    }

    // Using the resource URL and options object to fetch the resource
    if let Some(options) = options {
        let method = match options.get_method() {
            Some(val) => Method::from_str(&val.trim().to_uppercase()).unwrap_throw(),
            None => Method::GET,
        };

        let headers = js_headers_to_reqwest_headers(options.get_headers()).unwrap_throw();

        // let request_builder = client.request(method, url.clone());
        let resp = client
            .request(method, url)
            .headers(headers)
            .send()
            .await
            .unwrap_throw();

        return construct_js_response(resp).await;
    }

    // using only the URL to fetch the resource, with assumed GET method
    match client.get(url).send().await {
        Ok(resp) => construct_js_response(resp).await,
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
        log(&format!("Response Headers: {:?}", resp.headers()));

        resp_init.set_headers(&js_headers);
    }

    let mut body = resp.bytes().await.unwrap_throw().to_vec();

    log(&format!("Body {:?}", String::from_utf8_lossy(&body)));
    match web_sys::Response::new_with_opt_u8_array_and_init(Some(&mut body), &resp_init) {
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
fn retrieve_resource_url(resource: &JsValue) -> String {
    if resource.is_string() {
        // If the resource is a JsString, we assume it's a URL.
        return resource.as_string().unwrap_throw();
    }

    if resource.is_instance_of::<web_sys::Url>() {
        // If the resource is a URL object, we return it stringified.
        return String::from(
            resource
                .dyn_ref::<web_sys::Url>()
                .unwrap_throw()
                .to_string(),
        );
    }

    if resource.is_instance_of::<web_sys::Request>() {
        // If the resource is a Request object, we return its URL.
        return resource.dyn_ref::<web_sys::Request>().unwrap_throw().url();
    }

    panic!(
        "Invalid resource type for fetch. Expected a string, URL object, or Request object. {:?}",
        resource.js_typeof()
    );
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
            log(&format!("Stream read completed with {} bytes", data.len()));
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

fn js_headers_to_reqwest_headers(
    js_headers: JsValue,
) -> Result<reqwest::header::HeaderMap<HeaderValue>, JsValue> {
    if js_headers.is_null() || js_headers.is_undefined() {
        return Ok(reqwest::header::HeaderMap::new());
    }

    let headers = js_headers
        .dyn_into::<web_sys::Headers>()
        .map_err(|_| JsValue::from_str("Failed to convert JsValue to Headers"))?;

    let mut reqwest_headers = reqwest::header::HeaderMap::new();
    for entry in headers.entries() {
        // [key, value] item array
        let key_value_entry = js_sys::Array::from(&entry?);
        let key = key_value_entry.get(0);
        let value = key_value_entry.get(1);

        // Convert the key and value to reqwest's HeaderName and HeaderValue
        let header_name = reqwest::header::HeaderName::from_str(&key.as_string().unwrap_throw())
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

    Ok(reqwest_headers)
}
