use std::collections::HashMap;

pub mod network_state;
pub mod http_call_indirection;

pub enum Body {
    Bytes(Vec<u8>),
    Stream(wasm_streams::ReadableStream),
    Params(HashMap<String, String>),
    FormData(web_sys::FormData),
    #[allow(dead_code)]
    File(web_sys::File),
}
