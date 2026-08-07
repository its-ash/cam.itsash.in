mod utils;
mod recorder;
mod codec;
mod download;
mod segment;
mod blur;

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
pub fn set_recording_stream(stream: web_sys::MediaStream) {
    recorder::set_recording_stream(stream);
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

#[wasm_bindgen]
pub fn discard_recording() {
    download::discard();
}

#[wasm_bindgen]
pub fn init_segmentation() -> Result<(), JsValue> {
    segment::init().map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn segment_mask(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, JsValue> {
    segment::segment_mask(rgba, width, height).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn apply_background_blur_full(
    full_rgba: &[u8],
    full_width: u32,
    full_height: u32,
    mask: &[u8],
    mask_width: u32,
    mask_height: u32,
    radius: u32,
) -> Result<Vec<u8>, JsValue> {
    let upscaled_mask =
        segment::upscale_mask(mask, mask_width, mask_height, full_width, full_height);
    let blurred = blur::blur_rgba(full_rgba, full_width, full_height, radius);
    Ok(blur::composite_with_mask(
        full_rgba,
        &blurred,
        &upscaled_mask,
        full_width,
        full_height,
    ))
}

