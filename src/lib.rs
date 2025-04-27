use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;
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

// try to throw an error
#[wasm_bindgen]
pub async fn fetch(url: String, config: JsValue) -> Result<JsValue, JsValue> {
    console::log_1(&format!("Fetching URL: {}", url).into());
    console::log_1(&format!("Fetching with config: {:?}", config).into());
    let promise = js_sys::Promise::resolve(&url.into());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}

#[wasm_bindgen]
pub async fn get_static(uri: String) -> Result<JsValue, JsValue> {
    console::log_1(&format!("Getting static resource from: {}", uri).into());
    let promise = js_sys::Promise::reject(&"Check promise result in error".into());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}