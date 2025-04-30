<template>
  <div class="wrapper">
    <button
        @click="openDevTools"
        class="btn btn-primary pretty-button">
      Click Me
    </button>
  </div>
</template>

<script setup>
import * as interceptor_wasm from "interceptor-wasm";

function openDevTools() {
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
}
</script>

<style scoped>
.wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  width: 100%; /* Ensure the wrapper takes the full width of the viewport */
  margin: 0;
  box-sizing: border-box; /* Include padding and border in the element's dimensions */
}

.pretty-button {
  padding: 10px 20px;
  font-size: 16px;
  color: white;
  background-color: #42b983;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.pretty-button:hover {
  background-color: #0056b3;
}

</style>
