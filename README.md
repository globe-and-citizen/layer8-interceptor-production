# layer8-interceptor

The Interceptor is a Rust-compiled WebAssembly (WASM) module that runs inside the browser and sits between a JavaScript
client and a set of backend service providers. It loads via the WASM–JS bridge with no extension or plugin required,
transparently replacing the native `fetch` call with an encrypted, proxied equivalent routed through the Layer8 Proxy chain.
All tunnel state lives in browser memory for the duration of the page session.

Usage follows two sequential phases: first, `init_encrypted_tunnels` is called once during the page load to establish
nTor-encrypted tunnels with the backend service providers; then, `fetch` is called at runtime to intercept, encrypt, and
proxy individual requests through those tunnels.