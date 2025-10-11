use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::{JsValue, UnwrapThrowExt, throw_str};
use web_sys::ResponseInit;

#[derive(Deserialize, Debug)]
pub struct L8ResponseObject {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, serde_json::Value>,
    pub body: Vec<u8>,

    /* Below fields are present but not used because ResponseInit does not support */
    #[allow(dead_code)]
    pub ok: bool,
    #[allow(dead_code)]
    pub url: String,
    #[allow(dead_code)]
    pub redirected: bool,
    /* Other fields are ignored because rust and wasm do not support */
}

impl L8ResponseObject {
    pub fn reconstruct_js_response(&self) -> Result<web_sys::Response, JsValue> {
        let resp_init = ResponseInit::new();
        resp_init.set_status(self.status);
        resp_init.set_status_text(&self.status_text);

        let js_headers = web_sys::Headers::new().expect_throw("Failed to create Headers object");
        for (key, value) in self.headers.clone() {
            let value = serde_json::to_string(&value).expect_throw(
                "we expect the header value to be serializable as a JSON string at compile time",
            );

            js_headers
                .append(&key, &value)
                .expect_throw("Failed to append header to Headers object");
        }
        resp_init.set_headers(&js_headers);

        let array = js_sys::Uint8Array::new_with_length(self.body.len() as u32);
        array.copy_from(&self.body);

        match web_sys::Response::new_with_opt_js_u8_array_and_init(Some(&array), &resp_init) {
            Ok(response) => Ok(response),
            Err(err) => {
                throw_str(&format!(
                    "Failed to construct JS Response: {:?}",
                    err.as_string()
                ));
            }
        }
    }
}
