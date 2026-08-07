mod utils;
mod recorder;
mod codec;
mod download;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}

#[wasm_bindgen]
pub fn init_recorder() -> Result<(), JsValue> {
    recorder::init()
}

#[wasm_bindgen]
pub fn start_recording() -> Result<(), JsValue> {
    recorder::start()
}

#[wasm_bindgen]
pub fn stop_recording() -> Result<(), JsValue> {
    recorder::stop()
}

#[wasm_bindgen]
pub fn is_recording() -> bool {
    recorder::is_recording()
}

#[wasm_bindgen]
pub fn get_selected_codec() -> String {
    codec::selected_codec()
}

#[wasm_bindgen]
pub fn set_codec_preference(codec: &str) -> Result<(), JsValue> {
    codec::set_preference(codec)
}

#[wasm_bindgen]
pub fn list_supported_codecs() -> Vec<String> {
    codec::list_supported()
}

#[wasm_bindgen]
pub fn download_recording(filename: Option<String>) -> Result<(), JsValue> {
    download::download(filename)
}

