use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::console;
use crate::storage::InMemoryCache;
use crate::utils::{escape, normalize_linefeeds};

/// Converts an instance of `web_sys::FormData` to a `Uint8Array`
///
///  // Sample output:
/// --AaB03x
/// content-disposition: form-data; name="field1"
/// content-type: text/plain;charset=windows-1250
/// content-transfer-encoding: quoted-printable
///
/// Joe owes =80100.
/// --AaB03x
///
/// Ref: <https://github.com/nodejs/undici/blob/e39a6324c4474c6614cac98b8668e3d036aa6b18/lib/fetch/body.js#L31>
pub async fn parse_form_data_to_array(
    form: web_sys::FormData,
    boundary: &str,
) -> Result<Vec<u8>, JsValue> {
    let prefix = format!("--{}\r\nContent-Disposition: form-data", boundary);
    let mut blob_parts = Vec::new();
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

            let chunk = chunk_str.as_bytes();
            blob_parts.extend_from_slice(chunk);

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

        let chunk = chunk_str.as_bytes();
        blob_parts.extend_from_slice(chunk);
        blob_parts.extend_from_slice(&file_contents.to_vec());
        blob_parts.extend_from_slice(&rn.to_vec()); // \r\n
    }

    let chunk = format!("--{}--", boundary);
    blob_parts.extend_from_slice(chunk.as_bytes());

    Ok(blob_parts)
}

// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read#example_1_-_simple_example>
/// Converts a ReadableStream to a byte vector by reading all chunks from the stream.
/// This function reads the stream until it is done and accumulates the data into a Vec<u8>.
///
pub async fn readable_stream_to_bytes(stream: web_sys::ReadableStream) -> Result<Vec<u8>, JsValue> {
    let dev_flag = InMemoryCache::get_dev_flag();
    let reader = stream.get_reader();
    let reader = reader
        .dyn_ref::<web_sys::ReadableStreamDefaultReader>()
        .expect_throw("Expected ReadableStreamDefaultReader, already checked");

    let mut data = Vec::new();
    loop {
        // { done, value }
        // done  - true if the stream has already given you all its data.
        // value - some data. Always undefined when done is true.
        let object = wasm_bindgen_futures::JsFuture::from(reader.read()).await?;

        let done = js_sys::Reflect::get(&object, &"done".into())
            .expect_throw("Expected 'done' property in ReadableStreamDefaultReader.read() result")
            .as_bool()
            .expect_throw(
                "Expected 'done' property to be a boolean in ReadableStreamDefaultReader.read() result",
            );

        if done {
            // If done, we break from the loop and return the accumulated data.
            if dev_flag {
                console::log_1(&format!("Stream read completed with {} bytes", data.len()).into());
            }

            break;
        }

        // value for fetch streams is a Uint8Array
        let value = js_sys::Reflect::get(&object, &"value".into())
            .expect_throw(
                "Expected 'value' property in ReadableStreamDefaultReader.read() result",
            )
            .dyn_into::<js_sys::Uint8Array>()
            .expect_throw(
                "Expected 'value' property to be a Uint8Array in ReadableStreamDefaultReader.read() result",
            )
            .to_vec();

        data.extend_from_slice(&value);
    }

    // Release the reader lock
    reader.release_lock();
    Ok(data)
}
