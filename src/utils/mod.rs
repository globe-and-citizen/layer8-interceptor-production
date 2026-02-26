mod cookie;
mod print;
mod headers;
mod body;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

pub use headers::*;
pub use print::*;
pub use cookie::*;
pub use body::*;


pub(crate) async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, _: js_sys::Function| {
        _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay);
    };

    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

pub(crate) fn get_base_url(url: &str) -> Result<String, JsValue> {
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
    let url_object =
        url::Url::parse(&url).map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

    let mut uri = url_object.path().to_string();
    if let Some(query) = url_object.query() {
        uri.push_str(&format!("?{}", query));
    }
    Ok(uri)
}

// returns the URL of the resource to be fetched
pub fn retrieve_resource_url(resource: &JsValue) -> Result<String, JsValue> {
    let mut resource_url = String::new();
    if resource.is_string() {
        resource_url = resource
            .as_string()
            .expect_throw("Expected resource to be a string");
    }

    // If the resource is a URL object, we return it stringified.
    if resource.is_instance_of::<web_sys::Url>() {
        return Ok(String::from(
            resource
                .dyn_ref::<web_sys::Url>()
                .expect_throw("Expected resource to be a web_sys::Url")
                .to_string(),
        ));
    }

    if resource.is_instance_of::<web_sys::Request>() {
        resource_url = resource
            .dyn_ref::<web_sys::Request>()
            .expect_throw("Expected resource to be a web_sys::Request")
            .url();
    }

    if resource_url.is_empty() {
        return Err(JsValue::from_str(&format!(
            "Invalid resource type for fetch. Expected a string, URL object, or Request object. Got: {:?}",
            resource.js_typeof(),
        )));
    }

    // validate the URL from string and Request object
    if let Err(err) = web_sys::Url::new(&resource_url) {
        // If the URL is invalid, we throw an error with the details.
        return Err(JsValue::from_str(&format!(
            "Invalid URL: {}. Error: {}",
            resource_url,
            err.as_string()
                .unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    Ok(resource_url)
}

fn escape(str: &str) -> String {
    str.replace('\n', "%0A")
        .replace('\r', "%0D")
        .replace('"', "%22")
}

fn normalize_linefeeds(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

