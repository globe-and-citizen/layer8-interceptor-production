use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::{JsValue, throw_str};
use web_sys::{console, ResponseInit};
use crate::storage::InMemoryCache;
use crate::types::network_state::{NetworkStateOpen, NetworkStateResponse};
use crate::utils;

#[derive(Deserialize, Debug)]
pub struct L8ResponseObject {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, serde_json::Value>,
    pub body: Vec<u8>,

    /* Below fields are present but not used because ResponseInit does not support */
    #[allow(dead_code)]
    pub ok: bool,
    #[allow(dead_code)]
    pub url: String,
    #[allow(dead_code)]
    pub redirected: bool,
    /* Other fields are ignored because rust and wasm do not support */
}

impl L8ResponseObject {
    pub fn reconstruct_js_response(&self) -> Result<web_sys::Response, JsValue> {
        let resp_init = ResponseInit::new();
        resp_init.set_status(self.status);
        resp_init.set_status_text(&self.status_text);

        let js_headers = utils::hashmap_to_js_headers(&self.headers)?;
        resp_init.set_headers(&js_headers);

        let array = js_sys::Uint8Array::new_with_length(self.body.len() as u32);
        array.copy_from(&self.body);

        // we lost Set-Cookie header here
        match web_sys::Response::new_with_opt_js_u8_array_and_init(Some(&array), &resp_init) {
            Ok(response) => Ok(response),
            Err(err) => {
                throw_str(&format!(
                    "Failed to construct JS Response: {:?}",
                    err.as_string()
                ));
            }
        }
    }
}

pub async fn handle_response(
    network_state_open: &NetworkStateOpen,
    reinitialize_attempt: bool,
    response: reqwest::Response,
) -> Result<NetworkStateResponse, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();

    // status >= 400
    if response.status() >= reqwest::StatusCode::BAD_REQUEST {
        if dev_flag {
            console::log_1(
                &format!(
                    "Received error response from the proxy server: {}",
                    response.status()
                )
                    .into(),
            );
        }

        // we can reinitialize the network state
        if reinitialize_attempt {
            return Ok(NetworkStateResponse::Reinitialize);
        }

        return Ok(NetworkStateResponse::ProxyError(JsValue::from_str(
            &format!(
                "Unexpected response from the proxy server: {}; With body: {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No response body".to_string())
            ),
        )));
    }

    let body = &response
        .bytes()
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to read response body: {}", e)))?;

    let decrypted_response = network_state_open.ntor_decrypt(body)?;

    let l8_response = serde_json::from_slice::<L8ResponseObject>(&decrypted_response)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize response: {}", e)))?;

    if dev_flag {
        console::log_1(&format!("Response: {:?}", l8_response).into());
    }

    // convert L8ResponseObject to web_sys::Response
    let js_response = l8_response.reconstruct_js_response()?;
    Ok(NetworkStateResponse::ProviderResponse(js_response))
}
