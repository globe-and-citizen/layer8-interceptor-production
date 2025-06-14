use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};

// Converts an instance of `web_sys::FormData` to a `Uint8Array`
pub async fn parse_form_data_to_array(
    form: web_sys::FormData,
    boundary: String,
) -> Result<Vec<u8>, JsValue> {
    let body = extract_body(form, &boundary).await?;
    let mut chunks = Uint8Array::new_with_length(0);

    for part in body {
        let new_length = chunks.length() + part.length();
        let temp = Uint8Array::new_with_length(new_length);
        temp.set(&chunks, 0);
        temp.set(&part, chunks.length());
        chunks = temp;
    }

    // Sample output:
    //
    //     --AaB03x
    //     content-disposition: form-data; name="field1"
    //     content-type: text/plain;charset=windows-1250
    //     content-transfer-encoding: quoted-printable
    //
    //     Joe owes =80100.
    //     --AaB03x
    Ok(chunks.to_vec())
}

// Ref: <https://github.com/nodejs/undici/blob/e39a6324c4474c6614cac98b8668e3d036aa6b18/lib/fetch/body.js#L31>
async fn extract_body(form: web_sys::FormData, boundary: &str) -> Result<Vec<Uint8Array>, JsValue> {
    let prefix = format!("--{}\r\nContent-Disposition: form-data", boundary);
    let mut blob_parts: Vec<Uint8Array> = Vec::new();
    let rn = Uint8Array::from(&[13, 10][..]); // '\r\n'

    // for (const [name, value] of inputFormData)
    for entry in form.entries() {
        let val = js_sys::Array::from(&entry?);
        let key = val.get(0).as_string().ok_or_else(|| {
            JsValue::from_str("Expected first element of FormData entry to be a string")
        })?;
        let value = val.get(1);

        // form field values
        if let Some(value) = value.as_string() {
            // String value
            let chunk_str = format!(
                "{}; name=\"{}\"\r\n\r\n{}\r\n",
                prefix,
                escape(&normalize_linefeeds(&key)),
                normalize_linefeeds(&value)
            );

            let chunk = Uint8Array::from(chunk_str.as_bytes());
            blob_parts.push(chunk);

            continue;
        }

        // getting the name before casting to Blob
        let filename = js_sys::Reflect::get(&value, &"name".into())
            .map_err(|e| {
                JsValue::from_str(&format!(
                    "Expected to retrieve name property before casting to Blob: {}",
                    e.as_string().unwrap_or_else(|| "unknown error".to_string())
                ))
            })?
            .as_string()
            .unwrap_or_default();

        let blob = value.dyn_into::<web_sys::Blob>().map_err(|_| {
            JsValue::from_str("Expected second type cast of FormData entry to be a Blob")
        })?;

        // Blob values
        let file_contents = wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await?;
        let file_contents: Uint8Array = Uint8Array::new(&file_contents);

        let content_type = blob.type_();
        let chunk_str = format!(
            "{}; name=\"{}\"{}Content-Type: {}\r\n\r\n",
            prefix,
            escape(&normalize_linefeeds(&key)),
            if !filename.is_empty() {
                format!("; filename=\"{}\"\r\n", escape(&filename))
            } else {
                "\r\n".to_string()
            },
            content_type
        );

        let chunk = Uint8Array::from(chunk_str.as_bytes());
        blob_parts.push(chunk);
        blob_parts.push(file_contents);
        blob_parts.push(rn.clone());
    }

    let chunk = Uint8Array::from(format!("--{}--", boundary).as_bytes());
    blob_parts.push(chunk);

    Ok(blob_parts)
}

fn escape(str: &str) -> String {
    str.replace('\n', "%0A")
        .replace('\r', "%0D")
        .replace('"', "%22")
}
fn normalize_linefeeds(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}
