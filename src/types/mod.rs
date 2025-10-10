use serde::{Deserialize, Serialize};

pub mod http_caller;
pub mod network_state;
pub mod request;
mod response;
pub(crate) mod service_provider;

/// this struct will be replaced by the WasmEncryptedMessage struct from ntor repository when available
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WasmEncryptedMessage {
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}
