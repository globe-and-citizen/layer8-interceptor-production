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
