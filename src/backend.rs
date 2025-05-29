use reqwest::header::HeaderMap;
use wasm_bindgen::prelude::*;
use web_sys::{console};
use serde_wasm_bindgen;


#[wasm_bindgen(getter_with_clone)]
pub struct BackendConfig {
    pub base_url: String,
    pub login: String,
    pub register: String,
    pub get_image_path: String,
    pub get_images_path: String,
    pub get_poem_path: String,
    pub get_poems_path: String,
    pub get_profile_path: String,
}

#[wasm_bindgen]
impl BackendConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BackendConfig {
        console::log_1(&format!("BackendConfig created with base_url").into());
        BackendConfig {
            base_url: "http://localhost:6191".to_string(),
            login: "/login".to_string(),
            register: "/register".to_string(),
            get_image_path: "/images?id={}".to_string(),
            get_images_path: "/images".to_string(),
            get_poem_path: "/poems?id={}".to_string(),
            get_poems_path: "/poems".to_string(),
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
        Backend {config}
    }

    async fn get(&self, url: &String, headers: HeaderMap) -> Result<JsValue, JsValue> {
        let response = reqwest::Client::new()
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

        let body_bytes = response.bytes().await?;
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_throw();
        Ok(serde_wasm_bindgen::to_value(&body).unwrap_throw())
    }

    async fn post(&self, url: &String, headers: HeaderMap, body: serde_json::Value) -> Result<JsValue, JsValue> {
        let response = reqwest::Client::new()
            .post(url)
            .headers(headers)
            .body(serde_json::to_string(&body).unwrap_throw())
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

        let body_bytes = response.bytes().await?;
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_throw();
        Ok(serde_wasm_bindgen::to_value(&body).unwrap_throw())
    }

    pub async fn login(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        let url = self.config.base_url.clone() + &self.config.login;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap_throw());

        let body = serde_json::json!({
            "username": username,
            "password": password
        });

        self.post(&url, headers, body).await
    }

    pub async fn register(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        let url = self.config.base_url.clone() + &self.config.register;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap_throw());

        let body = serde_json::json!({
            "username": username,
            "password": password
        });

        self.post(&url, headers, body).await
    }

    #[wasm_bindgen]
    pub async fn get_images(&self, id: Option<String>, token: String) -> Result<JsValue, JsValue> {
        let mut url = self.config.base_url.clone() + &self.config.get_images_path;
        if let Some(id) = id {
            url = self.config.base_url.clone() + &self.config.get_image_path.replace("{}", &id);
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap_throw());
        self.get(&url, headers).await
    }

    #[wasm_bindgen]
    pub async fn get_poems(&self, id: Option<String>, token: String) -> Result<JsValue, JsValue> {
        let mut url = self.config.base_url.clone() + &self.config.get_poems_path;
        if let Some(id) = id {
            url = self.config.base_url.clone() + &self.config.get_poem_path.replace("{}", &id);
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap_throw());

        self.get(&url, headers).await
    }

    #[wasm_bindgen]
    pub async fn get_profile(&self, token: String) -> Result<JsValue, JsValue> {
        let url = self.config.base_url.clone() + &self.config.get_profile_path;

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap_throw());

        self.get(&url, headers).await
    }
}

