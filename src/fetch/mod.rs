use serde::{Deserialize, Serialize};

mod fetch_api;
mod formdata;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct WasmEncryptedMessage {
    nonce: Vec<u8>,
    data: Vec<u8>,
}
