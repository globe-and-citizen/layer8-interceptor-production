use wasm_bindgen::prelude::*;
use web_sys::{console, window, IdbFactory, IdbDatabase, IdbOpenDbRequest, IdbRequest,
              IdbTransactionMode, DomException};
use once_cell::unsync::OnceCell;

// thread_local! is used to safely manage the global DB variable in a way that is compatible with
// both the current single-threaded WASM environment and potential future multi-threaded scenarios
thread_local! {
    // OnceCell is thread-safe Global, the value can be set only once and can be accessed safely in multiple threads
    static DB: OnceCell<IdbDatabase> = OnceCell::new();
}

const DB_NAME: &str = "test_wasm_db";
const DB_OBJECT_STORE: &str = "images";

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsError> {
    // access the global `window` object (crate feature: web-sys::window)
    let window = window().expect("should have a window in this context");

    let indexed_db: IdbFactory;
    // get indexedDB field of window object (crate feature: web-sys::IdbFactory)
    // pub fn indexed_db(&self) -> Result<Option<IdbFactory>, JsValue>
    match window.indexed_db() {
        Ok(Some(factory)) => {
            indexed_db = factory
        }
        Ok(None) => {
            return Err(JsError::new("IndexedDB is not supported"));
        }
        Err(err) => {
            return Err(JsError::new(&format!("Error accessing IndexedDB: {:?}", err)));
        }
    }

    let db_request: IdbOpenDbRequest;
    match indexed_db.open_with_u32(DB_NAME, 4) {
        Ok(request) => {
            db_request = request
        }
        Err(err) => {
            return Err(JsError::new(&format!("Error opening database: {:?}", err)));
        }
    }

    let onsuccess_handler = |event: web_sys::Event| {
        // Get the result of the request -> feature: EventTarget
        let target = event.target().unwrap();

        // convert the target to IdbOpenDbRequest
        let request = target.dyn_into::<IdbOpenDbRequest>().unwrap();
        let result = request.result().unwrap();

        let db = result.dyn_into::<IdbDatabase>().unwrap();

        DB.with(|global_db| global_db.set(db.clone()).unwrap());

        console::log_1(&format!("Database opened successfully: {:?}", db.name()).into());
        console::log_1(&format!("Database version: {:?}", db.version()).into());
    };

    let onsuccess_handler_closure = Closure::once_into_js(onsuccess_handler);
    db_request.set_onsuccess(Some(onsuccess_handler_closure.as_ref().unchecked_ref()));

    let onerror = Closure::once_into_js(move |event: web_sys::Event| {
        let request = event.target().unwrap().dyn_into::<IdbOpenDbRequest>().unwrap();
        let error = request.error().unwrap().unwrap().dyn_into::<DomException>().unwrap();
        console::error_1(&format!("Error opening database: {:?}", error).into());
    });

    db_request.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    // The onupgradeneeded event is triggered when the database is being created or upgraded before onsuccess is called.
    let onupgradeneeded = Closure::once_into_js(move |event: web_sys::Event| {
        let request = event.target().unwrap().dyn_into::<IdbOpenDbRequest>().unwrap();
        let db = request.result().unwrap().dyn_into::<IdbDatabase>().unwrap();

        console::log_1(&format!("Upgrading database: {:?}", db.name()).into());
        // Create an object store if it doesn't exist -> feature needed web_sys::DomStringList
        if db.object_store_names().contains(DB_OBJECT_STORE) {
            console::log_1(&"Object store already exists".into());
        } else {
            db.create_object_store(DB_OBJECT_STORE).expect("should create object store");
            console::log_1(&"Object store created".into());
        }
    });

    db_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));

    Ok(())
}

#[wasm_bindgen]
pub fn get_db() {
    DB.with(|global_db| {
        if let Some(db) = global_db.get() {
            console::log_1(&format!("Get database name: {:?}", db.name()).into());
        } else {
            console::warn_1(&"Database is not initialized.".into());
        }
    });
}

#[wasm_bindgen]
pub fn save_image(filename: String, data: web_sys::Blob) {
    console::log_1(&format!("Input file: {}, blob: {:?}", filename, data).into());
    DB.with(move |global_db| {
        if let Some(db) = global_db.get() {
            console::log_1(&format!("Get database name: {:?}", db.name()).into());

            let transaction = db.transaction_with_str_and_mode(DB_OBJECT_STORE, IdbTransactionMode::Readwrite).unwrap();

            transaction.set_oncomplete(Some(Closure::once_into_js(|| {
                console::log_1(&"Transaction completed!".into())
            }).as_ref().unchecked_ref()));

            transaction.set_onerror(Some(Closure::once_into_js(|event: web_sys::Event| {
                let request = event.target().unwrap().dyn_into::<IdbRequest>().unwrap();
                let error = request.error().unwrap().unwrap().dyn_into::<DomException>().unwrap();
                console::error_1(&format!("Transaction OnError: {:?}", error).into());
            }).as_ref().unchecked_ref()));

            let object_store = transaction.object_store(DB_OBJECT_STORE).unwrap();

            let object_store_request = object_store.add_with_key(&JsValue::from(data), &filename.into()).unwrap();

            object_store_request.set_onsuccess(Some(Closure::once_into_js(|| {
                console::log_1(&"Image is added to db".into());
            }).as_ref().unchecked_ref()));

            object_store_request.set_onerror(Some(Closure::once_into_js(|event: web_sys::Event| {
                let request = event.target().unwrap().dyn_into::<IdbRequest>().unwrap();
                let error = request.error().unwrap().unwrap().dyn_into::<DomException>().unwrap();
                console::error_1(&format!("ObjectRequest OnError: {:?}", error).into());
            }).as_ref().unchecked_ref()))
        } else {
            console::warn_1(&"Database is not initialized.".into());
        }
    });
}
