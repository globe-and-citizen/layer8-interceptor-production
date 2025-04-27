import './assets/main.css'

import { createApp } from 'vue'
import App from './App.vue'
import * as interceptor_wasm from 'interceptor-wasm'

createApp(App).mount('#app')

interceptor_wasm.greet()