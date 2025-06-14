use aes_gcm::{Aes256Gcm, Key, Nonce};
use getrandom;
use wasm_bindgen::prelude::*;
use x25519_dalek::{PublicKey, StaticSecret};
use aes_gcm::aead::{Aead, KeyInit};
use std::convert::TryInto;
use serde_wasm_bindgen;

#[wasm_bindgen(getter_with_clone)]
pub struct PrivatePublicKeyPair {
    // In the future, type StaticSecret should be reserved for the server's static and the EphemeralSecret reserved for the ephemeral private key.
    // However, as a quirk of the nTOR protocol, we also need to use StaticSecret for the client's ephemeral private key hence why it is adopted here.
    pub(crate) private_key: Option<StaticSecret>,
    pub(crate) public_key: PublicKey,
}

#[wasm_bindgen]
pub fn generate_private_public_key_pair() -> PrivatePublicKeyPair {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).expect("generate random failed");
    let private_key = StaticSecret::from(buf);
    let public_key = PublicKey::from(&private_key);

    PrivatePublicKeyPair {
        private_key: Some(private_key),
        public_key,
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct Certificate {
    pub(crate) public_key: PublicKey,
    pub server_id: String,
}

#[wasm_bindgen]
impl Certificate {
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: Vec<u8>, server_id: String) -> Self {
        let pub_key = TryInto::<[u8; 32]>::try_into(public_key).unwrap_throw();
        Certificate {
            public_key: PublicKey::from(pub_key),
            server_id
        }
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> JsValue {
        let data = serde_json::json!({
            "public_key": self.public_key.to_bytes(),
            "server_id": self.server_id
        });
        serde_wasm_bindgen::to_value(&data).unwrap_throw()
    }

    #[wasm_bindgen]
    pub fn public_key(&self) -> Vec<u8> {
        self.public_key.to_bytes().to_vec()
    }
}

#[wasm_bindgen(getter_with_clone)]
// In the paper, the outgoing message is ("ntor", B_id, client_ephemeral_public_key).
pub struct InitSessionMessage {
    pub(crate) client_ephemeral_public_key: PublicKey,
}

#[wasm_bindgen]
impl InitSessionMessage {
    #[wasm_bindgen]
    pub fn to_json(&self) -> JsValue {
        let data = serde_json::json!({
            "client_ephemeral_public_key": self.client_ephemeral_public_key.to_bytes(),
        });
        serde_wasm_bindgen::to_value(&data).unwrap_throw()
    }

    #[wasm_bindgen]
    pub fn public_key(&self) -> Vec<u8> {
        self.client_ephemeral_public_key.to_bytes().to_vec()
    }
}

#[wasm_bindgen(getter_with_clone)]
// In the paper, the return message is ("ntor", server_ephemeral_public_key, t_b_hash).
pub struct InitSessionResponse {
    pub(crate) server_ephemeral_public_key: PublicKey,
    pub t_hash: Vec<u8>,
}

#[wasm_bindgen]
impl InitSessionResponse {
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: Vec<u8>, t_hash: Vec<u8>) -> Self {
        let pub_key = TryInto::<[u8; 32]>::try_into(public_key).unwrap_throw();
        return InitSessionResponse {
            server_ephemeral_public_key: PublicKey::from(pub_key),
            t_hash,
        }
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> JsValue {
        let data = serde_json::json!({
            "server_ephemeral_public_key": self.server_ephemeral_public_key.to_bytes(),
            "t_hash": self.t_hash
        });
        serde_wasm_bindgen::to_value(&data).unwrap_throw()
    }

    #[wasm_bindgen]
    pub fn public_key(&self) -> Vec<u8> {
        self.server_ephemeral_public_key.to_bytes().to_vec()
    }
}

pub(crate) fn encrypt(key_bytes: Vec<u8>, data: Vec<u8>) -> Result<([u8; 12], Vec<u8>), &'static str> {
    if key_bytes.len() != 32 {
        return Err("Invalid key length for AES-256");
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes).map_err(|_| "Random generation failed")?;
    let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|_| "Encryption failed")?;

    Ok((nonce_bytes, ciphertext))
}

pub(crate) fn decrypt(nonce_bytes: [u8; 12], key: Vec<u8>, ciphertext: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    return match TryInto::<[u8; 32]>::try_into(key) {
        Ok(key_bytes) => {
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let decrypted_data = cipher
                .decrypt(nonce, ciphertext.as_ref())
                .map_err(|_| "Decryption failed")?;

            Ok(decrypted_data)
        }
        Err(_) => {
            Err("Invalid key")
        }
    }
}
