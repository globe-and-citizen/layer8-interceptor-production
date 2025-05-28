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
        console::log_1(&format!("Backend created with address: {}", config.base_url).into());
        Backend {config}
    }

    async fn get(&self, url: &String, headers: HeaderMap) -> Result<JsValue, JsValue> {
        console::log_1(&format!("GET request to: {} with headers {:?}", url, headers).into());

        let response = reqwest::Client::new()
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

        let body_bytes = response.bytes().await?;

        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        console::log_1(&format!("Response body: {:?}", body).into());
        Ok(serde_wasm_bindgen::to_value(&body).unwrap())
    }

    async fn post(&self, url: &String, headers: HeaderMap, body: serde_json::Value) -> Result<JsValue, JsValue> {
        console::log_1(&format!("POST request to: {} with headers {:?}", url, headers).into());

        let response = reqwest::Client::new()
            .post(url)
            .headers(headers)
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

        let body_bytes = response.bytes().await?;

        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        console::log_1(&format!("Response body: {:?}", body).into());
        Ok(serde_wasm_bindgen::to_value(&body).unwrap())
    }

    pub async fn login(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        console::log_1(&format!("Logging in with username: {}", username).into());

        let url = self.config.base_url.clone() + &self.config.login;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let body = serde_json::json!({
            "username": username,
            "password": password
        });

        self.post(&url, headers, body).await
    }

    pub async fn register(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        console::log_1(&format!("Registering with username: {}", username).into());

        let url = self.config.base_url.clone() + &self.config.register;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let body = serde_json::json!({
            "username": username,
            "password": password
        });

        self.post(&url, headers, body).await
    }

    #[wasm_bindgen]
    pub async fn get_images(&self, id: Option<String>, token: String) -> Result<JsValue, JsValue> {
        console::log_1(&format!("Fetching images with id: {:?}", id).into());

        let mut url = self.config.base_url.clone() + &self.config.get_images_path;
        if let Some(id) = id {
            url = self.config.base_url.clone() + &self.config.get_image_path.replace("{}", &id);
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap());

        console::log_1(&format!("headers {:?}", headers).into());
        self.get(&url, headers).await
    }

    #[wasm_bindgen]
    pub async fn get_poems(&self, id: Option<String>, token: String) -> Result<JsValue, JsValue> {
        console::log_1(&format!("Fetching poems with id: {:?}", id).into());

        let mut url = self.config.base_url.clone() + &self.config.get_poems_path;
        if let Some(id) = id {
            url = self.config.base_url.clone() + format!("{}{}", &self.config.get_poem_path, id).as_str();
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap());


        console::log_1(&format!("headers {:?}", headers).into());
        self.get(&url, headers).await
    }

    #[wasm_bindgen]
    pub async fn get_profile(&self, token: String) -> Result<JsValue, JsValue> {
        console::log_1(&"Fetching profile".into());

        let url = self.config.base_url.clone() + &self.config.get_profile_path;

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap());

        console::log_1(&format!("headers {:?}", headers).into());
        self.get(&url, headers).await
    }
}

