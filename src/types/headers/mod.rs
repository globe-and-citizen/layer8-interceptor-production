use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use web_sys::console;
use crate::storage::InMemoryCache;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct L8Headers(HashMap<String, serde_json::Value>);

impl L8Headers {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    
    pub fn hashmap(&self) -> &HashMap<String, serde_json::Value> {
        &self.0
    }
    
    pub fn from_hashmap(hm: HashMap<String, serde_json::Value>) -> Self {
        Self(hm)
    }

    /// Read-only access to a header value
    pub fn get(&self, key: &str) -> &serde_json::Value {
        &self.0[key]
    }

    /// Read-only access to a header value as a string slice (helper)
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    /// Controlled insertion method
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.0.insert(key.into(), value.into());
    }

    /// Read-only iterator over entries (prevents direct map mutations)
    pub fn iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> {
        self.0.iter()
    }

    pub(crate) fn to_web_sys(&self) -> Result<web_sys::Headers, JsValue> {
        let js_headers = web_sys::Headers::new().expect_throw("Failed to create Headers object");
        for (key, value) in self.0.clone() {
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
    
    pub fn from_jsvalue(js_headers: JsValue) -> Result<L8Headers, JsValue> {
        let dev_flag = InMemoryCache::get_dev_flag();

        // If the headers are undefined or null, we return an empty HeaderMap
        if js_headers.is_null() || js_headers.is_undefined() {
            return Ok(L8Headers(HashMap::new()));
        }

        // We first check if the headers are an instance of web_sys::Headers
        if let Some(headers) = js_headers.dyn_ref::<web_sys::Headers>() {
            // return crate::utils::headers::js_headers_to_reqwest_headers(headers);
            return Self::from_web_sys_headers(headers)
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
            // return crate::utils::headers::js_headers_to_reqwest_headers(headers);
            return Self::from_web_sys_headers(headers);
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

        Ok(L8Headers(reqwest_headers))
    }
    
    fn from_web_sys_headers(
        headers: &web_sys::Headers,
    ) -> Result<Self, JsValue> {
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

        Ok(L8Headers(reqwest_headers))
    }
    
    pub fn extend(&mut self, other: L8Headers) {
        self.0.extend(other.0);
    }
    
}
