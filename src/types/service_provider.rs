use wasm_bindgen::prelude::wasm_bindgen;

/// Represents a service provider that can be used to request for resources.
#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ServiceProvider {
    pub url: String,
    _options: Option<js_sys::Object>, // for now, options is just any object including empty
}

#[wasm_bindgen]
impl ServiceProvider {
    pub fn new(url: String, _options: Option<js_sys::Object>) -> Self {
        ServiceProvider { url, _options }
    }
}
