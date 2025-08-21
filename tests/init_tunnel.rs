use {
    layer8_interceptor_production::{http_call::MockHttpCaller, init_tunnel::init_tunnel},
    wasm_bindgen_test::*,
};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn init_tunnel_simple_bench() {
    let mock_caller = MockHttpCaller {
        data: vec![],
        init: true,
    };

    let val = init_tunnel(String::new(), mock_caller).await.unwrap();
}
