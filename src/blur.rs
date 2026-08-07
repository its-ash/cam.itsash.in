fn box_blur_horizontal(src: &[u8], dst: &mut [u8], width: usize, height: usize, radius: usize) {
    for y in 0..height {
        let row = y * width * 4;
        let mut r = 0u32;
        let mut g = 0u32;
        let mut b = 0u32;
        let mut a = 0u32;

        for xx in 0..=radius.min(width - 1) {
            let idx = row + xx * 4;
            r += src[idx] as u32;
            g += src[idx + 1] as u32;
            b += src[idx + 2] as u32;
            a += src[idx + 3] as u32;
        }

        for x in 0..width {
            let count = (x.min(radius) + 1 + radius.min(width - 1 - x)) as u32;
            let idx = row + x * 4;
            dst[idx] = (r / count) as u8;
            dst[idx + 1] = (g / count) as u8;
            dst[idx + 2] = (b / count) as u8;
            dst[idx + 3] = (a / count) as u8;

            let add_x = x + radius + 1;
            if add_x < width {
                let add_idx = row + add_x * 4;
                r += src[add_idx] as u32;
                g += src[add_idx + 1] as u32;
                b += src[add_idx + 2] as u32;
                a += src[add_idx + 3] as u32;
            }
            if x >= radius {
                let sub_idx = row + (x - radius) * 4;
                r -= src[sub_idx] as u32;
                g -= src[sub_idx + 1] as u32;
                b -= src[sub_idx + 2] as u32;
                a -= src[sub_idx + 3] as u32;
            }
        }
    }
}

fn transpose_rgba(src: &[u8], dst: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let src_idx = (y * width + x) * 4;
            let dst_idx = (x * height + y) * 4;
            dst[dst_idx] = src[src_idx];
            dst[dst_idx + 1] = src[src_idx + 1];
            dst[dst_idx + 2] = src[src_idx + 2];
            dst[dst_idx + 3] = src[src_idx + 3];
        }
    }
}

pub fn blur_rgba(rgba: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;
    let radius = radius.max(1) as usize;

    let mut horiz = vec![0u8; rgba.len()];
    box_blur_horizontal(rgba, &mut horiz, width, height, radius);

    let mut transposed = vec![0u8; rgba.len()];
    transpose_rgba(&horiz, &mut transposed, width, height);

    let mut vert_blurred = vec![0u8; rgba.len()];
    box_blur_horizontal(&transposed, &mut vert_blurred, height, width, radius);

    let mut out = vec![0u8; rgba.len()];
    transpose_rgba(&vert_blurred, &mut out, height, width);

    out
}

#[inline]
fn glass_tint_pixel(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    const BRIGHTNESS: f32 = 1.1;
    const SATURATION: f32 = 1.3;
    const WHITE_MIX: f32 = 0.08;

    let luma = 0.299 * r + 0.587 * g + 0.114 * b;
    let bright_r = (luma + (r - luma) * SATURATION) * BRIGHTNESS;
    let bright_g = (luma + (g - luma) * SATURATION) * BRIGHTNESS;
    let bright_b = (luma + (b - luma) * SATURATION) * BRIGHTNESS;

    (
        bright_r + (255.0 - bright_r) * WHITE_MIX,
        bright_g + (255.0 - bright_g) * WHITE_MIX,
        bright_b + (255.0 - bright_b) * WHITE_MIX,
    )
}

pub fn composite_with_mask(
    sharp: &[u8],
    blurred: &[u8],
    mask: &[u8],
    width: u32,
    height: u32,
) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;
    let mut out = vec![0u8; width * height * 4];

    for i in 0..width * height {
        let alpha = mask[i] as u32;
        let px = i * 4;

        if alpha >= 255 {
            out[px] = sharp[px];
            out[px + 1] = sharp[px + 1];
            out[px + 2] = sharp[px + 2];
        } else if alpha == 0 {
            let (tr, tg, tb) = glass_tint_pixel(
                blurred[px] as f32,
                blurred[px + 1] as f32,
                blurred[px + 2] as f32,
            );
            out[px] = tr.clamp(0.0, 255.0) as u8;
            out[px + 1] = tg.clamp(0.0, 255.0) as u8;
            out[px + 2] = tb.clamp(0.0, 255.0) as u8;
        } else {
            let (tr, tg, tb) = glass_tint_pixel(
                blurred[px] as f32,
                blurred[px + 1] as f32,
                blurred[px + 2] as f32,
            );
            out[px] = ((sharp[px] as u32 * alpha + tr.clamp(0.0, 255.0) as u32 * (255 - alpha)) / 255) as u8;
            out[px + 1] = ((sharp[px + 1] as u32 * alpha + tg.clamp(0.0, 255.0) as u32 * (255 - alpha)) / 255) as u8;
            out[px + 2] = ((sharp[px + 2] as u32 * alpha + tb.clamp(0.0, 255.0) as u32 * (255 - alpha)) / 255) as u8;
        }
        out[px + 3] = 255;
    }

    out
}
