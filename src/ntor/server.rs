use crate::ntor::common::{
    generate_private_public_key_pair,
    InitSessionMessage,
    InitSessionResponse,
    PrivatePublicKeyPair,
    Certificate
};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;
use x25519_dalek::PublicKey;

#[wasm_bindgen]
pub struct Server {
    static_key_pair: PrivatePublicKeyPair,
    ephemeral_key_pair: PrivatePublicKeyPair,
    server_id: String,
    shared_secret: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl Server {

    #[wasm_bindgen(constructor)]
    pub fn new(server_id: String) -> Self {
        // In the future, implementations of static and ephemeral key pair generation should differ.
        Self{
            ephemeral_key_pair: PrivatePublicKeyPair {
                private_key: None,
                public_key: PublicKey::from([0; 32]),
            },
            server_id,
            shared_secret: None,
            static_key_pair: generate_private_public_key_pair(),
        }
    }

    #[wasm_bindgen]
    pub fn get_certificate(&self) -> Certificate {
        // Upon implementation and deployment, it's the Service Provider that will create and then upload a certificate to the Layer8 Authentication Server. Likely, Layer8 will also provide the necessary functions to create one for the client.
        crate::ntor::common::Certificate {
            public_key: self.static_key_pair.public_key,
            server_id: self.server_id.clone(),
        }
    }

    #[wasm_bindgen]
    pub fn accept_init_session_request(&mut self, init_msg: &InitSessionMessage) -> InitSessionResponse {
        println!("Server:");

        // generate session-specific ephemeral key pair
        self.ephemeral_key_pair = generate_private_public_key_pair();

        let mut buffer: Vec<u8> = Vec::new();
        // client_ephemeral_public^server_ephemeral_private (X^y),
        let taken_private_key = self.ephemeral_key_pair.private_key.take().unwrap();
        let mut ecdh_results_1 = taken_private_key.diffie_hellman(&init_msg.client_ephemeral_public_key).to_bytes().to_vec();
        println!("[Debug] ECDH result 1: {:?}", ecdh_results_1);
        buffer.append(&mut ecdh_results_1);

        // client_ephemeral_public^server_static_private (X^b),
        let taken_private_key = self.static_key_pair.private_key.take().unwrap();
        let mut ecdh_results_2 = taken_private_key.diffie_hellman(&init_msg.client_ephemeral_public_key).to_bytes().to_vec();
        println!("[Debug] ECDH result 2: {:?}", ecdh_results_2);
        buffer.append(&mut ecdh_results_2);

        // server id
        buffer.append(&mut self.server_id.as_bytes().to_vec());

        // client_ephemeral_public (X)
        buffer.append(&mut init_msg.client_ephemeral_public_key.to_bytes().to_vec());

        // server_ephemeral_public (Y)
        buffer.append(&mut self.ephemeral_key_pair.public_key.to_bytes().to_vec());

        // "ntor"
        buffer.append(&mut "ntor".as_bytes().to_vec());

        // Instantiate sha256 hash function and compute
        let mut hasher = Sha256::new();
        hasher.update(buffer);
        let sha256_hash = hasher.finalize();
        let sha256_hash: &[u8; 32] = match sha256_hash.as_slice().try_into() {
            Ok(array_ref) => array_ref,
            Err(_) => {
                panic!("Invalid sha256 hash length");
            }
        };

        let secret_key_prime = &sha256_hash[0..16];
        let secret_key = &sha256_hash[16..];
        println!("[Debug] Server secret key prime: {:?}", secret_key_prime);

        // Step 12: Compute HMAC (t_b in the paper):
        let mut hmac_key_buffer: Vec<u8> = Vec::new();
        // server id
        hmac_key_buffer.append(&mut self.server_id.as_bytes().to_vec());
        // server_ephemeral_public_key
        hmac_key_buffer.append(&mut self.ephemeral_key_pair.public_key.to_bytes().to_vec());
        // client_ephemeral_public_key
        hmac_key_buffer.append(&mut init_msg.client_ephemeral_public_key.to_bytes().to_vec());
        // "ntor"
        hmac_key_buffer.append(&mut "ntor".as_bytes().to_vec());
        // "server"
        hmac_key_buffer.append(&mut "server".as_bytes().to_vec());

        let mut hmac_hash = Hmac::<Sha256>::new_from_slice(&hmac_key_buffer).unwrap();
        hmac_hash.update(secret_key_prime);
        let output_hash = hmac_hash.finalize().into_bytes().to_vec();

        self.shared_secret = Some(secret_key.to_vec());

        println!("Shared secret:");
        println!("{:?}\n", secret_key);

        InitSessionResponse {
            server_ephemeral_public_key: self.ephemeral_key_pair.public_key,
            t_hash: output_hash,
        }
    }
}