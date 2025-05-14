use wasm_bindgen::prelude::*;
use web_sys::{console, window, IdbFactory, IdbDatabase, IdbOpenDbRequest};
use once_cell::unsync::OnceCell;

// thread_local! is used to safely manage the global DB variable in a way that is compatible with
// both the current single-threaded WASM environment and potential future multi-threaded scenarios
thread_local! {
    // OnceCell is thread-safe Global, the value can be set only once and can be accessed safely in multiple threads
    static DB: OnceCell<IdbDatabase> = OnceCell::new();
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsError> {
    let dbname = "test_wasm_db";

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
    match indexed_db.open_with_u32(dbname, 4) {
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
        let error = request.onerror().unwrap();
        console::log_1(&format!("Error opening database: {:?}", error).into());
    });

    db_request.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    // The onupgradeneeded event is triggered when the database is being created or upgraded before onsuccess is called.
    let onupgradeneeded = Closure::once_into_js(move |event: web_sys::Event| {
        let request = event.target().unwrap().dyn_into::<IdbOpenDbRequest>().unwrap();
        let db = request.result().unwrap().dyn_into::<IdbDatabase>().unwrap();

        console::log_1(&format!("Upgrading database: {:?}", db.name()).into());

        // Create an object store if it doesn't exist -> feature needed web_sys::DomStringList
        if db.object_store_names().contains("new_object_store") {
            console::log_1(&"Object store already exists".into());
        } else {
            db.create_object_store("new_object_store").expect("should create object store");
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
            console::log_1(&"Database is not initialized.".into());
        }
    });
}

