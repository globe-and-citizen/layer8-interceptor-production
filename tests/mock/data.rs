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
            6, 125, 114, 67, 188, 229, 229, 105, 42, 171, 210, 162, 67, 30, 187, 53, 132, 163, 2,
            23, 171, 185, 171, 134, 182, 91, 11, 131, 25, 186, 174, 245, 36, 187, 139, 150, 43,
            208, 77, 9, 124, 244, 160, 195, 176, 40, 224, 236, 157, 53, 180, 67, 254, 221, 122, 59,
            168, 45, 101, 113, 127, 172, 166, 206, 65, 242, 201, 48, 68, 233, 160, 196, 181, 38,
            108, 231, 99, 160, 46, 112,
        ],
        l8_request_object: L8RequestObject {
            uri: "/me".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
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
            6, 255, 49, 225, 129, 239, 59, 5, 123, 241, 139, 245, 251, 6, 2, 178, 135, 21, 29, 185,
            110, 247, 30, 176, 103, 8, 54, 151, 119, 239, 126, 176, 28, 93, 119, 73, 11, 112, 228,
            155, 228, 207, 183, 144, 186, 241, 121, 197, 148, 156, 195, 123, 121, 31, 78, 172, 42,
            251, 141, 0, 60, 155, 115, 224, 23, 178, 49, 48, 189, 96, 14, 169, 87, 18, 159, 168,
            82, 133, 95, 117, 246, 157, 22, 134, 74, 200, 101, 158, 229, 141, 78, 45, 158, 114,
            136, 182, 67, 24, 42, 231, 18, 179, 95, 189, 231, 208, 203, 15, 202, 187, 148, 20, 122,
            214, 141, 208, 53, 249, 43, 85, 64, 250, 169, 83, 173, 133, 232, 1, 140, 117, 241, 146,
            206, 141, 145, 182, 242, 77, 1, 197, 75, 252, 83, 17, 156, 10, 246, 84, 108, 252, 153,
            28, 97, 191, 6, 113, 200, 161, 182, 139, 130, 50, 67, 223, 126, 122, 214, 34, 237, 55,
            228, 171, 18, 114, 115, 86, 177, 40, 130, 184, 121, 70, 75, 252, 15, 49, 74, 235, 0,
            41, 100, 45, 167, 120, 43, 224, 157, 108, 139, 80, 252, 237, 251, 41, 85, 57, 171, 7,
            111, 5, 195, 128, 88, 242, 230, 198, 55, 173, 45, 186, 44, 20, 107, 157, 9, 114, 191,
            87, 43, 255, 135, 39, 233, 40, 98, 241, 210, 141, 208, 127, 121, 106, 128, 36, 140, 94,
            109, 3, 108, 178, 216, 254, 214, 86, 153, 220, 172, 137, 109, 161, 151, 246, 191, 242,
            26, 87, 33, 97, 39, 30, 148, 48, 35, 180, 190, 164, 250, 216, 100, 71, 247, 69, 49,
            110, 195, 83, 189, 202, 48, 80, 44, 198, 244, 84, 132, 113, 18, 227, 112, 90, 193, 144,
            233, 231, 190, 223, 109, 74, 161, 250, 122, 49, 83, 23, 14, 175, 118, 123, 14, 5, 79,
            113, 205, 92, 77, 218, 23, 119, 249, 102, 175, 85, 3, 17, 165, 128, 125, 124, 155, 115,
            171, 182, 197, 122, 32, 126, 17, 154, 185, 181, 55, 221, 226, 113, 13, 185, 237, 10,
            54, 98, 211, 40, 59, 252, 3, 34, 145, 155, 58, 217, 192, 183, 206, 39, 200, 102, 116,
            183, 0, 99, 230, 140, 228, 185, 3, 118, 21, 50, 162, 130, 41, 179, 203, 118, 153, 22,
            244, 53, 150, 33, 91, 211, 162, 161, 196, 210, 92, 218, 196, 161, 80, 117, 169, 187,
            84, 104, 124, 48, 201, 174, 243, 208, 116, 183, 60, 169, 207, 50, 40, 249, 199, 216,
            114, 67, 213, 142, 99, 4, 239, 245, 40, 232, 174, 113, 166, 108, 118, 223, 57, 172, 48,
            36, 36, 14, 138, 217, 127, 209, 211, 216, 71, 99, 202, 82, 24, 0, 156, 93, 95, 186,
            150, 56, 88, 133, 87, 131, 154, 133, 186, 193, 68, 63, 29, 91, 182, 122, 188, 234, 48,
            126, 129, 206, 117, 191, 245, 138, 215, 70, 34, 248, 150, 137, 139, 101, 62, 31, 127,
            174, 190, 189, 2, 136, 174, 58, 95, 84, 81, 45, 193, 92, 108, 183, 147, 194, 34, 122,
            91, 158, 146, 12, 39, 32, 0,
        ],
        l8_response_object: L8ResponseObject {
            status: 401,
            status_text: "Unauthorized".to_string(),
            headers: [
                (
                    "date".to_string(),
                    serde_json::Value::String("Thu, 21 May 2026 04:25:36 GMT".to_string()),
                ),
                (
                    "content-length".to_string(),
                    serde_json::Value::String("23".to_string()),
                ),
                (
                    "vary".to_string(),
                    serde_json::Value::String("Origin".to_string()),
                ),
                (
                    "keep-alive".to_string(),
                    serde_json::Value::String("timeout=5".to_string()),
                ),
                (
                    "connection".to_string(),
                    serde_json::Value::String("keep-alive".to_string()),
                ),
                (
                    "x-powered-by".to_string(),
                    serde_json::Value::String("Express".to_string()),
                ),
                (
                    "etag".to_string(),
                    serde_json::Value::String("W/\"17-VIEFRCuHQRfwSbpuk4+iLdGeWgY\"".to_string()),
                ),
                (
                    "content-type".to_string(),
                    serde_json::Value::String("application/json; charset=utf-8".to_string()),
                ),
                (
                    "access-control-allow-credentials".to_string(),
                    serde_json::Value::String("true".to_string()),
                ),
            ]
            .into_iter()
            .collect(),
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
            182, 127, 86, 194, 126, 24, 1, 71, 32, 4, 11, 1, 77, 193, 80, 139, 29, 145, 44, 253,
            234, 157, 74, 73, 140, 131, 143, 204, 216, 112, 59, 147, 115, 82, 1, 117, 31, 166, 81,
            111, 193, 158, 246, 72, 140, 201, 131, 57, 214, 140, 127, 170, 80, 120, 150, 176, 239,
            59, 4, 177, 67, 153, 154, 119, 140, 71, 115, 205, 231, 126, 5, 151, 68, 17, 173, 237,
            240, 186, 116, 77, 116, 205, 18, 42, 79, 218, 102, 187, 91, 168,
        ],
        l8_request_object: L8RequestObject {
            uri: "/profile/test".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
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
            147, 136, 252, 165, 213, 182, 174, 57, 125, 170, 104, 69, 251, 221, 2, 91, 27, 50, 200,
            1, 22, 19, 190, 215, 164, 255, 44, 140, 63, 246, 158, 50, 143, 2, 206, 98, 179, 233,
            237, 182, 52, 17, 118, 48, 87, 154, 28, 129, 247, 91, 43, 207, 55, 147, 89, 241, 18,
            40, 92, 108, 137, 233, 167, 159, 80, 131, 163, 78, 35, 134, 198, 203, 160, 60, 253,
            238, 90, 223, 12, 74, 106, 156, 27, 209, 228, 32, 8, 177, 120, 16, 175, 3, 151, 63,
            204, 200, 174, 108, 67, 196, 91, 119, 55, 55, 15, 90, 241, 151, 62, 243, 225, 164, 135,
            203, 158, 224, 98, 20, 182, 75, 201, 164, 11, 64, 33, 136, 43, 252, 191, 21, 6, 80,
            160, 89, 158, 145, 61, 97, 41, 123, 112, 243, 218, 169, 192, 39, 104, 226, 67, 116,
            188, 116, 153, 79, 125, 140, 113, 37, 186, 209, 117, 137, 41, 171, 67, 231, 91, 17,
            168, 48, 93, 198, 156, 207, 220, 164, 74, 5, 225, 217, 245, 95, 202, 23, 27, 251, 152,
            57, 76, 80, 152, 250, 235, 223, 119, 81, 238, 35, 15, 3, 138, 230, 29, 19, 211, 113,
            162, 163, 8, 162, 181, 106, 167, 235, 2, 50, 254, 42, 96, 166, 245, 115, 27, 230, 87,
            3, 85, 200, 106, 80, 27, 142, 176, 194, 124, 202, 224, 55, 165, 166, 183, 136, 144,
            247, 243, 156, 198, 141, 205, 199, 83, 46, 110, 123, 6, 21, 221, 65, 80, 102, 0, 168,
            25, 75, 220, 87, 44, 16, 153, 183, 85, 151, 246, 204, 39, 35, 16, 68, 224, 226, 8, 176,
            138, 67, 223, 33, 18, 129, 165, 54, 43, 222, 143, 220, 160, 175, 206, 91, 204, 54, 48,
            120, 13, 12, 109, 18, 8, 120, 12, 98, 58, 11, 28, 130, 175, 19, 153, 124, 219, 114, 45,
            214, 75, 69, 224, 33, 15, 163, 227, 247, 237, 122, 147, 189, 56, 142, 42, 2, 61, 253,
            83, 101, 41, 93, 230, 86, 73, 157, 106, 226, 233, 125, 216, 85, 118, 83, 201, 61, 22,
            197, 52, 111, 6, 202, 163, 161, 191, 79, 206, 147, 69, 196, 55, 231, 171, 248, 27, 233,
            21, 164, 7, 13, 147, 82, 27, 232, 199, 255, 14, 178, 80, 69, 232, 204, 122, 60, 65,
            250, 21, 8, 146, 143, 238, 12, 119, 49, 201, 68, 128, 102, 254, 71, 44, 152, 75, 40,
            198, 22, 117, 86, 111, 7, 43, 212, 2, 193, 252, 153, 103, 238, 239, 7, 37, 14, 94, 216,
            176, 74, 38, 85, 245, 243, 255, 67, 73, 176, 149, 245, 148, 110, 230, 195, 131, 33,
            222, 16, 114, 178, 111, 64, 188, 38, 226, 96, 172, 54, 104, 128, 253, 16, 237, 179,
            122, 137, 102, 14, 230, 66, 45, 30, 141, 51, 57, 32, 74, 44, 164, 74, 16, 150, 136, 9,
            46, 66, 98, 0, 163, 95, 141, 87, 117, 76, 148, 157, 7, 84, 5, 35, 51, 99, 53, 18, 137,
            23, 174, 128, 125, 186, 0, 26, 146, 92, 116, 198, 229, 191, 106, 226, 237, 92, 87, 169,
            60, 207, 26, 178, 90, 207, 132, 50, 162, 160, 175, 160, 153, 142, 8, 56, 197, 103, 91,
            239, 100, 230, 34, 50, 5, 174, 164, 226, 0, 137, 107, 37, 45, 151, 222, 70, 199, 127,
            168, 61, 224, 147, 51, 97, 189, 62, 129, 116, 47, 150, 58, 26, 45, 126, 196, 52, 22,
            76, 242, 209, 211, 39, 218, 31, 159, 19, 21, 82, 181, 145, 104, 92, 116, 96, 109, 142,
            64, 172, 133, 152, 24, 245, 182, 175, 156, 174, 248, 199, 191, 223, 138, 32, 203, 165,
            247, 238, 32, 193, 99, 41, 199, 117, 23, 140, 248, 6, 26, 59, 121, 222, 89, 220, 119,
            20, 63, 144, 79, 218, 248, 199, 130, 164, 62, 218, 14, 145, 111, 50, 115, 243, 156,
            124, 251, 194, 227, 81, 241, 116, 40, 134, 154, 173, 130, 135, 50, 14, 231, 109, 78,
            25, 60, 142, 44, 94, 89, 255, 141, 136, 242, 249, 137, 105, 182, 217, 234, 107, 35,
            117, 134, 178, 173, 107, 53, 86, 229, 253, 93, 218, 116, 72, 71, 51, 226, 91, 121, 143,
            223, 64, 51, 77, 246, 137, 175, 70, 179, 252, 105, 56, 84, 200, 118, 205, 57, 220, 219,
            179, 120, 217, 62, 217, 56, 14, 233, 19, 128, 31,
        ],
        l8_response_object: L8ResponseObject {
            status: 200,
            status_text: "OK".to_string(),
            headers: [
                (
                    "date".to_string(),
                    serde_json::Value::String("Thu, 21 May 2026 04:26:41 GMT".to_string()),
                ),
                (
                    "content-length".to_string(),
                    serde_json::Value::String("84".to_string()),
                ),
                (
                    "vary".to_string(),
                    serde_json::Value::String("Origin".to_string()),
                ),
                (
                    "x-powered-by".to_string(),
                    serde_json::Value::String("Express".to_string()),
                ),
                (
                    "connection".to_string(),
                    serde_json::Value::String("keep-alive".to_string()),
                ),
                (
                    "keep-alive".to_string(),
                    serde_json::Value::String("timeout=5".to_string()),
                ),
                (
                    "etag".to_string(),
                    serde_json::Value::String("W/\"54-ixZzo2slWOb49jyPhvzKYcRgt0Q\"".to_string()),
                ),
                (
                    "content-type".to_string(),
                    serde_json::Value::String("application/json; charset=utf-8".to_string()),
                ),
                (
                    "access-control-allow-credentials".to_string(),
                    serde_json::Value::String("true".to_string()),
                ),
            ]
            .into_iter()
            .collect(),
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
