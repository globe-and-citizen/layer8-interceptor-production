use serde::Deserialize;
use std::collections::HashMap;

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
