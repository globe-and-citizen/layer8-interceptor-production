use wasm_bindgen::prelude::*;
use web_sys::console;
use ntor::common::{NTorParty};
use serde::{Serialize, Deserialize};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmNTorClient {
    pub(crate) client: ntor::client::NTorClient,
}

#[wasm_bindgen]
impl WasmNTorClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmNTorClient {
        WasmNTorClient {
            client: ntor::client::NTorClient::new()
        }
    }

    #[wasm_bindgen]
    pub fn encrypt(&self, data: Vec<u8>) -> Result<EncryptedMessage, JsError> {
        console::debug_1(&format!("Encrypt input: {:?}", data).into());
        return match self.client.wasm_encrypt(data) {
            Ok((nonce, encrypted)) => {
                console::debug_1(&format!("Encrypted nonce: {:?}", nonce).into());
                console::debug_1(&format!("Encrypted data: {:?}", encrypted).into());
                Ok(EncryptedMessage {
                    nonce: nonce.to_vec(),
                    data: encrypted,
                })
            }
            Err(err) => Err(JsError::new(err))
        };
    }

    #[wasm_bindgen]
    pub fn decrypt(&self, nonce: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, JsError> {
        console::debug_1(&format!("Decrypt input: {:?}, nonce: {:?}", data, nonce).into());
        return match self.client.wasm_decrypt(nonce, data) {
            Ok(decrypted) => {
                console::debug_1(&format!("Decrypted data: {:?}", decrypted).into());
                Ok(decrypted)
            }
            Err(err) => Err(JsError::new(err))
        };
    }

    pub fn tmp_encrypt(&self, data: Vec<u8>) -> Result<EncryptedMessage, JsError> {
        Ok(EncryptedMessage {
            nonce: vec![0; 12], // Placeholder nonce
            data: data,
        })
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EncryptedMessage {
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}

