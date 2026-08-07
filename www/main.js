import init, {
  init_recorder,
  start_recording,
  stop_recording,
  is_recording,
  download_recording,
} from '../pkg/wasm_video_recorder.js';

async function main() {
  await init();
  init_recorder();

  const startBtn = document.getElementById('start-btn');
  const stopBtn = document.getElementById('stop-btn');
  const downloadBtn = document.getElementById('download-btn');
  const statusEl = document.getElementById('status');
  const timerEl = document.getElementById('timer');
  const video = document.getElementById('preview');

  stopBtn.style.display = 'none';
  downloadBtn.style.display = 'none';
  timerEl.style.display = 'none';

  let timerInterval = null;
  let startTime = 0;

  function startTimer() {
    startTime = Date.now();
    timerEl.style.display = 'block';
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
    setTimeout(() => { timerEl.style.display = 'none'; }, 2000);
  }

  function setStatus(msg) {
    statusEl.textContent = msg;
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
    video.srcObject = stream;
    await video.play();
    const track = stream.getVideoTracks()[0];
    const settings = track.getSettings();
    setStatus(`Camera: ${settings.width}x${settings.height}@${settings.frameRate}fps`);
  } catch (e) {
    setStatus(`Camera error: ${e}`);
  }

  startBtn.addEventListener('click', async () => {
    try {
      await start_recording();
      startBtn.style.display = 'none';
      stopBtn.style.display = 'flex';
      stopBtn.disabled = false;
      downloadBtn.style.display = 'none';
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
      stopTimer();
      setStatus('Ready');
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

  window.addEventListener('beforeunload', async () => {
    if (is_recording()) {
      try { await stop_recording(); } catch (_) {}
    }
  });
}

main();