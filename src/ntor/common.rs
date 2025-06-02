use getrandom;
use wasm_bindgen::prelude::*;
use x25519_dalek::{PublicKey, StaticSecret};

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
    getrandom::fill(&mut buf).expect("generate random failed");
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

#[wasm_bindgen(getter_with_clone)]
// In the paper, the outgoing message is ("ntor", B_id, client_ephemeral_public_key).
pub struct InitSessionMessage {
    pub(crate) client_ephemeral_public_key: PublicKey,
}

#[wasm_bindgen(getter_with_clone)]
// In the paper, the return message is ("ntor", server_ephemeral_public_key, t_b_hash).
pub struct InitSessionResponse {
    pub(crate) server_ephemeral_public_key: PublicKey,
    pub t_hash: Vec<u8>
}



