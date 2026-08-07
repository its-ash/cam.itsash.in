use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use crate::utils::log;

thread_local! {
    static COMPRESSED_URL: RefCell<Option<String>> = RefCell::new(None);
}

pub fn store_compressed_url(url: String) {
    COMPRESSED_URL.with(|u| {
        if let Some(old) = u.borrow().as_ref() {
            let _ = web_sys::Url::revoke_object_url(old);
        }
        *u.borrow_mut() = Some(url);
    });
}

pub fn get_compressed_url() -> Option<String> {
    COMPRESSED_URL.with(|u| u.borrow().clone())
}

pub fn clear() {
    COMPRESSED_URL.with(|u| {
        if let Some(old) = u.borrow().as_ref() {
            let _ = web_sys::Url::revoke_object_url(old);
        }
        *u.borrow_mut() = None;
    });
}

pub fn log_compression_info(original_size: u32, compressed_size: u32) {
    let ratio = if original_size > 0 {
        (compressed_size as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };
    let saved_mb = (original_size - compressed_size) as f64 / 1024.0 / 1024.0;
    log(&format!(
        "Compression: {:.2}MB -> {:.2}MB ({}% of original, saved {:.2}MB)",
        original_size as f64 / 1024.0 / 1024.0,
        compressed_size as f64 / 1024.0 / 1024.0,
        ratio as u32,
        saved_mb
    ));
}