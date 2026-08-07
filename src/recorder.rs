use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    Blob, BlobEvent, Event, HtmlVideoElement, MediaRecorder,
    MediaRecorderOptions, MediaStream,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::{log, error};
use crate::codec;
use crate::download;

thread_local! {
    static STATE: Rc<RecorderState> = Rc::new(RecorderState::default());
}

#[derive(Default)]
struct RecorderState {
    recorder: RefCell<Option<MediaRecorder>>,
    stream: RefCell<Option<MediaStream>>,
    recording_stream: RefCell<Option<MediaStream>>,
    chunks: RefCell<Vec<JsValue>>,
    recording: RefCell<bool>,
    on_stop_closure: RefCell<Option<Closure<dyn FnMut(Event)>>>,
    on_data_closure: RefCell<Option<Closure<dyn FnMut(BlobEvent)>>>,
    on_error_closure: RefCell<Option<Closure<dyn FnMut(Event)>>>,
}

pub fn set_recording_stream(stream: MediaStream) {
    STATE.with(|state| state.recording_stream.borrow_mut().replace(stream));
}

pub fn init() -> Result<(), JsValue> {
    log("Initializing WASM video recorder...");
    codec::detect_best_codec();
    Ok(())
}

pub fn start() -> Result<(), JsValue> {
    STATE.with(|state| {
        if *state.recording.borrow() {
            error("Already recording.");
            return Ok(());
        }

        let stream = state
            .recording_stream
            .borrow()
            .clone()
            .ok_or("No recording stream set. Call set_recording_stream first.")?;

        state.stream.borrow_mut().replace(stream.clone());
        match create_recorder(state.clone(), &stream) {
            Ok(_) => log("Recorder created successfully."),
            Err(e) => error(&format!("Failed to create recorder: {:?}", e)),
        }
        Ok(())
    })
}

fn create_recorder(state: Rc<RecorderState>, stream: &MediaStream) -> Result<(), JsValue> {
    let options = MediaRecorderOptions::new();
    let mime = codec::best_mime_type();
    options.set_mime_type(&mime);
    let bitrate = codec::optimal_bitrate();
    options.set_video_bits_per_second(bitrate);
    options.set_audio_bits_per_second(128_000);

    let recorder = match MediaRecorder::new_with_media_stream_and_media_recorder_options(stream, &options) {
        Ok(r) => r,
        Err(e) => {
            error(&format!("MediaRecorder creation failed: {:?}", e));
            return Err(e);
        }
    };

    let data_closure = Closure::new(move |event: BlobEvent| {
        STATE.with(|s| {
            if let Some(blob) = event.data() {
                s.chunks.borrow_mut().push(blob.into());
            }
        });
    });
    recorder.set_ondataavailable(Some(data_closure.as_ref().unchecked_ref()));
    state.on_data_closure.borrow_mut().replace(data_closure);

    let stop_closure = Closure::new(move |_event: Event| {
        STATE.with(|s| {
            *s.recording.borrow_mut() = false;
            let chunks: Vec<JsValue> = s.chunks.borrow().iter().cloned().collect();
            let arr = js_sys::Array::new();
            for chunk in chunks.iter() {
                arr.push(chunk);
            }
            let result = Blob::new_with_blob_sequence(&arr)
                .and_then(|blob| {
                    web_sys::Url::create_object_url_with_blob(&blob)
                        .and_then(|url| {
                            download::store_blob_url(url.clone());
                            update_preview_and_download(&url, &blob)?;
                            Ok(())
                        })
                });
            if let Err(e) = result {
                error(&format!("Post-recording error: {:?}", e));
            }
            s.chunks.borrow_mut().clear();
            s.recorder.borrow_mut().take();
            s.stream.borrow_mut().take();
            log("Recording stopped and processed.");
        });
    });
    recorder.set_onstop(Some(stop_closure.as_ref().unchecked_ref()));
    state.on_stop_closure.borrow_mut().replace(stop_closure);

    let err_closure = Closure::new(move |event: Event| {
        error(&format!("MediaRecorder error: {:?}", event));
        STATE.with(|s| *s.recording.borrow_mut() = false);
    });
    recorder.set_onerror(Some(err_closure.as_ref().unchecked_ref()));
    state.on_error_closure.borrow_mut().replace(err_closure);

    recorder.start_with_time_slice(1000)?;
    *state.recording.borrow_mut() = true;
    *state.recorder.borrow_mut() = Some(recorder);
    Ok(())
}

fn update_preview_and_download(url: &str, blob: &Blob) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    if let Some(video) = document.get_element_by_id("preview")
        .and_then(|el| el.dyn_into::<HtmlVideoElement>().ok())
    {
        video.set_src_object(None);
        video.set_src(url);
    }

    let _size = blob.size();
    if let Some(info) = document.get_element_by_id("info") {
        info.set_inner_html("");
    }

    Ok(())
}

pub fn stop() -> Result<(), JsValue> {
    STATE.with(|state| {
        if let Some(recorder) = state.recorder.borrow().as_ref() {
            recorder.stop()?;
            *state.recording.borrow_mut() = false;
        }
        Ok(())
    })
}

pub fn is_recording() -> bool {
    STATE.with(|state| *state.recording.borrow())
}