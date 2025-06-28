import * as interceptor_wasm from "interceptor-wasm";

const imagePreview = document.getElementById("image-preview");
const fileUpload = document.getElementById("file-upload");

document.getElementById("test-wasm").addEventListener("click", () => {
    interceptor_wasm.test_wasm();
    interceptor_wasm.init_tunnel("http://localhost:6191/init-tunnel")
        .then(res => {
            // console.log("init-tunnel success:", res);
            //
            let client = res.client
            //
            let headers = new Map<string, string>([
                ["Content-Type", "application/json"],
                // ["nTor_session_id", res.ntor_session_id]
            ]);
            let options = new interceptor_wasm.HttpRequestOptions();
            options.headers = headers;

            let body = {
                username: "tester",
                password: "1234"
            }
            // const bodyBytes = new TextEncoder().encode(JSON.stringify(body));
            // let encrypted = client.encrypt(bodyBytes);
            // let encryptedBody = {
            //     nonce: Array.from(encrypted.nonce),
            //     encrypted: Array.from(encrypted.data)
            // }

            interceptor_wasm.http_post(res,"http://localhost:6191", "/login", body, options)
                .then(response => {
                    console.log("login encrypted res", response, response.get("encrypted"), response.get("nonce"))

                    let login_res = client.decrypt(
                        new Uint8Array(response.get("nonce")),
                        new Uint8Array(response.get("encrypted"))
                    )
                    console.log('login decrypted res:', login_res)
                    let deciphered = new TextDecoder().decode(login_res);
                    console.log("deciphered:", deciphered)

                }).catch(err => {
                console.error("login err", err)
            })

        }).catch(err => {
        console.error("init-tunnel error:", err)
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
