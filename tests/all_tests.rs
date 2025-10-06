wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use {
    layer8_interceptor_production::{
        fetch::formdata::parse_form_data_to_array, http_call_indirection::MockHttpCaller,
        init_tunnel::init_tunnel,
    },
    uuid::Uuid,
    wasm_bindgen_test::*,
    web_sys::{FormData, console},
};

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
            MockHttpCaller {
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
