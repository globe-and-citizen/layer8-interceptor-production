# layer8-interceptor

## Components structure:

```
layer8-interceptor
├── src
│   ├── types
│   │   ├── request
│   │   │   ├── mod.rs               - contains `L8RequestObject` struct to wrap request data
│   │   │   ├── mode_and_policies.rs - contains `L8RequestMode` enum and request policies related functions
│   │   │   ├── body.ts              - contains `L8RequestBody` struct and its methods to handle request body
│   │   ├── response.rs         - contains `L8ResponseObject` struct
│   │   ├── http_caller.rs      - contains http caller types to make real http calls or mock them
│   │   ├── network_state.rs    - contains `NetworkState`, `NetworkStateResponse` enums and `NetworkStateOpen` struct
│   │   ├── service_provider.rs - contains `ServiceProvider` struct
│   │   └── mod.rs
│   ├── utils
│   │   └── mod.rs     - contains utility functions
│   ├── constants.rs   - contains all constants used in the project
│   ├── storage.rs     - contains private in-memory variables and methods to access them via InMemoryStorage public struct
│   ├── fetch.rs       - contains exported `fetch` api
│   ├── init_tunnel.rs - contains exported `init_tunnel` api
│   └── lib.rs
├── tests
│   ├── api_tests.rs   - contains benchmark tests
```

