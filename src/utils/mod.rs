use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use std::collections::HashMap;
use wasm_streams::ReadableStream;
use web_sys::console;
use js_sys::Uint8Array;
use crate::storage::InMemoryCache;
use crate::types::request::L8BodyType;

pub(crate) async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, _: js_sys::Function| {
        _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay);
    };

    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

pub(crate) fn get_base_url(url: &str) -> Result<String, JsValue> {
    let url =
        url::Url::parse(url).map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    // get without query or path fragments
    let mut base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
    if let Some(port) = url.port() {
        base_url = format!("{}:{}", base_url, port);
    }

    Ok(base_url)
}

pub(crate) fn get_uri(url: &str) -> Result<String, JsValue> {
    let url_object = url::Url::parse(&url)
        .map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    let mut uri = url_object.path().to_string();
    if let Some(query) = url_object.query() {
        uri.push_str(&format!("?{}", query));
    }
    Ok(uri)
}

// returns the URL of the resource to be fetched
pub fn retrieve_resource_url(resource: &JsValue) -> Result<String, JsValue> {
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
pub fn headers_to_reqwest_headers(
    js_headers: JsValue,
) -> Result<HashMap<String, serde_json::Value>, JsValue>
{
    let dev_flag = InMemoryCache::get_dev_flag();

    // If the headers are undefined or null, we return an empty HeaderMap
    if js_headers.is_null() || js_headers.is_undefined() {
        return Ok(HashMap::new());
    }

    // We first check if the headers are an instance of web_sys::Headers
    if let Some(headers) = js_headers.dyn_ref::<web_sys::Headers>() {
        return js_headers_to_reqwest_headers(headers);
    }

    if dev_flag {
        console::log_1(&format!("Headers typeof: {:?}", js_headers.js_typeof()).into());
    }

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
) -> Result<HashMap<String, serde_json::Value>, JsValue>
{
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

// Converts a Javascript request body to a reqwest Body type.
// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_a_body>
pub async fn parse_js_request_body(body: JsValue) -> Result<L8BodyType, JsValue> {
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
        return Ok(L8BodyType::Bytes(
            body.as_string()
                .expect_throw("Expected body to be a string")
                .into_bytes(),
        ));
    }

    // ArrayBuffer
    if let Some(val) = body.dyn_ref::<js_sys::ArrayBuffer>() {
        let uint8_array = js_sys::Uint8Array::new(val);
        return Ok(L8BodyType::Bytes(uint8_array.to_vec()));
    }

    // *TypedArray, todo

    // DataView
    if let Some(val) = body.dyn_ref::<js_sys::DataView>() {
        let uint8_array = js_sys::Uint8Array::new(&val.buffer());
        return Ok(L8BodyType::Bytes(uint8_array.to_vec()));
    }

    // Blob
    if let Some(val) = body.dyn_ref::<web_sys::Blob>() {
        let readable_stream = val.stream();
        let body = ReadableStream::from_raw(readable_stream);
        return Ok(L8BodyType::Stream(body));
    }

    // File
    if body.is_instance_of::<web_sys::File>() {
        let val = body
            .dyn_into::<web_sys::File>()
            .expect_throw("Expected body to be a web_sys::File");
        let readable_stream = val.stream();
        let body = ReadableStream::from_raw(readable_stream);
        return Ok(L8BodyType::Stream(body));
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
        return Ok(L8BodyType::Params(params));
    }

    // FormData
    if body.is_instance_of::<web_sys::FormData>() {
        let val = body
            .dyn_into::<web_sys::FormData>()
            .expect_throw("Expected body to be a web_sys::FormData");

        return Ok(L8BodyType::FormData(val));
    }

    // ReadableStream
    if body.is_instance_of::<web_sys::ReadableStream>() {
        let readable_stream = body
            .dyn_into::<web_sys::ReadableStream>()
            .expect_throw("Expected body to be a web_sys::ReadableStream");
        let body = ReadableStream::from_raw(readable_stream);
        return Ok(L8BodyType::Stream(body));
    }

    // Other objects are converted to strings using their toString() method.
    if let Some(val) = body.dyn_ref::<js_sys::Object>() {
        let val = js_sys::Object::to_string(val)
            .as_string()
            .expect_throw("Expected body to be a string representation of an object");
        return Ok(L8BodyType::Bytes(val.into_bytes()));
    }

    Err(JsValue::from_str(
        "Invalid body type for fetch. Expected a string, ArrayBuffer, TypedArray, DataView, Blob, File, URLSearchParams, FormData, or ReadableStream.",
    ))
}

// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read#example_1_-_simple_example>
/// Converts a ReadableStream to a byte vector by reading all chunks from the stream.
/// This function reads the stream until it is done and accumulates the data into a Vec<u8>.
///
pub async fn readable_stream_to_bytes(stream: web_sys::ReadableStream) -> Result<Vec<u8>, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();
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
            if dev_flag {
                console::log_1(&format!("Stream read completed with {} bytes", data.len()).into());
            }

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

/// Converts an instance of `web_sys::FormData` to a `Uint8Array`
///
///  // Sample output:
/// --AaB03x
/// content-disposition: form-data; name="field1"
/// content-type: text/plain;charset=windows-1250
/// content-transfer-encoding: quoted-printable
///
/// Joe owes =80100.
/// --AaB03x
///
/// Ref: <https://github.com/nodejs/undici/blob/e39a6324c4474c6614cac98b8668e3d036aa6b18/lib/fetch/body.js#L31>
pub async fn parse_form_data_to_array(
    form: web_sys::FormData,
    boundary: &str,
) -> Result<Vec<u8>, JsValue>
{
    let prefix = format!("--{}\r\nContent-Disposition: form-data", boundary);
    let mut blob_parts = Vec::new();
    let rn = Uint8Array::from(&[13, 10][..]); // '\r\n'

    // for (const [name, value] of inputFormData)
    for entry in form.entries() {
        let val = js_sys::Array::from(&entry?);
        let key = val.get(0).as_string().ok_or_else(|| {
            JsValue::from_str("Expected first element of FormData entry to be a string")
        })?;
        let value = val.get(1);

        // form field values
        if let Some(value) = value.as_string() {
            // String value
            let chunk_str = format!(
                "{}; name=\"{}\"\r\n\r\n{}\r\n",
                prefix,
                escape(&normalize_linefeeds(&key)),
                normalize_linefeeds(&value)
            );

            let chunk = chunk_str.as_bytes();
            blob_parts.extend_from_slice(chunk);

            continue;
        }

        // getting the name before casting to Blob
        let filename = js_sys::Reflect::get(&value, &"name".into())
            .map_err(|e| {
                JsValue::from_str(&format!(
                    "Expected to retrieve name property before casting to Blob: {}",
                    e.as_string().unwrap_or_else(|| "unknown error".to_string())
                ))
            })?
            .as_string()
            .unwrap_or_default();

        let blob = value.dyn_into::<web_sys::Blob>().map_err(|_| {
            JsValue::from_str("Expected second type cast of FormData entry to be a Blob")
        })?;

        // Blob values
        let file_contents = wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await?;
        let file_contents: Uint8Array = Uint8Array::new(&file_contents);

        let content_type = blob.type_();

        let chunk_str = format!(
            "{}; name=\"{}\"{}Content-Type: {}\r\n\r\n",
            prefix,
            escape(&normalize_linefeeds(&key)),
            if !filename.is_empty() {
                format!("; filename=\"{}\"\r\n", escape(&filename))
            } else {
                "\r\n".to_string()
            },
            content_type
        );

        let chunk = chunk_str.as_bytes();
        blob_parts.extend_from_slice(chunk);
        blob_parts.extend_from_slice(&file_contents.to_vec());
        blob_parts.extend_from_slice(&rn.to_vec()); // \r\n
    }

    let chunk = format!("--{}--", boundary);
    blob_parts.extend_from_slice(chunk.as_bytes());

    Ok(blob_parts)
}

fn escape(str: &str) -> String {
    str.replace('\n', "%0A")
        .replace('\r', "%0D")
        .replace('"', "%22")
}

fn normalize_linefeeds(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}
