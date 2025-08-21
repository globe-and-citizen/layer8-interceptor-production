use {
    layer8_interceptor_production::fetch::formdata::parse_form_data_to_array,
    serde::{Deserialize, Serialize},
    uuid::Uuid,
    wasm_bindgen_test::*,
    web_sys::FormData,
};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const MB: u32 = 1024 * 1024; // 1 MB in bytes

#[derive(Serialize, Deserialize)]
struct BenchmarkResult {
    name: String,
    benches: Vec<Benchmark>,
}

#[derive(Serialize, Deserialize)]
struct Benchmark {
    variant: String,
    average_duration: f64,
    standard_deviation: f64,
    best_duration: f64,
}

#[wasm_bindgen_test]
async fn formdata_simple_bench() {
    let mut benchmark_result = BenchmarkResult {
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
            parse_form_data_to_array(form_data, boundary).await.unwrap();
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
        benchmark_result.benches.push(Benchmark {
            variant: format!("{}MB", i),
            average_duration: avg_duration,
            standard_deviation: stddev,
            best_duration,
        });
    }

    web_sys::console::log_1(
        &format!("{}", serde_json::to_string(&benchmark_result).unwrap()).into(),
    );
}
