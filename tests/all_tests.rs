use wasm_bindgen_test::wasm_bindgen_test_configure;
#[path = "./mock/mod.rs"]
mod mock;

wasm_bindgen_test_configure!(run_in_browser);

mod tests_benchmark {
    use l8_intercept::utils::parse_form_data_to_array;
    use {
        l8_intercept::{init_tunnel::init_tunnel},
        uuid::Uuid,
        wasm_bindgen_test::*,
        web_sys::{FormData, console},
    };
    use crate::mock::init_tunnel::MockInitTunnelHttpCaller;

    const MB: u32 = 1024 * 1024; // 1 MB in bytes

    #[wasm_bindgen_test]
    pub async fn init_tunnel_simple_bench() {
        // doing 10_000 iterations of init_tunnel to benchmark
        let mut total_duration = 0.0;
        let mut durations = Vec::with_capacity(10_000);
        let mut best_duration = f64::MAX;
        let mut worst_duration = f64::MIN;
        for _ in 0..10_000 {
            let start = js_sys::Date::now();
            let _ = init_tunnel(
                String::from("https://example.com/"),
                MockInitTunnelHttpCaller {
                    data: vec![],
                    init: true,
                },
            )
            .await
            .unwrap();

            let duration = js_sys::Date::now() - start;
            total_duration += duration;
            durations.push(duration);
            if duration < best_duration {
                best_duration = duration;
            }
            if duration > worst_duration {
                worst_duration = duration;
            }
        }

        let average_duration = total_duration / durations.len() as f64;
        let standard_deviation = (durations
            .iter()
            .map(|&d| (d - average_duration).powi(2))
            .sum::<f64>()
            / durations.len() as f64)
            .sqrt();

        console::log_1(
            &format!(
                "Average durarion: {:.6}ms, Standard deviation: {:.6}ms, Best: {:.6}ms, Worst: {:.6}ms",
                average_duration, standard_deviation, best_duration, worst_duration
            )
                .into(),
        );
    }

    #[wasm_bindgen_test]
    async fn formdata_simple_bench() {
        let mut benchmark_result = benchmark_utils::BenchmarkResult {
            name: "FormData Parsing Benchmark".to_string(),
            benches: Vec::new(),
        };

        // 1MB, 2MB, 8MB, 16MB, 32MB,
        for i in &[1, 2, 8, 16, 32] {
            let mut total_duration = 0.0;
            let mut durations = Vec::with_capacity(100);
            let mut best_duration = f64::MAX;
            let mut worst_duration = f64::MIN;

            let form_data = {
                let form_data = FormData::new().unwrap();
                form_data.append_with_str("key1", "value1").unwrap();
                form_data.append_with_str("key2", "value2").unwrap();
                form_data.append_with_str("key3", "value3").unwrap();

                // dummy file
                let array = js_sys::Uint8Array::new_with_length(i * MB);
                array.copy_from("a".repeat((i * MB) as usize).as_bytes());

                let blob = web_sys::Blob::new_with_str_sequence(&array).unwrap();
                form_data
                    .append_with_blob_and_filename("name", &blob, "test.txt")
                    .unwrap();

                form_data
            };

            for _ in 0..100 {
                let form_data = form_data.clone();
                let boundary = Uuid::new_v4().to_string();

                // Inline timing for async function
                let start = js_sys::Date::now();
                parse_form_data_to_array(form_data, &boundary)
                    .await
                    .unwrap();
                let end = js_sys::Date::now();
                let duration = end - start; // milliseconds

                durations.push(duration);
                total_duration += duration;
                if duration < best_duration {
                    best_duration = duration;
                }
                if duration > worst_duration {
                    worst_duration = duration;
                }
            }

            let avg_duration = total_duration / 100.0;
            // Calculate standard deviation
            let variance = durations
                .iter()
                .map(|d| {
                    let diff = d - avg_duration;
                    diff * diff
                })
                .sum::<f64>()
                / 100.0;
            let stddev = variance.sqrt();

            // Color style for the log (background dark, text green)
            let style = "background: #222; color: #bada55; font-weight: bold; padding: 2px 6px; border-radius: 4px;";
            let msg = format!(
                "%cSize: {i}MB
            Average duration: {:.6}ms, Standard deviation: {:.6}ms, Best: {:.6}ms, Worst: {:.6}ms",
                avg_duration, stddev, best_duration, worst_duration
            );

            use wasm_bindgen::JsValue;
            web_sys::console::log_2(&JsValue::from_str(&msg), &JsValue::from_str(style));

            // create benchmark result
            benchmark_result.benches.push(benchmark_utils::Benchmark {
                variant: format!("{}MB", i),
                average_duration: avg_duration,
                standard_deviation: stddev,
                best_duration,
            });
        }

        #[allow(clippy::useless_format)]
        web_sys::console::log_1(
            &format!("{}", serde_json::to_string(&benchmark_result).unwrap()).into(),
        );
    }

    mod benchmark_utils {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        pub struct BenchmarkResult {
            pub name: String,
            pub benches: Vec<Benchmark>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct Benchmark {
            pub variant: String,
            pub average_duration: f64,
            pub standard_deviation: f64,
            pub best_duration: f64,
        }
    }
}

mod tests_l8_request_object {
    mod tests_constructor {
        use l8_intercept::types::request::L8RequestObject;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_test::*;
        use web_sys::{Request};

        #[wasm_bindgen_test]
        async fn test_new_with_url_only() {
            let backend_url = "http://localhost:8000".to_string();
            let resource = JsValue::from_str("http://example.com");
            let options = None;

            let result = L8RequestObject::new(backend_url, resource, options).await;

            assert!(result.is_ok());
            let req = result.unwrap();
            assert_eq!(req.method, "GET");
            assert_eq!(req.uri, "/");
        }

        #[wasm_bindgen_test]
        async fn test_new_with_invalid_backend_url() {
            let backend_url = "invalid_url".to_string();
            let resource = JsValue::from_str("http://example.com");
            let options = None;

            let result = L8RequestObject::new(backend_url, resource, options).await;

            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        async fn test_new_with_options() {
            let backend_url = "http://example.com/api/data".to_string();
            let resource = JsValue::from_str("http://example.com/api/data");
            let init = web_sys::RequestInit::new();
            init.set_method("POST");

            let headers = web_sys::Headers::new().unwrap();
            headers.set("Content-Type", "application/json").unwrap();
            init.set_headers(&headers);

            init.set_body(&JsValue::from_str("{\"key\":\"value\"}"));
            let options = Some(init);

            let result = L8RequestObject::new(backend_url, resource, options).await;
            assert!(result.is_ok());
            let req = result.unwrap();
            assert_eq!(req.method, "POST");
            assert_eq!(req.headers["content-type"], "application/json");
            assert_eq!(req.uri, "/api/data");
            assert_eq!(req.body, b"{\"key\":\"value\"}".to_vec());
        }

        #[wasm_bindgen_test]
        async fn test_new_with_resource_request() {
            let backend_url = "http://example.com/api/data".to_string();

            let init = web_sys::RequestInit::new();
            init.set_method("POST");

            let headers = web_sys::Headers::new().unwrap();
            headers.set("Content-Type", "application/json").unwrap();
            init.set_headers(&headers);

            init.set_body(&JsValue::from_str("{\"key\":\"value\"}"));

            let request =
                Request::new_with_str_and_init("https://example.com/api/data", &init).unwrap();
            let resource = JsValue::from(request);

            let result = L8RequestObject::new(backend_url, resource, None).await;
            assert!(result.is_ok());
            let req = result.unwrap();
            assert_eq!(req.method, "POST");
            assert_eq!(req.headers["content-type"], "application/json");
            assert_eq!(req.uri, "/api/data");
            assert_eq!(req.body, b"{\"key\":\"value\"}".to_vec());
        }
    }
}

mod tests_l8_response_object {
    use js_sys::futures::JsFuture;
    use l8_intercept::types::response::{L8ResponseObject};
    use wasm_bindgen_test::{wasm_bindgen_test};

    #[wasm_bindgen_test]
    async fn test_reconstruct_response() {
        let l8_response = L8ResponseObject {
            status: 401,
            status_text: "Unauthorized".to_string(),
            headers: [
                (
                    "Content-Type".to_string(),
                    serde_json::Value::String("application/json".into()),
                ),
                (
                    "keep-alive".to_string(),
                    serde_json::Value::String("timeout=5".to_string()),
                ),
                (
                    "date".to_string(),
                    serde_json::Value::String("Tue, 19 May 2026 08:32:28 GMT".to_string()),
                ),
                (
                    "access-control-allow-credentials".to_string(),
                    serde_json::Value::String("true".to_string()),
                ),
                (
                    "content-length".to_string(),
                    serde_json::Value::String("23".to_string()),
                ),
                (
                    "connection".to_string(),
                    serde_json::Value::String("keep-alive".to_string()),
                ),
                (
                    "etag".to_string(),
                    serde_json::Value::String("W/\"17-VIEFRCuHQRfwSbpuk4+iLdGeWgY\"".to_string()),
                ),
                (
                    "vary".to_string(),
                    serde_json::Value::String("Origin".to_string()),
                ),
                (
                    "x-powered-by".to_string(),
                    serde_json::Value::String("Express".to_string()),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
            body: vec![
                123, 34, 97, 117, 116, 104, 101, 110, 116, 105, 99, 97, 116, 101, 100, 34, 58, 102,
                97, 108, 115, 101, 125,
            ],
            ok: false,
            url: "http://localhost:3000/me".to_string(),
            redirected: false,
        };

        let res = l8_response.reconstruct_js_response().expect("Failed to reconstruct JS Response");
        assert_eq!(res.status(), 401);
        assert_eq!(res.status_text(), "Unauthorized");
        assert_eq!(res.headers().get("Content-Type").unwrap().unwrap(), "\"application/json\"");
        assert_eq!(res.headers().get("keep-alive").unwrap().unwrap(), "\"timeout=5\"");
        assert_eq!(res.headers().get("date").unwrap().unwrap(), "\"Tue, 19 May 2026 08:32:28 GMT\"");
        assert_eq!(res.headers().get("access-control-allow-credentials").unwrap().unwrap(), "\"true\"");
        assert_eq!(res.headers().get("content-length").unwrap().unwrap(), "\"23\"");
        assert_eq!(res.headers().get("connection").unwrap().unwrap(), "\"keep-alive\"");
        assert_eq!(res.headers().get("etag").unwrap().unwrap(), "\"W/\\\"17-VIEFRCuHQRfwSbpuk4+iLdGeWgY\\\"\"");
        assert_eq!(res.headers().get("vary").unwrap().unwrap(), "\"Origin\"");
        assert_eq!(res.headers().get("x-powered-by").unwrap().unwrap(), "\"Express\"");

        let json = JsFuture::from(res.json().unwrap()).await.expect("Failed to read JS response");

        #[derive(serde::Deserialize, Debug)]
        struct TestResponse {
            authenticated: bool,
        }

        let res_body: TestResponse = serde_wasm_bindgen::from_value(json).expect("Failed to deserialize test response");
        assert_eq!(res_body.authenticated, false);
    }

    // #[wasm_bindgen_test]
    // async fn test_handle_response() {
    //     let mut ntor_client = NTorClient::new();
    //     // ephemeral_public_key: [236,157,132,84,37,145,238,202,2,168,32,39,191,252,13,97,187,31,212,233,43,137,245,85,78,63,171,116,235,194,65,75]
    //     ntor_client.set_shared_secret(vec![25, 120, 135, 79, 224, 125, 4, 92, 252, 226, 250, 109, 40, 85, 141, 63]);
    //
    //     let init_tunnel_result = InitTunnelResult {
    //         client: ntor_client,
    //         int_rp_jwt: "".to_string(),
    //         int_fp_jwt: "dummy_fp_jwt".to_string(),
    //     };
    //
    //     let network_state_open = NetworkStateOpen {
    //         http_client: reqwest::Client::new(),
    //         init_tunnel_result,
    //         forward_proxy_url: "http://localhost:6191".to_string(),
    //     };
    //
    //     // let mut headers = web_sys::Headers::new().unwrap();
    //     // headers.set("Content-Type", "application/json").unwrap();
    //     // let init = ResponseInit::new();
    //     // init.set_status(200);
    //     // init.set_status_text("OK");
    //     // init.set_headers(&headers);
    //     //
    //     // let body = Some("{\"hello\": \"world\"}");
    //     //
    //     // let response = Response::new_with_opt_str_and_init(body, &init).expect("Failed to create response");
    //
    //     // let res = handle_response(&network_state_open, true, reqwest_response).await;
    // }
}
