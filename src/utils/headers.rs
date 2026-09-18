use std::collections::HashMap;
use wasm_bindgen::{JsValue};

pub fn hashmap_to_reqwest_header_map(
    input: &HashMap<String, serde_json::Value>,
) -> Result<reqwest::header::HeaderMap, JsValue> {
    let mut header_map = reqwest::header::HeaderMap::new();
    for (key, value) in input {
        let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| JsValue::from_str(&format!("Invalid header name '{}': {}", key, e)))?;

        let value_str = match &value {
            serde_json::Value::String(s) => s.clone(),
            other => serde_json::to_string(other).map_err(|e| {
                JsValue::from_str(&format!("Failed to serialize header value: {}", e))
            })?,
        };

        let header_value = reqwest::header::HeaderValue::from_str(&value_str).map_err(|e| {
            JsValue::from_str(&format!("Invalid header value for '{}': {}", key, e))
        })?;

        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}
