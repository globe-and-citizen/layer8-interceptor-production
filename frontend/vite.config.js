import {defineConfig} from 'vite'
import wasm from "vite-plugin-wasm";

export default defineConfig({
    plugins: [
        wasm()
    ],
    server: {
        open: true, // Automatically open the browser
        fs: {
            allow: ['..'], // Allow access to parent directory for WASM files
        },
    }
})
