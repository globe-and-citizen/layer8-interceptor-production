use wasm_bindgen::JsValue;
use web_sys::console;

pub fn print_web_sys_header(headers: web_sys::Headers, msg: &str) -> Result<(), JsValue> {
    let headers_js = JsValue::from(headers.clone());
    print_js_headers(&headers_js, msg)
}

pub fn print_js_headers(headers: &JsValue, msg: &str) -> Result<(), JsValue> {
    // Print individual header pairs
    if let Some(iter) = js_sys::try_iter(&headers)? {
        for item in iter {
            let pair = item?;
            let arr = js_sys::Array::from(&pair);
            let key = arr.get(0).as_string().unwrap_or_default();
            let value = arr.get(1).as_string().unwrap_or_default();
            console::log_1(&format!("{msg}: {}: {}", key, value).into());
        }
    }
    Ok(())
}
