use bytes::Bytes;
use ntor::common::NTorParty;
use crate::init_tunnel::InitTunnelResult;
use wasm_bindgen::prelude::*;
use crate::types::WasmEncryptedMessage;

/// Represents the current state of the network connection for a service provider.
#[derive(Debug)]
pub(crate) enum NetworkState {
    /// The network is currently being established.
    CONNECTING,
    /// The network is open and ready for use.
    OPEN(NetworkStateOpen),
    /// An error occurred while trying to establish the network connection.
    ERRORED(JsValue),
}

/// This is the state of the network connection for a service provider when it has
/// completed key exchange and is ready to be used.
#[derive(Debug)]
pub(crate) struct NetworkStateOpen {
    pub http_client: reqwest::Client,
    pub init_tunnel_result: InitTunnelResult,
    pub forward_proxy_url: String,
}

// This enum is used to represent the response from the network state.
pub enum NetworkStateResponse {
    // This is an error in response to the unexpected response from the proxy server.
    ProxyError(JsValue),
    // This is a successful response from the proxy server.
    ProviderResponse(web_sys::Response),
    // This is an indicator that we are reinitializing the connection
    Reinitialize,
}

impl NetworkStateOpen {
    pub fn ntor_encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, JsValue> {
        let (nonce, encrypted) = self.init_tunnel_result
            .client
            .wasm_encrypt(data)
            .map_err(|e| {
                JsValue::from_str(&format!("Failed to encrypt data: {}", e))
            })?;

        let msg = serde_json::to_vec(&WasmEncryptedMessage {
            nonce: nonce.to_vec(),
            data: encrypted,
        }).map_err(|e| {
            JsValue::from_str(&format!("Failed to serialize encrypted message: {}", e))
        })?;

        Ok(msg)
    }

    pub fn ntor_decrypt(&self, data: &Bytes) -> Result<Vec<u8>, JsValue> {
        let encrypted_data =
            serde_json::from_slice::<WasmEncryptedMessage>(&data).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to deserialize EncryptedMessage body: {}",
                    e
                ))
            })?;

        let decrypted_response = self
            .init_tunnel_result
            .client
            .wasm_decrypt(encrypted_data.nonce, encrypted_data.data)
            .map_err(|e| JsValue::from_str(&format!("Failed to decrypt data: {}", e)))?;

        Ok(decrypted_response)
    }

    pub fn int_rp_jwt(&self) -> String {
        self.init_tunnel_result.int_rp_jwt.clone()
    }

    pub fn int_fp_jwt(&self) -> String {
        self.init_tunnel_result.int_fp_jwt.clone()
    }
}
