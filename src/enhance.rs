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

pub fn blur_frame(
    frame_data: &[u8],
    width: u32,
    height: u32,
    radius: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, frame_data.to_vec())
        .ok_or("Invalid frame dimensions for image buffer")?;

    let blurred = image::imageops::blur(&img, radius as f32);

    let mut output = Vec::with_capacity((width * height * 3) as usize);
    for pixel in blurred.pixels() {
        let channels = pixel.channels();
        output.extend_from_slice(channels);
    }

    log(&format!("Frame blurred: {}x{} (radius={})", width, height, radius));
    Ok(output)
}

pub fn blur_background_keep_person(
    frame_data: &[u8],
    width: u32,
    height: u32,
    blur_radius: f32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, frame_data.to_vec())
        .ok_or("Invalid frame dimensions")?;

    let blurred = image::imageops::blur(&img, blur_radius);

    let center_x = width / 2;
    let center_y = height / 2;
    let face_zone_w = width * 3 / 10;
    let face_zone_h = height / 2;

    let mut mask = vec![false; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let p = img.get_pixel(x, y);
            let r = p[0];
            let g = p[1];
            let b = p[2];

            let is_skin = {
                let r_f = r as f32;
                let g_f = g as f32;
                let b_f = b as f32;
                let max = r_f.max(g_f).max(b_f);
                let min = r_f.min(g_f).min(b_f);
                let diff = max - min;
                max > 60.0 && min < 240.0 && diff > 12.0 &&
                r_f > g_f && r_f > b_f &&
                (r_f - g_f) > 8.0 &&
                (r_f - b_f) > 8.0 &&
                (max - min) > 12.0
            };

            let dx = (x as i32 - center_x as i32).abs();
            let dy = (y as i32 - (center_y as i32 - (height as i32 / 6))).abs();
            let in_zone = dx < face_zone_w as i32 && dy < face_zone_h as i32;

            if is_skin || in_zone {
                mask[idx] = true;
            }
        }
    }

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let idx = (y * width + x) as usize;
            if mask[idx] {
                let mut count = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        let nidx = (ny * width + nx) as usize;
                        if mask[nidx] { count += 1; }
                    }
                }
                if count < 4 { mask[idx] = false; }
            }
        }
    }

    for _ in 0..2 {
        let mut dilated = mask.clone();
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let idx = (y * width + x) as usize;
                if !mask[idx] {
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let nx = (x as i32 + dx) as u32;
                            let ny = (y as i32 + dy) as u32;
                            let nidx = (ny * width + nx) as usize;
                            if mask[nidx] {
                                dilated[idx] = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        mask = dilated;
    }

    let mut output = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if mask[idx] {
                let p = img.get_pixel(x, y);
                let c = p.channels();
                output.extend_from_slice(c);
            } else {
                let p = blurred.get_pixel(x, y);
                let c = p.channels();
                output.extend_from_slice(c);
            }
        }
    }

    log(&format!("Background blurred (person kept): {}x{}", width, height));
    Ok(output)
}

pub fn composite_with_mask(
    frame_data: &[u8],
    mask_data: &[u8],
    width: u32,
    height: u32,
    blur_radius: f32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, frame_data.to_vec())
        .ok_or("Invalid frame dimensions")?;

    let blurred = image::imageops::blur(&img, blur_radius);

    let mut output = Vec::with_capacity((width * height * 3) as usize);
    for i in 0..(width * height) as usize {
        let m = mask_data.get(i).copied().unwrap_or(0) as f32 / 255.0;
        let alpha = if m > 0.6 { 1.0 } else if m < 0.3 { 0.0 } else { (m - 0.3) / 0.3 };
        let pixel_idx = i * 3;
        let p_orig = &frame_data[pixel_idx..pixel_idx + 3];
        let p_blur = &blurred.as_raw()[pixel_idx..pixel_idx + 3];
        for c in 0..3 {
            let val = p_orig[c] as f32 * alpha + p_blur[c] as f32 * (1.0 - alpha);
            output.push(val.clamp(0.0, 255.0) as u8);
        }
    }

    Ok(output)
}