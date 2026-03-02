use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::console;
use crate::storage::InMemoryCache;

// Ref <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_headers>
// we expect the headers to be either Headers or an Object
pub fn headers_to_reqwest_headers(
    js_headers: JsValue,
) -> Result<HashMap<String, serde_json::Value>, JsValue> {
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

pub fn hashmap_to_js_headers(
    headers: &HashMap<String, serde_json::Value>,
) -> Result<web_sys::Headers, JsValue> {
    let js_headers = web_sys::Headers::new().expect_throw("Failed to create Headers object");
    for (key, value) in headers.clone() {
        let value = serde_json::to_string(&value).expect_throw(
            "we expect the header value to be serializable as a JSON string at compile time",
        );

        js_headers
            .append(&key, &value)
            .expect_throw("Failed to append header to Headers object");

        // console::log_1(&format!("js header: {}: {}", key, value).into());
    }
    Ok(js_headers)
}