use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlAnchorElement, Url};
use std::cell::RefCell;
use crate::utils::log;

thread_local! {
    static LAST_BLOB_URL: RefCell<Option<String>> = RefCell::new(None);
}

pub fn store_blob_url(url: String) {
    LAST_BLOB_URL.with(|u| {
        if let Some(old) = u.borrow().as_ref() {
            let _ = Url::revoke_object_url(old);
        }
        *u.borrow_mut() = Some(url.clone());
    });
}

pub fn download(filename: Option<String>) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    let url = LAST_BLOB_URL.with(|u| u.borrow().clone())
        .ok_or("No recording available to download")?;

    let anchor = document.create_element("a")?;
    let anchor = anchor.dyn_into::<HtmlAnchorElement>()?;
    anchor.set_href(&url);

    let name = filename.unwrap_or_else(|| {
        let ts = js_sys::Date::new_0();
        let time = ts.get_time() as u64;
        format!("recording_{}.mp4", time)
    });
    anchor.set_download(&name);
    document.body().ok_or("no body")?.append_child(&anchor)?;
    anchor.click();
    anchor.remove();

    log(&format!("Download triggered: {}", name));
    Ok(())
}