use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use wasm_bindgen::prelude::*;
use js_sys;
use web_sys::console;

#[wasm_bindgen]
pub fn test_wasm() -> bool {
    console::log_1(&"Hello from test_wasm!".into());
    true
}

#[wasm_bindgen]
pub fn persistence_check() -> i32 {
    console::log_1(&"Hello from persistence_check!".into());
    1
}

#[wasm_bindgen]
pub async fn check_encrypted_tunnel() -> Result<JsValue, JsValue> {
    console::log_1(&"Hello from check_encrypted_tunnel!".into());
    let promise = js_sys::Promise::resolve(&42.into());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}

#[wasm_bindgen]
pub async fn init_encrypted_tunnel(config: JsValue) -> Result<JsValue, JsValue> {
    console::log_1(&"Hello from init_encrypted_tunnel!".into());
    let promise = js_sys::Promise::resolve(&config);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}

// #[wasm_bindgen]
// pub async fn fetch(url: String, config: JsValue) -> Result<JsValue, JsValue> {
//     console::log_1(&format!("Fetching URL: {}", url).into());
//     console::log_1(&format!("Fetching with config: {:?}", config).into());
//     let promise = js_sys::Promise::resolve(&url.into());
//     let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
//     Ok(result)
// }

// try to throw an error
#[wasm_bindgen]
pub async fn get_static(uri: String) -> Result<JsValue, JsValue> {
    console::log_1(&format!("Getting static resource from: {}", uri).into());
    let promise = js_sys::Promise::reject(&"Check promise result in error".into());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}

pub fn js_map_to_headers(headers: &js_sys::Map) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    headers.for_each(&mut |value, key| {
        if let (Some(header_name), Some(val)) = (key.as_string(), value.as_string()) {
            if let (Ok(name), Ok(value)) = (header_name.parse::<HeaderName>(), val.parse::<HeaderValue>()) {
                header_map.insert(name, value);
            }
        }
    });
    header_map
}

pub fn map_serialize(map: &js_sys::Map) -> String {
    let array = js_sys::Array::new();
    map.for_each(&mut |value, key| {
        if let (Some(key_str), Some(value_str)) = (key.as_string(), value.as_string()) {
            let pair = js_sys::Array::of2(&JsValue::from_str(&key_str), &JsValue::from_str(&value_str));
            array.push(&pair);
        }
    });
    js_sys::JSON::stringify(&array).unwrap_or_else(|_| "[]".to_string().into()).into()
}

pub fn map_deserialize(json: &str) -> js_sys::Map {
    let array = js_sys::JSON::parse(json)
        .unwrap_or_else(|_| JsValue::from_str("[]"))
        .dyn_into::<js_sys::Array>()
        .unwrap();
    let map = js_sys::Map::new();
    for item in array.iter() {
        if let Some(pair) = item.dyn_into::<js_sys::Array>().ok() {
            if pair.length() == 2 {
                let key = pair.get(0);
                let value = pair.get(1);
                map.set(&key, &value);
            }
        }
    }
    map
}

pub fn jsvalue_to_vec_u8(val: &JsValue) -> Result<Vec<u8>, JsValue> {
    // Convert JsValue to serde_json::Value
    let json_value: serde_json::Value = serde_wasm_bindgen::from_value(val.clone())
        .map_err(|e| JsValue::from_str(&format!("serde_wasm_bindgen error: {}", e)))?;
    // Serialize to JSON string
    let json_str = serde_json::to_string(&json_value)
        .map_err(|e| JsValue::from_str(&format!("serde_json error: {}", e)))?;
    // Convert to bytes
    Ok(json_str.into_bytes())
}

pub fn struct_to_vec<T>(value: &T) -> Result<Vec<u8>, JsValue>
where
    T: serde::Serialize,
{
    serde_json::to_vec(value)
        .map_err(|e| JsValue::from_str(&format!("serde_json error: {}", e)))
}

pub fn vec_to_struct<T>(bytes: Vec<u8>) -> Result<T, JsValue>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(&bytes)
        .map_err(|e| JsValue::from_str(&format!("serde_json error: {}", e)))
}

pub fn vec_to_string(vec: Vec<u8>) -> String {
    String::from_utf8_lossy(&vec).to_string()
}