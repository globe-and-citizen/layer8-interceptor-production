use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Deserialize, Serialize};

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

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WasmEncryptedMessage {
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}
