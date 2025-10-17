use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_streams::ReadableStream;

pub enum L8BodyType {
    Bytes(Vec<u8>),
    Stream(ReadableStream),
    Params(HashMap<String, String>),
    FormData(web_sys::FormData),
    #[allow(dead_code)]
    File(web_sys::File),
}

impl L8BodyType {
    /// Converts a Javascript request body to a reqwest Body type.
    /// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#setting_a_body>
    /// Supported types:
    /// - a string
    /// - ArrayBuffer
    /// - TypedArray (todo)
    /// - DataView
    /// - Blob
    /// - File
    /// - URLSearchParams
    /// - FormData
    /// - ReadableStream
    pub async fn from_jsvalue(body: JsValue) -> Result<Self, JsValue> {
        // a string
        if body.is_string() {
            return Ok(L8BodyType::Bytes(
                body.as_string()
                    .expect_throw("Expected body to be a string")
                    .into_bytes(),
            ));
        }

        // ArrayBuffer
        if let Some(val) = body.dyn_ref::<js_sys::ArrayBuffer>() {
            let uint8_array = js_sys::Uint8Array::new(val);
            return Ok(L8BodyType::Bytes(uint8_array.to_vec()));
        }

        // *TypedArray, todo

        // DataView
        if let Some(val) = body.dyn_ref::<js_sys::DataView>() {
            let uint8_array = js_sys::Uint8Array::new(&val.buffer());
            return Ok(L8BodyType::Bytes(uint8_array.to_vec()));
        }

        // Blob
        if let Some(val) = body.dyn_ref::<web_sys::Blob>() {
            let readable_stream = val.stream();
            let body = ReadableStream::from_raw(readable_stream);
            return Ok(L8BodyType::Stream(body));
        }

        // File
        if body.is_instance_of::<web_sys::File>() {
            let val = body
                .dyn_into::<web_sys::File>()
                .expect_throw("Expected body to be a web_sys::File");
            let readable_stream = val.stream();
            let body = ReadableStream::from_raw(readable_stream);
            return Ok(L8BodyType::Stream(body));
        }

        // URLSearchParams
        if let Some(val) = body.dyn_ref::<web_sys::UrlSearchParams>() {
            let mut params = HashMap::new();
            for entry in val.entries() {
                // [key, value] item array
                let key_value_entry = js_sys::Array::from(
                    &entry.expect_throw("Expected entry to be a valid URLSearchParams entry"),
                );
                let key = key_value_entry
                    .get(0)
                    .as_string()
                    .expect_throw("Expected key in URLSearchParams key entry to be a string");
                let value = key_value_entry
                    .get(1)
                    .as_string()
                    .expect_throw("Expected value in URLSearchParams value entry to be a string");
                params.insert(key, value);
            }
            return Ok(L8BodyType::Params(params));
        }

        // FormData
        if body.is_instance_of::<web_sys::FormData>() {
            let val = body
                .dyn_into::<web_sys::FormData>()
                .expect_throw("Expected body to be a web_sys::FormData");

            return Ok(L8BodyType::FormData(val));
        }

        // ReadableStream
        if body.is_instance_of::<web_sys::ReadableStream>() {
            let readable_stream = body
                .dyn_into::<web_sys::ReadableStream>()
                .expect_throw("Expected body to be a web_sys::ReadableStream");
            let body = ReadableStream::from_raw(readable_stream);
            return Ok(L8BodyType::Stream(body));
        }

        // Other objects are converted to strings using their toString() method.
        if let Some(val) = body.dyn_ref::<js_sys::Object>() {
            let val = js_sys::Object::to_string(val)
                .as_string()
                .expect_throw("Expected body to be a string representation of an object");
            return Ok(L8BodyType::Bytes(val.into_bytes()));
        }

        Err(JsValue::from_str(
            "Invalid body type for fetch. Expected a string, ArrayBuffer, TypedArray, \
            DataView, Blob, File, URLSearchParams, FormData, or ReadableStream.",
        ))
    }
}
