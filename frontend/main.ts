import * as interceptor_wasm from "interceptor-wasm";
import {initEncryptedTunnel, ServiceProvider} from "interceptor-wasm";

const imagePreview = document.getElementById("image-preview");
const fileUpload = document.getElementById("file-upload");

document.getElementById("test-wasm").addEventListener("click", () => {
    let body = {
        username: "tester",
        password: "1234"
    }
    let forward_proxy_url = 'http://localhost:6191';
    let backend_url = 'http://10.10.10.102:6193';

    try {
        let providers = [ServiceProvider.new(backend_url)];

        initEncryptedTunnel(forward_proxy_url, providers, true)
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
    } catch (err) {
        console.error(`Failed to initialize encrypted tunnel: ${err}`)
    }
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
