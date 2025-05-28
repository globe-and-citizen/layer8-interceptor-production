use wasm_bindgen::prelude::*;
use web_sys::{console, Blob, DomException};

#[wasm_bindgen]
pub struct CacheHandler {
}

#[wasm_bindgen]
impl CacheHandler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CacheHandler {
        CacheHandler {}
    }

    #[wasm_bindgen]
    pub async fn get_images(&self, keys: &[String]) -> Result<JsValue, JsValue> {
        for i in 0..keys.len() {
            get_image(keys[i].clone()).await?;
        }
        let promise = js_sys::Promise::resolve(&"All images fetched".into());

        JsValue::from("Cached value")
    }

    #[wasm_bindgen]
    pub fn set_images(&self, key: String, value: web_sys::Blob) {
        // Simulate setting a cache value
    }
}

