import './assets/main.css'

import { createApp } from 'vue'
import App from './App.vue'
import * as interceptor_wasm from 'interceptor-wasm'

createApp(App).mount('#app')

interceptor_wasm.test_wasm()
interceptor_wasm.persistence_check()
interceptor_wasm.check_encrypted_tunnel().then(val => {
    console.log("Encrypted tunnel check result: ", val)
})

interceptor_wasm.init_encrypted_tunnel({"hello": "world"}).then(val => {
    console.log("Encrypted tunnel init result: ", val)
})

interceptor_wasm.fetch("hello").then(val => {
    console.log("Encrypted tunnel fetch result: ", val)
})

interceptor_wasm.get_static("hello").then(val => {
    console.log("Static fetch result: ", val)
}).catch(err => {
    console.error("Static fetch error: ", err)
})