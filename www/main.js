import init, {
  init_recorder,
  set_recording_stream,
  start_recording,
  stop_recording,
  is_recording,
  download_recording,
  discard_recording,
  init_segmentation,
  segment_mask,
  apply_background_blur_full,
} from '../pkg/wasm_video_recorder.js';

async function main() {
  await init();
  init_recorder();

  const startBtn = document.getElementById('start-btn');
  const stopBtn = document.getElementById('stop-btn');
  const downloadBtn = document.getElementById('download-btn');
  const discardBtn = document.getElementById('discard-btn');
  const blurBtn = document.getElementById('blur-btn');
  const statusEl = document.getElementById('status');
  const timerEl = document.getElementById('timer');
  const video = document.getElementById('preview');
  const outputCanvas = document.getElementById('output');
  const outputCtx = outputCanvas.getContext('2d');

  stopBtn.style.display = 'none';
  downloadBtn.style.display = 'none';
  discardBtn.style.display = 'none';
  timerEl.style.display = 'none';

  let timerInterval = null;
  let timerHideTimeout = null;
  let startTime = 0;

  function startTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    if (timerHideTimeout) {
      clearTimeout(timerHideTimeout);
      timerHideTimeout = null;
    }
    timerEl.textContent = '00:00';
    startTime = Date.now();
    timerEl.style.display = 'flex';
    void timerEl.offsetWidth;
    timerEl.classList.add('active');
    timerInterval = setInterval(() => {
      const elapsed = Math.floor((Date.now() - startTime) / 1000);
      const mm = String(Math.floor(elapsed / 60)).padStart(2, '0');
      const ss = String(elapsed % 60).padStart(2, '0');
      timerEl.textContent = `${mm}:${ss}`;
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    timerEl.classList.remove('active');
    if (timerHideTimeout) {
      clearTimeout(timerHideTimeout);
    }
    timerHideTimeout = setTimeout(() => {
      timerEl.style.display = 'none';
      timerHideTimeout = null;
    }, 2000);
  }

  function resetTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    if (timerHideTimeout) {
      clearTimeout(timerHideTimeout);
      timerHideTimeout = null;
    }
    timerEl.classList.remove('active');
    timerEl.style.display = 'none';
    timerEl.textContent = '00:00';
  }

  function setStatus(msg) {
    statusEl.textContent = msg;
  }

  let blurActive = false;
  let renderLoop = null;
  let segReady = false;
  let playingBack = false;
  const segDim = 256;
  const workDim = 640;
  const segCanvas = document.createElement('canvas');
  const segCtx = segCanvas.getContext('2d', { willReadFrequently: true });
  segCanvas.width = segDim;
  segCanvas.height = segDim;
  const workCanvas = document.createElement('canvas');
  const workCtx = workCanvas.getContext('2d', { willReadFrequently: true });

  function renderTick() {
    if (playingBack) return;

    const w = video.videoWidth;
    const h = video.videoHeight;
    if (w === 0 || h === 0) {
      renderLoop = requestAnimationFrame(renderTick);
      return;
    }
    if (outputCanvas.width !== w || outputCanvas.height !== h) {
      outputCanvas.width = w;
      outputCanvas.height = h;
    }

    if (blurActive && segReady) {
      try {
        const scale = Math.min(1, workDim / Math.max(w, h));
        const ww = Math.max(1, Math.round(w * scale));
        const wh = Math.max(1, Math.round(h * scale));
        if (workCanvas.width !== ww || workCanvas.height !== wh) {
          workCanvas.width = ww;
          workCanvas.height = wh;
        }

        segCtx.drawImage(video, 0, 0, segDim, segDim);
        const segFrame = segCtx.getImageData(0, 0, segDim, segDim);
        const mask = segment_mask(segFrame.data, segDim, segDim);

        workCtx.drawImage(video, 0, 0, ww, wh);
        const workFrame = workCtx.getImageData(0, 0, ww, wh);
        const result = apply_background_blur_full(
          workFrame.data, ww, wh,
          mask, segDim, segDim,
          8
        );
        workCtx.putImageData(new ImageData(new Uint8ClampedArray(result), ww, wh), 0, 0);

        outputCtx.imageSmoothingEnabled = true;
        outputCtx.drawImage(workCanvas, 0, 0, ww, wh, 0, 0, w, h);
      } catch (err) {
        console.error('Blur error:', err);
        outputCtx.drawImage(video, 0, 0, w, h);
      }
    } else {
      outputCtx.drawImage(video, 0, 0, w, h);
    }

    renderLoop = requestAnimationFrame(renderTick);
  }

  function toggleBlur() {
    blurActive = !blurActive;
    blurBtn.classList.toggle('active', blurActive);
  }

  blurBtn.addEventListener('click', toggleBlur);

  let micTrack = null;
  let cameraStream = null;

  function resumeLiveView() {
    playingBack = false;
    video.srcObject = cameraStream;
    video.play().catch(() => {});
    outputCanvas.style.display = 'block';
    if (!renderLoop) {
      renderLoop = requestAnimationFrame(renderTick);
    }
  }

  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      video: {
        width: { ideal: 1920 },
        height: { ideal: 1080 },
        frameRate: { ideal: 60 },
        facingMode: 'user',
      },
      audio: true,
    });
    cameraStream = stream;
    video.srcObject = stream;
    await video.play();
    const track = stream.getVideoTracks()[0];
    const settings = track.getSettings();
    setStatus(`Camera: ${settings.width}x${settings.height}@${settings.frameRate}fps`);
    micTrack = stream.getAudioTracks()[0] || null;

    try {
      init_segmentation();
      segReady = true;
    } catch (err) {
      console.error('Segmentation init failed:', err);
    }

    outputCanvas.style.display = 'block';
    renderLoop = requestAnimationFrame(renderTick);
  } catch (e) {
    setStatus(`Camera error: ${e}`);
  }

  startBtn.addEventListener('click', async () => {
    try {
      if (playingBack) {
        resumeLiveView();
      }

      const canvasStream = outputCanvas.captureStream(30);
      if (micTrack) {
        canvasStream.addTrack(micTrack);
      }
      set_recording_stream(canvasStream);

      await start_recording();
      startBtn.style.display = 'none';
      stopBtn.style.display = 'flex';
      stopBtn.disabled = false;
      downloadBtn.style.display = 'none';
      downloadBtn.disabled = true;
      discardBtn.style.display = 'none';
      discardBtn.disabled = true;
      startTimer();
      setStatus('● REC');
      statusEl.classList.add('recording');
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  });

  stopBtn.addEventListener('click', async () => {
    try {
      await stop_recording();
      stopBtn.style.display = 'none';
      startBtn.style.display = 'flex';
      downloadBtn.style.display = 'flex';
      downloadBtn.disabled = false;
      discardBtn.style.display = 'flex';
      discardBtn.disabled = false;
      stopTimer();
      setStatus('Ready');

      playingBack = true;
      if (renderLoop) {
        cancelAnimationFrame(renderLoop);
        renderLoop = null;
      }
      outputCanvas.style.display = 'none';
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  });

  downloadBtn.addEventListener('click', async () => {
    try {
      await download_recording(null);
    } catch (e) {
      setStatus(`Download error: ${e}`);
    }
  });

  discardBtn.addEventListener('click', () => {
    discard_recording();
    downloadBtn.style.display = 'none';
    downloadBtn.disabled = true;
    discardBtn.style.display = 'none';
    discardBtn.disabled = true;
    resetTimer();
    setStatus('Discarded');
    resumeLiveView();
  });

  window.addEventListener('beforeunload', async () => {
    if (is_recording()) {
      try { await stop_recording(); } catch (_) {}
    }
  });
}

main();
