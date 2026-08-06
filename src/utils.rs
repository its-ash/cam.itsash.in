use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&format!("Panic: {}", info).into());
    }));
}

pub fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

pub fn error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}