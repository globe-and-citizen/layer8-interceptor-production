import * as interceptor_wasm from "interceptor-wasm";
import {initEncryptedTunnel, ServiceProvider} from "interceptor-wasm";

const imagePreview = document.getElementById("image-preview");
const fileUpload = document.getElementById("file-upload");

document.getElementById("test-wasm").addEventListener("click", () => {
    interceptor_wasm.test_wasm();

    let body = {
        username: "tester",
        password: "1234"
    }
    let forward_proxy_url = 'http://localhost:6191';
    let backend_url = 'http://localhost:6193';

    let providers = [ServiceProvider.new(backend_url)];
    initEncryptedTunnel(forward_proxy_url, providers)
        .then(() => {
            console.log('Encrypted tunnel initialized successfully');

            interceptor_wasm.fetch(`${backend_url}/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(body)
            })
                .then(response => {
                    console.log("fetch response:", response);
                    if (response.ok) {
                        response.json().then(data => {
                            let token = data.token || data["token"] || data.get("token");
                            console.log('token', token)
                        });
                    } else {
                        alert(`An error occurred while logging in. ${response.status}`);
                    }
                }).catch(err => {
                console.error("Fetch error:", err);
            });
        }).catch(err => {
        console.error(`Failed to initialize encrypted tunnel: ${err}`)
    })
});

document.getElementById("persistence-check").addEventListener("click", () => {
    interceptor_wasm.persistence_check();
});

document.getElementById("check-encrypted-tunnel").addEventListener("click", () => {
    interceptor_wasm
        .check_encrypted_tunnel()
        .then((val) => console.log("CheckEncryptedTunnel Result:", val))
        .catch((err) => console.error("CheckEncryptedTunnel Error:", err));
});

document.getElementById("init-encrypted-tunnel").addEventListener("click", () => {
    interceptor_wasm
        .init_encrypted_tunnel({hello: "world"})
        .then((val) => console.log("InitEncryptedTunnel Result:", val))
        .catch((err) => console.error("InitEncryptedTunnel Error:", err));
});

document.getElementById("fetch").addEventListener("click", () => {
    interceptor_wasm
        .fetch("hello")
        .then((val) => console.log("Fetch Result:", val))
        .catch((err) => console.error("Fetch Error:", err));
});

document.getElementById("get-static").addEventListener("click", () => {
    interceptor_wasm
        .get_static("hello")
        .then((val) => console.log("GetStatic Result:", val))
        .catch((err) => console.error("GetStatic Error:", err));
});

fileUpload.addEventListener("change", (event) => {
    const file = event.target.files[0];
    if (file) {
        console.log("Uploaded file:", file);
        interceptor_wasm
            .save_image(file.name, file)
            .then((res) => {
                console.log(`Image ${file.name} is saved into DB`);
                return interceptor_wasm.get_image(file.name);
            })
            .then((blob) => {
                console.log("blob", blob);
                const imageUrl = URL.createObjectURL(blob);
                imagePreview.innerHTML = `<img src="${imageUrl}" alt="Uploaded Image" class="uploaded-image" />`;
            })
            .catch((err) => {
                console.log(err);
            });
    }
});
