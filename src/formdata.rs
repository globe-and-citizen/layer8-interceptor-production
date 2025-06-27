use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, console};

const STREAM_CHUNK_SIZE: usize = 1 * 1024 * 1024; // 1MB

pub enum FormDataParser {
    InMemory,
    Streamer(FormDataStreamer),
}

impl FormDataParser {
    pub async fn new(
        form: web_sys::FormData,
        boundary: String,
    ) -> Result<(Vec<u8>, Self), JsValue> {
        if can_stream(&form) {
            let (form_fieds, streamer) = FormDataStreamer::new(form, boundary)?;
            return Ok((form_fieds, FormDataParser::Streamer(streamer)));
        }

        Ok((
            parse_form_data_in_memory(form, boundary).await?,
            FormDataParser::InMemory,
        ))
    }
}

struct FormDataStreamer {
    files: Vec<File>,
    boundary: String,
}

struct File {
    key: String,
    filename: String,
    blob: Blob,
    read_idx: f64, // -1 if no file is currently being processed
}

impl FormDataStreamer {
    fn new(form: web_sys::FormData, boundary: String) -> Result<(Vec<u8>, Self), JsValue> {
        let mut data_streamer = FormDataStreamer {
            files: Vec::new(),
            boundary: boundary.clone(),
        };

        // aggregate form fields to get that out of the way
        let prefix = format!("--{}\r\nContent-Disposition: form-data", boundary);
        let mut content: Vec<Uint8Array> = Vec::new();

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

                content.push(chunk);
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

            let blob = value.dyn_into::<Blob>().map_err(|_| {
                JsValue::from_str("Expected second type cast of FormData entry to be a Blob")
            })?;

            data_streamer.files.push(File {
                key,
                filename,
                blob,
                read_idx: -1.0,
            });
        }

        let mut chunks = Uint8Array::new_with_length(0);
        for part in content {
            let new_length = chunks.length() + part.length();
            let temp = Uint8Array::new_with_length(new_length);
            temp.set(&chunks, 0);
            temp.set(&part, chunks.length());
            chunks = temp;
        }

        Ok((chunks.to_vec(), data_streamer))
    }

    // Streams the next file in the queue, if available. The stream uses  FormData content encoding.
    // Returns `None` if there are no more files to stream.
    async fn stream(&mut self) -> Result<Option<Vec<u8>>, JsValue> {
        if self.files.is_empty() {
            return Ok(None);
        }

        let file = &mut self.files[0];
        let (start, end) = calculate_indices(&file.blob, &mut file.read_idx);

        let mut formdata_header = String::new();
        if file.read_idx == -1.0 {
            formdata_header = format!(
                "{}; name=\"{}\"{}Content-Type: {}\r\n\r\n",
                format!("--{}\r\nContent-Disposition: form-data", self.boundary),
                escape(&normalize_linefeeds(&file.key)),
                if !file.filename.is_empty() {
                    format!("; filename=\"{}\"\r\n", escape(&file.filename))
                } else {
                    "\r\n".to_string()
                },
                file.blob.type_()
            )
        }

        let mut suffix = String::new();
        let mut chunk = {
            let chunk = file.blob.slice_with_f64_and_f64(start, end)?;

            // advance the read index
            file.read_idx = end - 1.0;

            // if the end is reached, pop the file from the queue
            if end == file.blob.size() {
                self.files.remove(0);
                suffix = format!("\r\n--{}--", self.boundary);
            }

            Uint8Array::new(&wasm_bindgen_futures::JsFuture::from(chunk.array_buffer()).await?)
                .to_vec()
        };

        let mut stream = Vec::new();
        stream.extend_from_slice(formdata_header.as_bytes());
        stream.extend_from_slice(&chunk);
        stream.extend_from_slice(&suffix.as_bytes());

        Ok(Some(stream))
    }
}

fn calculate_indices(blob: &Blob, start: &mut f64) -> (f64, f64) {
    if *start == -1.0 {
        *start = 0.0;
    }

    let mut end = blob.size();
    let slice_end = *start + STREAM_CHUNK_SIZE as f64;
    if slice_end < end {
        end = slice_end
    }

    (*start, end)
}

fn can_stream(form: &web_sys::FormData) -> bool {
    for entry in form.entries() {
        if let Ok(val) = entry {
            // if we have a blob treat it as a file
            if let Some(val) = val.dyn_ref::<web_sys::Blob>() {
                // If the blob size is greater than 5MB, we need to stream it
                if val.size() > STREAM_CHUNK_SIZE as f64 {
                    return true;
                }
            }
        }
    }

    false
}

// Converts an instance of `web_sys::FormData` to a `Uint8Array`
pub async fn parse_form_data_in_memory(
    form: web_sys::FormData,
    boundary: String,
) -> Result<Vec<u8>, JsValue> {
    let body = extract_body_in_memory(form, &boundary).await?;
    let mut chunks = Uint8Array::new_with_length(0);

    for part in body {
        let new_length = chunks.length() + part.length();
        let temp = Uint8Array::new_with_length(new_length);
        temp.set(&chunks, 0);
        temp.set(&part, chunks.length());
        chunks = temp;
    }

    Ok(chunks.to_vec())
}

// Ref: <https://github.com/nodejs/undici/blob/e39a6324c4474c6614cac98b8668e3d036aa6b18/lib/fetch/body.js#L31>
async fn extract_body_in_memory(
    form: web_sys::FormData,
    boundary: &str,
) -> Result<Vec<Uint8Array>, JsValue> {
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
