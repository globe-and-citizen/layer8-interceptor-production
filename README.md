# interceptor-refactor
A refactor of the current source repos that the layer8-interceptor makes use of.


### Init project
- Initiate Rust library:

```bash
$ cargo init --lib
```
Write rust code in `src/lib.rs`

- Add dependencies to `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2.88"
web-sys = { version = "0.3.77", features = ["console"] }
```

Build wasm: `wasm-pack build` - this will create a `pkg` folder with the compiled wasm and js bindings.

- Initiate a new frontend:
```bash
$ npm create vue@latest frontend
```

- Add dependencies to `frontend/package.json`:

```json
  ...
  "dependencies": {
    "vite-plugin-wasm": "^3.4.1",
    "vue": "^3.5.13",
    "interceptor-wasm": "file:../pkg"
  },
  ...
```

- Add wasm plugin to `frontend/vite.config.js`:

```javascript
import wasm from "vite-plugin-wasm";

export default {
    plugins: [
        // other plugins...
        wasm(),
    ],
    ...
}
```

Run `cd frontend && npm install` to install the dependencies.

Open `frontend/src/main.js` and import the wasm module, for example: `import * as interceptor_wasm from 'interceptor-wasm';`

You can then use the wasm module.

Run frontend with `npm run dev` and open the browser to see the result.

- Publish to npm:
```bash
$ wasm-pack build
$ cd pkg && wasm-pack publish
```