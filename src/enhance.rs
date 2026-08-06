use image::{ImageBuffer, Rgb, Pixel};
use wasm_bindgen::JsValue;
use crate::utils::log;

pub fn enhance_frame(
    frame_data: &[u8],
    width: u32,
    height: u32,
    sharpness: f32,
    contrast: f32,
    saturation: f32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, frame_data.to_vec())
        .ok_or("Invalid frame dimensions for image buffer")?;

    let mut enhanced = img;

    if saturation != 1.0 {
        for pixel in enhanced.pixels_mut() {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;
            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
            pixel[0] = (gray + (r - gray) * saturation).clamp(0.0, 255.0) as u8;
            pixel[1] = (gray + (g - gray) * saturation).clamp(0.0, 255.0) as u8;
            pixel[2] = (gray + (b - gray) * saturation).clamp(0.0, 255.0) as u8;
        }
    }

    if contrast != 1.0 {
        let factor = (259.0 * (contrast * 255.0 + 255.0)) / (255.0 * (259.0 - contrast * 255.0));
        for pixel in enhanced.pixels_mut() {
            let channels = pixel.channels_mut();
            for c in channels.iter_mut() {
                let val = *c as f32;
                *c = (factor * (val - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    if sharpness > 0.0 {
        let sharpened = image::imageops::unsharpen(&enhanced, sharpness, 1);
        enhanced = sharpened;
    }

    let mut output = Vec::with_capacity((width * height * 3) as usize);
    for pixel in enhanced.pixels() {
        let channels = pixel.channels();
        output.extend_from_slice(channels);
    }

    log(&format!("Frame enhanced: {}x{} (sharp={}, contrast={}, sat={})", width, height, sharpness, contrast, saturation));
    Ok(output)
}

pub fn process_recording_blob(
    blob: &web_sys::Blob,
    sharpness: f32,
    contrast: f32,
    saturation: f32,
) -> Result<bool, JsValue> {
    let size = blob.size() as usize;
    log(&format!("Processing recording blob: {} bytes", size));
    Ok(true)
}

pub fn default_sharpness() -> f32 { 0.8 }
pub fn default_contrast() -> f32 { 1.15 }
pub fn default_saturation() -> f32 { 1.25 }