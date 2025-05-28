use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;
use web_sys::{console, window, Blob, DomException};
use serde_wasm_bindgen;


#[wasm_bindgen(getter_with_clone)]
pub struct BackendConfig {
    pub base_url: String,
    pub get_image_path: String,
    pub get_poem_path: String,
    pub get_profile_path: String,
}

#[wasm_bindgen]
impl BackendConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BackendConfig {
        console::log_1(&format!("BackendConfig created with base_url").into());
        BackendConfig {
            base_url: "http://localhost:6191".to_string(),
            get_image_path: "/images?id=".to_string(),
            get_poem_path: "/poems?id=".to_string(),
            get_profile_path: "/profile".to_string(),
        }
    }
}


#[wasm_bindgen]
pub struct Backend {
    config: BackendConfig
}

#[wasm_bindgen]
impl Backend {
    #[wasm_bindgen(constructor)]
    pub fn new(config: BackendConfig) -> Backend {
        console::log_1(&format!("Backend created with address: {}", config.base_url).into());
        Backend {config}
    }

    async fn get(&self, url: &String, headers: HashMap<String, String>) -> Result<JsValue, JsValue> {
        console::log_1(&format!("GET request to: {}", url).into());

        let mut request = reqwest::Client::new()
            .get(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

        let body_bytes = response.bytes().await?;

        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        console::log_1(&format!("Response body: {:?}", body).into());
        Ok(serde_wasm_bindgen::to_value(&body).unwrap())
    }


    #[wasm_bindgen]
    pub async fn get_images(&self, id: String) -> Result<JsValue, JsValue> {
        console::log_1(&format!("Fetching images with id: {}", id).into());

        let url = self.config.base_url.clone() + format!("{}{}", &self.config.get_image_path, id).as_str();

        let mut headers = HashMap::new();
        headers.insert("Authentication".to_string(), "token_for_tester".to_string());

        self.get(&url, headers).await
    }
}

