<template>
  <div class="wrapper">
    <div class="button-row">
      <button @click="interceptor_wasm.test_wasm" class="btn btn-primary pretty-button">
        test_wasm
      </button>
      <button @click="interceptor_wasm.persistence_check" class="btn btn-primary pretty-button">
        persistence_check
      </button>
      <button @click="() => interceptor_wasm.check_encrypted_tunnel().
        then(val => console.log('CheckEncryptedTunnel Result:', val)).
        catch(err => console.error('CheckEncryptedTunnel Error:', err))" class="btn btn-primary pretty-button">
        check_encrypted_tunnel
      </button>
    </div>

    <div class="button-row">
      <button @click="() => interceptor_wasm.init_encrypted_tunnel({ 'hello': 'world' }).
        then(val => console.log('InitEncryptedTunnel Result:', val)).
        catch(err => console.error('InitEncryptedTunnel Error:', err))" class="btn btn-primary pretty-button">
        init_encrypted_tunnel
      </button>
      <button @click="interceptor_wasm.get_static('hello').
        then(val => console.log('GetStatic Result:', val)).
        catch(err => console.error('GetStatic Error:', err))" class="btn btn-primary pretty-button">
        get_static
      </button>


      <!-- Fetch API scenarios -->

      <!-- Simple Get -->
      <button @click="interceptor_wasm.fetch('https://jsonplaceholder.typicode.com/todos/1').
        then(val => console.log('Fetch Result for Simple Get:', {
          // iterate on and console log headers
          headers: Object.fromEntries(val.headers.entries()),
          val
        }))
        .catch(err => console.error('Fetch Error:', err))" class="btn btn-primary pretty-button">
        Simple Get
      </button>

      <!-- Get with an options arg -->
      <button @click="interceptor_wasm.fetch('https://jsonplaceholder.typicode.com/posts/1', { method: 'GET' }).
        then(val =>
          console.log('Fetch Result for Get with Options:', { headers: Object.fromEntries(val.headers.entries()), val })
        ).catch(err => console.error('Fetch Error:', err))" class="btn btn-primary pretty-button">
        Get with Options
      </button>


      <!-- Get with a Request object -->
      <button @click="interceptor_wasm.fetch(simpleGetReq).
        then(val => console.log('Fetch Result for Get with Request Object:', {
          headers: Object.fromEntries(val.headers.entries()),
          val
        }))
        .catch(err => console.error('Fetch Error:', err))" class="btn btn-primary pretty-button">
        Get with Request Object
      </button>

      <!-- Put req with a body using Request object -->
      <button @click="interceptor_wasm.fetch(fetchWithMethodBody).
        then(val => console.log('Fetch Result for Post with Body:', {
          headers: Object.fromEntries(val.headers.entries()),
          val
        }))
        .catch(err => console.error('Fetch Error:', err))" class="btn btn-primary pretty-button">
        Post with Body
      </button>

      <!-- Test plaintext data with fecth -->
      <button @click="interceptor_wasm.fetch('http://localhost:3000/echo', { method: 'POST', body: 'Hello, World!', headers: { 'Content-Type': 'text/plain' } }).
        then(val => val.text().then(text => console.log('Fetch Result for Plaintext Data:', text)))
        .catch(err => console.error('Fetch Error:', err))" class="btn btn-primary pretty-button">
        Plaintext Data
      </button>

      <!-- Test formdata with fetch -->
      <div
        style="border: 2px solid black; border-radius: 8px; padding: 10px; display: flex; align-items: center; background: #fff;">
        <input type="file" @change="onFileChange" style="margin-right: 10px;" />
        <button @click="sendFormData" class="btn btn-primary pretty-button">
          FormData
        </button>
      </div>

      <!-- Testing url params -->
      <button @click="interceptor_wasm.fetch('http://localhost:3000/params', {
        body: params,
      }).
        then(val => val.text().then(text => console.log('Fetch Result for URL Params:', text)))
        .catch(err => console.error('Fetch Error:', err))" class="btn btn-primary pretty-button">
        URL Params
      </button>

    </div>
  </div>
</template>

<script setup>
import * as interceptor_wasm from "interceptor-wasm";

const simpleGetReq = new Request('https://jsonplaceholder.typicode.com/todos/1');
const fetchWithMethodBody = new Request('https://jsonplaceholder.typicode.com/posts/1', {
  method: 'PUT',
  body: JSON.stringify({ title: 'foo', body: 'bar', userId: 1 }),
  headers: { 'Content-Type': 'application/json' }
});

const formData = new FormData();
formData.append('message', 'Hello, World!');

const params = new URLSearchParams();
params.append('message', 'Hello, World!');

const sendFormData = () => {
  const fileInput = document.querySelector('input[type="file"]');
  if (fileInput.files.length > 0) {
    formData.append('my_file', fileInput.files[0]);
  }
  formData.append('message', 'Hello, World!');

  interceptor_wasm.fetch('http://localhost:3000/formdata', {
    method: 'POST',
    body: formData,
  })
    .then(val => val.text().then(text => console.log('Fetch Result for FormData:', text)))
    .catch(err => console.error('Fetch Error:', err));
};

</script>

<style scoped>
.wrapper {
  display: flex;
  flex-wrap: wrap;
  /* Allow buttons to wrap to a new line */
  justify-content: center;
  align-items: center;
  height: 80vh;
  width: 100%;
  /* Ensure the wrapper takes the full width of the viewport */
  margin: 0;
  box-sizing: border-box;
  /* Include padding and border in the element's dimensions */
}

.button-row {
  display: flex;
  justify-content: center;
  margin-bottom: 10px;
  /* Add spacing between rows */
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
  margin: 0 10px;
  /* Add horizontal spacing between buttons */
}

.pretty-button:hover {
  background-color: #0056b3;
}
</style>
