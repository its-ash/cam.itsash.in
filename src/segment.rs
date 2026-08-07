use std::cell::RefCell;
use std::io::Cursor;
use tract_onnx::prelude::*;

const MODEL_BYTES: &[u8] = include_bytes!("models/selfie_segmentation.onnx");
const MODEL_SIZE: usize = 256;

type Model = std::sync::Arc<RunnableModel<TypedFact, Box<dyn TypedOp>>>;

thread_local! {
    static MODEL: RefCell<Option<Model>> = RefCell::new(None);
}

fn load_model() -> TractResult<Model> {
    let mut cursor = Cursor::new(MODEL_BYTES);
    let model = tract_onnx::onnx()
        .model_for_read(&mut cursor)?
        .with_input_fact(0, f32::fact([1, 3, MODEL_SIZE, MODEL_SIZE]).into())?
        .into_optimized()?
        .into_runnable()?;
    Ok(model)
}

pub fn init() -> Result<(), String> {
    MODEL.with(|m| {
        if m.borrow().is_some() {
            return Ok(());
        }
        let model = load_model().map_err(|e| e.to_string())?;
        *m.borrow_mut() = Some(model);
        Ok(())
    })
}

fn resize_bilinear_rgba_to_rgb_tensor(
    rgba: &[u8],
    src_w: usize,
    src_h: usize,
) -> Tensor {
    let mut data = vec![0f32; 3 * MODEL_SIZE * MODEL_SIZE];
    let x_ratio = src_w as f32 / MODEL_SIZE as f32;
    let y_ratio = src_h as f32 / MODEL_SIZE as f32;

    for y in 0..MODEL_SIZE {
        let sy = ((y as f32 + 0.5) * y_ratio - 0.5).max(0.0);
        let y0 = sy.floor() as usize;
        let y1 = (y0 + 1).min(src_h - 1);
        let fy = sy - y0 as f32;

        for x in 0..MODEL_SIZE {
            let sx = ((x as f32 + 0.5) * x_ratio - 0.5).max(0.0);
            let x0 = sx.floor() as usize;
            let x1 = (x0 + 1).min(src_w - 1);
            let fx = sx - x0 as f32;

            for c in 0..3 {
                let p00 = rgba[(y0 * src_w + x0) * 4 + c] as f32;
                let p01 = rgba[(y0 * src_w + x1) * 4 + c] as f32;
                let p10 = rgba[(y1 * src_w + x0) * 4 + c] as f32;
                let p11 = rgba[(y1 * src_w + x1) * 4 + c] as f32;
                let top = p00 + (p01 - p00) * fx;
                let bottom = p10 + (p11 - p10) * fx;
                let value = (top + (bottom - top) * fy) / 255.0;
                data[c * MODEL_SIZE * MODEL_SIZE + y * MODEL_SIZE + x] = value;
            }
        }
    }

    Tensor::from_shape(&[1, 3, MODEL_SIZE, MODEL_SIZE], &data).unwrap()
}

pub fn segment_mask(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let width = width as usize;
    let height = height as usize;
    if rgba.len() != width * height * 4 {
        return Err("frame buffer size mismatch".into());
    }

    MODEL.with(|m| {
        let borrow = m.borrow();
        let model = borrow.as_ref().ok_or("segmentation model not initialized")?;

        let input = resize_bilinear_rgba_to_rgb_tensor(rgba, width, height);
        let outputs = model.run(tvec!(input.into())).map_err(|e| e.to_string())?;
        let alphas = outputs[0]
            .to_plain_array_view::<f32>()
            .map_err(|e| e.to_string())?;

        let mut mask = vec![0u8; MODEL_SIZE * MODEL_SIZE];
        for y in 0..MODEL_SIZE {
            for x in 0..MODEL_SIZE {
                let value = alphas[[0, 0, y, x]];
                mask[y * MODEL_SIZE + x] = (value.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }

        Ok(mask)
    })
}

pub fn upscale_mask(
    mask: &[u8],
    mask_width: u32,
    mask_height: u32,
    target_width: u32,
    target_height: u32,
) -> Vec<u8> {
    let mask_width = mask_width as usize;
    let mask_height = mask_height as usize;
    let target_width = target_width as usize;
    let target_height = target_height as usize;

    let mut out = vec![0u8; target_width * target_height];
    let x_ratio = mask_width as f32 / target_width as f32;
    let y_ratio = mask_height as f32 / target_height as f32;

    for y in 0..target_height {
        let sy = ((y as f32 + 0.5) * y_ratio - 0.5).max(0.0);
        let y0 = sy.floor() as usize;
        let y1 = (y0 + 1).min(mask_height - 1);
        let fy = sy - y0 as f32;

        for x in 0..target_width {
            let sx = ((x as f32 + 0.5) * x_ratio - 0.5).max(0.0);
            let x0 = sx.floor() as usize;
            let x1 = (x0 + 1).min(mask_width - 1);
            let fx = sx - x0 as f32;

            let p00 = mask[y0 * mask_width + x0] as f32;
            let p01 = mask[y0 * mask_width + x1] as f32;
            let p10 = mask[y1 * mask_width + x0] as f32;
            let p11 = mask[y1 * mask_width + x1] as f32;
            let top = p00 + (p01 - p00) * fx;
            let bottom = p10 + (p11 - p10) * fx;
            let value = top + (bottom - top) * fy;

            out[y * target_width + x] = value.clamp(0.0, 255.0) as u8;
        }
    }

    out
}
