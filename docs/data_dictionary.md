# 📘 Data Dictionary

This document describes the data structures, enums, and constants used in the network service provider module. It
provides a clear understanding of each element’s purpose, type, and relationships.

---

## 1. Constants

**Description:**  
Configuration constants controlling retry behavior for network operations.

| Constant                        | Type  | Value  | Description                                                                    |
|---------------------------------|-------|--------|--------------------------------------------------------------------------------|
| `FETCH_RETRY_SLEEP_DELAY`       | `i32` | `50`   | Delay (milliseconds) between fetch retry attempts.                             |
| `INIT_TUNNEL_RETRY_SLEEP_DELAY` | `i32` | `1000` | Delay (milliseconds) between tunnel initialization retries.                    |
| `FETCH_RETRY_ATTEMPTS`          | `u32` | `3`    | Maximum number of attempts to reinitialize the tunnel during fetch operations. |
| `INIT_TUNNEL_RETRY_ATTEMPTS`    | `u32` | `3`    | Maximum number of attempts to send the `init_tunnel` request.                  |

---

## 2. `HttpCaller` (Trait)

**Description:**  
Abstracts the HTTP transport layer. Implemented by ActualHttpCaller in production and substituted with a mock in tests. 
Passed into `init_tunnel` to allow the handshake to be tested without a live network.

---

## 3. `ServiceProvider` (Struct)

**Description:**  
Represents a service provider capable of handling resource requests via a proxy or tunnel.

| Field      | Type                     | Description                                                                                      |
|------------|--------------------------|--------------------------------------------------------------------------------------------------|
| `url`      | `String`                 | Base URL of the service provider.                                                                |
| `_options` | `Option<js_sys::Object>` | Optional configuration parameters passed from JavaScript. Currently treated as a generic object. |

**Attributes:**

- `#[derive(Clone)]`: Allows duplication of the provider instance.
- `#[wasm_bindgen(getter_with_clone)]`: Enables safe access to fields from JavaScript by cloning values.

---

## 4. `NetworkState` (Enum)

**Description:**  
Represents the lifecycle state of the network connection for a `ServiceProvider`.

| Variant      | Type               | Description                                                                                                 |
|--------------|--------------------|-------------------------------------------------------------------------------------------------------------|
| `CONNECTING` | —                  | The network connection is currently being established.                                                      |
| `OPEN`       | `NetworkStateOpen` | The connection is successfully established and ready for use.                                               |
| `ERRORED`    | `JsValue`          | An error occurred during connection establishment, typically originating from JavaScript/WASM interactions. |

---

## 5. `NetworkStateOpen` (Struct)

**Description:**  
Represents the operational state of the network once the key exchange has completed and the connection is ready for use.

| Field                | Type               | Description                                                                                     |
|----------------------|--------------------|-------------------------------------------------------------------------------------------------|
| `http_client`        | `reqwest::Client`  | HTTP client used to send network requests through the established tunnel.                       |
| `init_tunnel_result` | `InitTunnelResult` | Result of the tunnel initialization process, containing cryptographic and session-related data. |
| `forward_proxy_url`  | `String`           | URL of the forward proxy through which requests are routed.                                     |

---

## 6. `NetworkStateResponse` (Enum)

**Description:**  
Represents possible outcomes when interacting with the network state or proxy server.

| Variant            | Type                | Description                                                                                   |
|--------------------|---------------------|-----------------------------------------------------------------------------------------------|
| `ProxyError`       | `JsValue`           | Indicates an unexpected or malformed response from the proxy server.                          |
| `ProviderResponse` | `web_sys::Response` | Successful HTTP response returned from the proxy server.                                      |
| `Reinitialize`     | —                   | Signals that the network connection should be reinitialized (e.g., tunnel expired or failed). |

---

## 7. `InitTunnelResponse` (Struct)

**Description:**  
Represents the server’s response when initializing a secure tunnel, including cryptographic materials and authentication
tokens.

| Field                  | Type      | JSON Name              | Description                                                     |
|------------------------|-----------|------------------------|-----------------------------------------------------------------|
| `ephemeral_public_key` | `Vec<u8>` | `ephemeral_public_key` | Ephemeral public key used for key exchange.                     |
| `t_b_hash`             | `Vec<u8>` | `t_b_hash`             | Hash value used for tunnel binding or verification.             |
| `int_rp_jwt`           | `String`  | `jwt1`                 | JWT token for the relying party.                                |
| `int_fp_jwt`           | `String`  | `jwt2`                 | JWT token for the forward proxy.                                |
| `server_id`            | `String`  | `server_id`            | Unique identifier of the server establishing the tunnel.        |
| `static_public_key`    | `Vec<u8>` | `public_key`           | Server’s static public key for long-term identity verification. |

---

## 8. `InitTunnelResult` (Struct)

**Description:**
Represents the internal result of tunnel initialization, containing the NTor client state and session JWTs.

| Field        | Type         | Description                                                                |
|--------------|--------------|----------------------------------------------------------------------------|
| `client`     | `NTorClient` | NTor client holding key-exchange state and derived cryptographic material. |
| `int_rp_jwt` | `String`     | JWT for the relying party used in subsequent requests.                     |
| `int_fp_jwt` | `String`     | JWT for the forward proxy used for routing requests.                       |

---

## 9. `L8RequestObject` (Struct)

**Description:**  
A JSON-serializable wrapper for HTTP requests compatible with the browser Fetch API.

### 9.1 Core Request Fields

| Field     | Type                                 | Description                                         |
|-----------|--------------------------------------|-----------------------------------------------------|
| `uri`     | `String`                             | Target URI for the HTTP request.                    |
| `method`  | `String`                             | HTTP method (e.g., `GET`, `POST`, `PUT`, `DELETE`). |
| `headers` | `HashMap<String, serde_json::Value>` | HTTP headers represented as key-value pairs.        |
| `body`    | `Vec<u8>`                            | Raw request body in bytes.                          |

### 9.2 User-Agent / Fetch Configuration (Not Serialized)

These fields are marked with `#[serde(skip)]` and are intended for internal usage but current unused to mirror Fetch API
behavior.

| Field                   | Type                    | Description                                                           |
|-------------------------|-------------------------|-----------------------------------------------------------------------|
| `body_used`             | `bool`                  | Indicates whether the request body has already been consumed.         |
| `cache`                 | `String`                | Cache mode (e.g., `"default"`, `"no-cache"`).                         |
| `credentials`           | `String`                | Credential mode (e.g., `"include"`, `"same-origin"`).                 |
| `destination`           | `String`                | Request destination type (e.g., `"document"`, `"script"`).            |
| `integrity`             | `String`                | Subresource integrity metadata.                                       |
| `is_history_navigation` | `bool`                  | Indicates if the request is triggered by browser history navigation.  |
| `keep_alive`            | `Option<bool>`          | Whether the request should outlive the page lifecycle.                |
| `mode`                  | `Option<L8RequestMode>` | Fetch mode (e.g., `cors`, `no-cors`, `same-origin`).                  |
| `redirect`              | `Option<String>`        | Redirect handling behavior (e.g., `"follow"`, `"error"`, `"manual"`). |
| `signal`                | `Option<AbortSignal>`   | Allows the request to be aborted.                                     |

---

## 10. `L8ResponseObject` (Struct)

**Description:**  
A JSON-serializable representation of an HTTP response received from the proxy, adapted for WASM compatibility.

| Field         | Type                                 | Description                                                                        |
|---------------|--------------------------------------|------------------------------------------------------------------------------------|
| `status`      | `u16`                                | HTTP status code (e.g., `200`, `404`).                                             |
| `status_text` | `String`                             | Human-readable description of the status code (e.g., `"OK"`).                      |
| `headers`     | `HashMap<String, serde_json::Value>` | Collection of HTTP headers represented as key-value pairs.                         |
| `body`        | `Vec<u8>`                            | Raw response body in bytes.                                                        |
| `ok`          | `bool`                               | Indicates whether the response status is successful (`2xx`). Present but not used. |
| `url`         | `String`                             | Final URL of the response after redirects. Present but not used.                   |
| `redirected`  | `bool`                               | Indicates whether the request was redirected. Present but not used.                |

---

## 9. Relationships Overview

```text
ServiceProvider
      │
      ▼
NetworkState ──► NetworkStateOpen ──► InitTunnelResult
      │
      ▼
NetworkStateResponse

L8RequestObject ──► Sent through tunnel ──► L8ResponseObject

InitTunnelResponse ──► Used during tunnel initialization