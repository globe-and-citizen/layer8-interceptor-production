<template>
  <div class="wrapper">
    <div class="button-row">
      <button
          @click="interceptor_wasm.test_wasm"
          class="btn btn-primary pretty-button">
        test_wasm
      </button>
      <button
          @click="interceptor_wasm.persistence_check"
          class="btn btn-primary pretty-button">
        persistence_check
      </button>
      <button
          @click="() => interceptor_wasm.check_encrypted_tunnel().
        then(val => console.log('CheckEncryptedTunnel Result:', val)).
        catch(err => console.error('CheckEncryptedTunnel Error:', err))"
          class="btn btn-primary pretty-button">
        check_encrypted_tunnel
      </button>
    </div>

    <div class="button-row">
      <button
          @click="() => interceptor_wasm.init_encrypted_tunnel({'hello': 'world'}).
        then(val => console.log('InitEncryptedTunnel Result:', val)).
        catch(err => console.error('InitEncryptedTunnel Error:', err))"
          class="btn btn-primary pretty-button">
        init_encrypted_tunnel
      </button>
      <button
          @click="interceptor_wasm.fetch('hello').
        then(val => console.log('Fetch Result:', val)).
        catch(err => console.error('Fetch Error:', err))"
          class="btn btn-primary pretty-button">
        fetch
      </button>
      <button
          @click="interceptor_wasm.get_static('hello').
        then(val => console.log('GetStatic Result:', val)).
        catch(err => console.error('GetStatic Error:', err))"
          class="btn btn-primary pretty-button">
        fetch
      </button>
    </div>
  </div>
</template>

<script setup>
import * as interceptor_wasm from "interceptor-wasm";
</script>

<style scoped>
.wrapper {
  display: flex;
  flex-wrap: wrap; /* Allow buttons to wrap to a new line */
  justify-content: center;
  align-items: center;
  height: 80vh;
  width: 100%; /* Ensure the wrapper takes the full width of the viewport */
  margin: 0;
  box-sizing: border-box; /* Include padding and border in the element's dimensions */
}

.button-row {
  display: flex;
  justify-content: center;
  margin-bottom: 10px; /* Add spacing between rows */
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
  margin: 0 10px; /* Add horizontal spacing between buttons */
}

.pretty-button:hover {
  background-color: #0056b3;
}

</style>
