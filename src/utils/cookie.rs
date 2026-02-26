use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlDocument};

#[wasm_bindgen]
pub fn set_cookies(cookie: &str) -> Result<(), JsValue> {
    let document = window()
        .ok_or(JsValue::NULL)?
        .document()
        .ok_or(JsValue::NULL)?;

    let html_doc = document.dyn_into::<HtmlDocument>()?;

    html_doc.set_cookie(cookie)
}

#[wasm_bindgen]
pub fn get_cookies() -> Result<String, JsValue> {
    let document = window()
        .ok_or(JsValue::NULL)?
        .document()
        .ok_or(JsValue::NULL)?;

    let html_doc = document.dyn_into::<HtmlDocument>()?;

    html_doc.cookie()
}