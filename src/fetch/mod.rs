use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

mod fetch_api;
mod formdata;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WasmEncryptedMessage {
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}
