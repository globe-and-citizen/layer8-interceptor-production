use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Deserialize, Serialize};

mod client;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WasmEncryptedMessage {
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}
