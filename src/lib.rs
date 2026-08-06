mod utils;
mod recorder;
mod codec;
mod download;
mod enhance;

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

#[wasm_bindgen]
pub fn enhance_frame(
    frame_data: &[u8],
    width: u32,
    height: u32,
    sharpness: f32,
    contrast: f32,
    saturation: f32,
) -> Result<Vec<u8>, JsValue> {
    enhance::enhance_frame(frame_data, width, height, sharpness, contrast, saturation)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn get_enhance_defaults() -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"sharpness".into(), &enhance::default_sharpness().into());
    js_sys::Reflect::set(&obj, &"contrast".into(), &enhance::default_contrast().into());
    js_sys::Reflect::set(&obj, &"saturation".into(), &enhance::default_saturation().into());
    obj.into()
}