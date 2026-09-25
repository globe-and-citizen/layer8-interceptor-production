use l8_intercept::types::headers::L8Headers;
use l8_intercept::types::request::L8RequestObject;
use l8_intercept::types::response::L8ResponseObject;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ResponseFromProxy {
    pub status: u16,
    pub status_str: String,
    pub headers: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct MockData {
    pub shared_secret: [u8; 16],
    pub forward_proxy_url: String,
    #[allow(dead_code)]
    pub request_encrypted_body: Vec<u8>,
    pub l8_request_object: L8RequestObject,
    pub response_from_proxy: ResponseFromProxy,
    pub response_encrypted_body: Vec<u8>,
    pub l8_response_object: L8ResponseObject,
}

pub(crate) fn get_mock_data() -> [MockData; 2] {
    let mock1 = MockData {
        shared_secret: [
            128, 233, 21, 160, 15, 80, 6, 198, 169, 140, 10, 1, 21, 175, 4, 98,
        ],
        forward_proxy_url: "http://example-forward-proxy.com".to_string(),
        request_encrypted_body: vec![
            30, 22, 249, 139, 40, 35, 218, 178, 106, 112, 176, 237, 26, 62, 53, 193, 221, 194, 213,
            62, 6, 191, 185, 230, 208, 236, 139, 122, 160, 170, 62, 98, 124, 63, 191, 168, 198,
            226, 133,
        ],
        l8_request_object: L8RequestObject {
            uri: "/me".to_string(),
            method: "GET".to_string(),
            headers: L8Headers::new(),
            body: vec![],
            body_used: false,
            cache: "default".to_string(),
            credentials: "include".to_string(),
            destination: "".to_string(),
            integrity: "".to_string(),
            is_history_navigation: false,
            keep_alive: None,
            mode: Some(l8_intercept::types::request::mode_and_policies::L8RequestMode::Cors),
            redirect: Some("follow".to_string()),
            signal: None,
        },
        response_from_proxy: ResponseFromProxy {
            status: 200,
            status_str: "OK".to_string(),
            headers: [(
                "content-type".to_string(),
                serde_json::Value::String("application/json".to_string()),
            )]
            .into_iter()
            .collect(),
        },
        response_encrypted_body: vec![
            76, 51, 169, 187, 181, 183, 255, 92, 125, 182, 181, 57, 251, 80, 1, 206, 138, 164, 174,
            62, 206, 28, 198, 73, 176, 210, 247, 20, 125, 248, 230, 241, 214, 41, 51, 68, 3, 100,
            255, 129, 147, 147, 13, 252, 82, 228, 43, 150, 199, 208, 203, 201, 117, 48, 155, 24,
            18, 157, 130, 102, 185, 32, 131, 18, 115, 131, 168, 10, 217, 179, 137, 111, 19, 217,
            109, 254, 101, 189, 107, 122, 155, 84, 128, 2, 78, 181, 30, 192, 155, 197, 1, 61, 148,
            41, 167, 176, 201, 125, 35, 131, 83, 71, 238, 195, 66, 108, 132, 219, 166, 99, 19, 16,
            42, 143, 91, 11, 59, 202, 41, 112, 48, 75, 207, 43, 9, 59, 75, 36, 247, 237, 245, 1,
            222, 81, 151, 200, 0, 161, 35, 77, 98, 214, 196, 216, 51, 144, 172, 240, 42, 48, 160,
            173, 106, 166, 3, 246, 62, 176, 179, 175, 191, 194, 235, 150, 167, 10, 98, 135, 121,
            250, 138, 212, 87, 183, 131, 212, 158, 93, 194, 6, 21, 14, 250, 246, 221, 251, 50, 157,
            177, 150, 29, 92, 17, 109, 240, 10, 67, 21, 126, 234, 39, 163, 115, 36, 214, 163, 89,
            101, 68, 162, 225, 71, 178, 25, 26, 166, 48, 123, 240, 174, 27, 183, 102, 172, 178, 50,
            150, 203, 40, 208, 44, 12, 236, 204, 25, 165, 22, 20, 241, 220, 159, 217, 242, 176,
            222, 250, 118, 5, 98, 163, 206, 0, 241, 135, 147, 174, 89, 56, 144, 162, 225, 106, 156,
            139, 172, 163, 109, 2, 194, 62, 37, 84, 244, 189, 238, 142, 56, 23, 250, 80, 164, 234,
            148, 109, 203, 188, 155, 137, 44, 37, 146, 189, 195, 181, 198, 39, 138, 70, 241, 37,
            201, 242, 82, 116, 235, 115, 70, 74, 32, 88, 252, 228, 30, 29, 211, 216, 66, 67, 223,
            144, 233, 248, 164, 66, 160, 169, 184, 5, 151, 39, 180, 255, 156, 93, 117, 163, 197,
            213, 49, 111, 147, 7, 130, 147, 7, 57, 11, 177, 220, 14, 193,
        ],
        l8_response_object: L8ResponseObject {
            status: 401,
            status_text: "Unauthorized".to_string(),
            headers: L8Headers::from_hashmap(
                [
                    (
                        "date".to_string(),
                        "Thu, 21 May 2026 04:25:36 GMT".to_string(),
                    ),
                    ("content-length".to_string(), "23".to_string()),
                    ("vary".to_string(), "Origin".to_string()),
                    ("keep-alive".to_string(), "timeout=5".to_string()),
                    ("connection".to_string(), "keep-alive".to_string()),
                    ("x-powered-by".to_string(), "Express".to_string()),
                    (
                        "etag".to_string(),
                        "W/\"17-VIEFRCuHQRfwSbpuk4+iLdGeWgY\"".to_string(),
                    ),
                    (
                        "content-type".to_string(),
                        "application/json; charset=utf-8".to_string(),
                    ),
                    (
                        "access-control-allow-credentials".to_string(),
                        "true".to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            body: vec![
                123, 34, 97, 117, 116, 104, 101, 110, 116, 105, 99, 97, 116, 101, 100, 34, 58, 102,
                97, 108, 115, 101, 125,
            ],
            ok: false,
            url: "http://localhost:3000/me".to_string(),
            redirected: false,
        },
    };

    let mock2 = MockData {
        shared_secret: [
            79, 197, 43, 207, 103, 198, 43, 124, 42, 250, 89, 58, 81, 77, 2, 10,
        ],
        forward_proxy_url: "http://example-forward-proxy.com".to_string(),
        request_encrypted_body: vec![
            119, 206, 92, 80, 224, 29, 236, 220, 33, 236, 50, 169, 36, 202, 59, 174, 56, 164, 66,
            76, 123, 82, 147, 21, 19, 79, 37, 223, 118, 75, 116, 29, 239, 112, 214, 177, 208, 54,
            16, 173, 77, 215, 19, 32, 86, 91, 142, 165, 101,
        ],
        l8_request_object: L8RequestObject {
            uri: "/profile/test".to_string(),
            method: "GET".to_string(),
            headers: L8Headers::new(),
            body: vec![],
            body_used: false,
            cache: "default".to_string(),
            credentials: "include".to_string(),
            destination: "".to_string(),
            integrity: "".to_string(),
            is_history_navigation: false,
            keep_alive: None,
            mode: Some(l8_intercept::types::request::mode_and_policies::L8RequestMode::Cors),
            redirect: Some("follow".to_string()),
            signal: None,
        },
        response_from_proxy: ResponseFromProxy {
            status: 200,
            status_str: "OK".to_string(),
            headers: [(
                "content-type".to_string(),
                serde_json::Value::String("application/json".to_string()),
            )]
            .into_iter()
            .collect(),
        },
        response_encrypted_body: vec![
            124, 129, 63, 44, 24, 169, 233, 79, 8, 236, 160, 157, 251, 139, 1, 161, 224, 92, 63,
            229, 57, 177, 167, 200, 182, 203, 117, 91, 17, 67, 81, 141, 134, 162, 99, 1, 21, 211,
            246, 242, 86, 36, 88, 43, 44, 81, 221, 113, 195, 115, 6, 234, 124, 145, 158, 145, 105,
            21, 85, 201, 251, 233, 214, 243, 111, 205, 128, 199, 50, 249, 179, 98, 140, 242, 170,
            116, 56, 92, 75, 224, 141, 30, 163, 210, 41, 185, 173, 183, 66, 163, 164, 86, 146, 202,
            243, 217, 86, 250, 50, 222, 19, 101, 239, 55, 133, 73, 31, 88, 136, 156, 252, 6, 32,
            242, 216, 35, 139, 221, 141, 174, 231, 113, 91, 145, 72, 63, 255, 172, 68, 157, 22, 10,
            67, 105, 48, 237, 107, 97, 222, 17, 228, 67, 41, 79, 195, 165, 226, 228, 58, 248, 133,
            71, 196, 78, 133, 151, 186, 224, 142, 23, 212, 170, 243, 108, 131, 123, 24, 140, 246,
            56, 96, 193, 86, 206, 245, 184, 75, 220, 5, 141, 153, 38, 193, 7, 64, 162, 176, 80,
            123, 0, 58, 1, 237, 114, 206, 191, 48, 119, 194, 148, 8, 5, 181, 54, 18, 124, 56, 214,
            184, 54, 177, 72, 99, 250, 21, 55, 205, 126, 236, 39, 106, 159, 162, 85, 247, 23, 86,
            106, 6, 196, 121, 2, 207, 57, 173, 159, 44, 121, 147, 108, 102, 60, 165, 1, 201, 134,
            53, 38, 215, 144, 8, 29, 22, 129, 186, 125, 53, 113, 52, 249, 145, 230, 58, 211, 73,
            12, 216, 4, 45, 147, 189, 180, 113, 234, 207, 59, 29, 221, 7, 159, 10, 118, 210, 247,
            53, 89, 3, 135, 113, 249, 231, 24, 150, 42, 191, 107, 255, 48, 82, 229, 175, 127, 138,
            28, 1, 10, 216, 39, 65, 199, 66, 208, 166, 210, 197, 90, 146, 227, 7, 233, 221, 173,
            72, 201, 200, 191, 171, 2, 69, 196, 42, 131, 113, 229, 237, 92, 155, 196, 40, 155, 113,
            116, 184, 214, 167, 131, 118, 111, 252, 107, 70, 248, 139, 52, 173, 3, 200, 81, 159,
            71, 60, 66, 8, 166, 121, 91, 248, 108, 226, 13, 8, 242, 129, 223, 151, 242, 248, 45,
            40, 179, 195, 224, 241, 112, 205, 15, 130, 119, 53, 250, 83, 190, 235, 250, 246, 150,
            44, 48, 5, 202, 56, 143, 132, 154, 126, 201, 57, 233, 120, 33,
        ],
        l8_response_object: L8ResponseObject {
            status: 200,
            status_text: "OK".to_string(),
            headers: L8Headers::from_hashmap(
                [
                    (
                        "date".to_string(),
                        "Thu, 21 May 2026 04:26:41 GMT".to_string(),
                    ),
                    ("content-length".to_string(), "84".to_string()),
                    ("vary".to_string(), "Origin".to_string()),
                    ("x-powered-by".to_string(), "Express".to_string()),
                    ("connection".to_string(), "keep-alive".to_string()),
                    ("keep-alive".to_string(), "timeout=5".to_string()),
                    (
                        "etag".to_string(),
                        "W/\"54-ixZzo2slWOb49jyPhvzKYcRgt0Q\"".to_string(),
                    ),
                    (
                        "content-type".to_string(),
                        "application/json; charset=utf-8".to_string(),
                    ),
                    (
                        "access-control-allow-credentials".to_string(),
                        "true".to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            body: vec![
                123, 34, 117, 115, 101, 114, 110, 97, 109, 101, 34, 58, 34, 116, 101, 115, 116, 34,
                44, 34, 109, 101, 116, 97, 100, 97, 116, 97, 34, 58, 123, 34, 101, 109, 97, 105,
                108, 95, 118, 101, 114, 105, 102, 105, 101, 100, 34, 58, 102, 97, 108, 115, 101,
                44, 34, 100, 105, 115, 112, 108, 97, 121, 95, 110, 97, 109, 101, 34, 58, 34, 34,
                44, 34, 99, 111, 108, 111, 114, 34, 58, 34, 34, 125, 125,
            ],
            ok: true,
            url: "http://localhost:3000/profile/test".to_string(),
            redirected: false,
        },
    };

    [mock1, mock2]
}
