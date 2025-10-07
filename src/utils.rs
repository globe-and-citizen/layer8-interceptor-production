use wasm_bindgen::JsValue;

pub(crate) async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, _: js_sys::Function| {
        _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay);
    };

    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

pub(crate) fn base_url(url: &str) -> Result<String, JsValue> {
    let url =
        url::Url::parse(url).map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    // get without query or path fragments
    let mut base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
    if let Some(port) = url.port() {
        base_url = format!("{}:{}", base_url, port);
    }

    Ok(base_url)
}

pub(crate) fn get_uri(url: &str) -> Result<String, JsValue> {
    let url_object = url::Url::parse(&url)
        .map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    let mut uri = url_object.path().to_string();
    if let Some(query) = url_object.query() {
        uri.push_str(&format!("?{}", query));
    }
    Ok(uri)
}
