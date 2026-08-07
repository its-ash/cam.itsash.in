use std::cell::RefCell;
use wasm_bindgen::JsValue;
use web_sys::MediaRecorder;
use crate::utils::log;

thread_local! {
    static PREFERENCE: RefCell<Option<String>> = RefCell::new(None);
    static DETECTED_MIME: RefCell<String> = RefCell::new(String::new());
    static SUPPORTED: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

const CODEC_PRIORITY: &[&str] = &[
    "video/mp4;codecs=avc1.640034",
    "video/mp4;codecs=avc1.640028",
    "video/mp4;codecs=avc1.4d4034",
    "video/mp4;codecs=h264",
    "video/mp4;codecs=avc1.42E01E",
    "video/mp4",
];

const CODEC_LABELS: &[(&str, &str)] = &[
    ("video/mp4;codecs=avc1.640034", "H.264 (AVC High 5.1)"),
    ("video/mp4;codecs=avc1.640028", "H.264 (AVC High 5.0)"),
    ("video/mp4;codecs=avc1.4d4034", "H.264 (AVC Main 5.1)"),
    ("video/mp4;codecs=h264", "H.264 (base)"),
    ("video/mp4;codecs=avc1.42E01E", "H.264 (AVC Baseline)"),
    ("video/mp4", "MP4 (default)"),
];

pub fn detect_best_codec() {
    let mut supported = Vec::new();
    let mut best_mime = String::new();

    for &mime in CODEC_PRIORITY {
        if MediaRecorder::is_type_supported(mime) {
            supported.push(mime.to_string());
            if best_mime.is_empty() {
                best_mime = mime.to_string();
            }
        }
    }

    if best_mime.is_empty() {
        best_mime = "video/webm".to_string();
        supported.push(best_mime.clone());
    }

    SUPPORTED.with(|s| *s.borrow_mut() = supported);
    DETECTED_MIME.with(|m| *m.borrow_mut() = best_mime.clone());
    log(&format!("Best detected codec: {}", best_mime));
}

pub fn best_mime_type() -> String {
    PREFERENCE.with(|p| {
        if let Some(pref) = p.borrow().as_ref() {
            if MediaRecorder::is_type_supported(pref) {
                return pref.clone();
            }
        }
        DETECTED_MIME.with(|m| m.borrow().clone())
    })
}

pub fn optimal_bitrate() -> u32 {
    let mime = best_mime_type();
    if mime.contains("av01") {
        2_500_000
    } else if mime.contains("vp9") {
        3_000_000
    } else if mime.contains("h264") || mime.contains("avc1") {
        10_000_000
    } else if mime.contains("vp8") {
        4_000_000
    } else {
        6_000_000
    }
}

pub fn compressed_bitrate() -> u32 {
    let mime = best_mime_type();
    if mime.contains("av01") {
        1_000_000
    } else if mime.contains("vp9") {
        1_200_000
    } else if mime.contains("h264") || mime.contains("avc1") {
        2_500_000
    } else if mime.contains("vp8") {
        1_500_000
    } else {
        2_000_000
    }
}

pub fn selected_codec() -> String {
    best_mime_type()
}

pub fn set_preference(codec: &str) -> Result<(), JsValue> {
    if !MediaRecorder::is_type_supported(codec) {
        return Err(JsValue::from_str(&format!("Unsupported codec: {}", codec)));
    }
    PREFERENCE.with(|p| *p.borrow_mut() = Some(codec.to_string()));
    log(&format!("Codec preference set to: {}", codec));
    Ok(())
}

pub fn list_supported() -> Vec<String> {
    SUPPORTED.with(|s| s.borrow().clone())
}

pub fn codec_label(mime: &str) -> &'static str {
    for &(m, label) in CODEC_LABELS {
        if m == mime {
            return label;
        }
    }
    "Unknown"
}