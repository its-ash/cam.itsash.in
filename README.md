# Rust WebAssembly Video Recorder

Record video directly in the browser and download it with optimal compression using Rust + WebAssembly.

## Features

- **In-browser recording** via `getUserMedia` (camera + microphone)
- **Best-in-class codecs** — auto-detects and prioritizes AV1 → VP9 → H.264
- **Optimal bitrates** — tuned per-codec (AV1: 2.5 Mbps, VP9: 3 Mbps, H.264: 4 Mbps)
- **Codec selector** — manually pick codec from supported list
- **Instant preview** — recorded video plays back immediately
- **One-click download** — save recording with auto-generated filename
- **Release-optimized WASM** — `opt-level="z"`, LTO, single codegen unit

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core logic | Rust + `wasm-bindgen` + `web-sys` |
| Capture / compression | MediaRecorder API (WebCodecs-compatible) |
| Frontend | HTML, CSS, vanilla JS (ES modules) |
| Dev server / bundler | Vite + `vite-plugin-wasm` |
| WASM build | `wasm-pack` (`--target web`) |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- `wasm-pack` — `cargo install wasm-pack`
- `wasm32-unknown-unknown` target — `rustup target add wasm32-unknown-unknown`
- [Node.js](https://nodejs.org/) 18+

## Quick Start

```bash
# 1. Build the Rust → WASM artifact
wasm-pack build --target web --out-dir pkg

# 2. Install JS dependencies
npm install

# 3. Start the dev server
npm run dev
```

Open the printed URL (default <http://localhost:5173>) in a browser. Allow camera/mic access, then click **Start Recording**.

## Usage

1. **Select codec** (optional) — defaults to best available; use the dropdown to override.
2. **Start Recording** — captures video + audio from your camera.
3. **Stop** — finalizes the recording; preview appears automatically.
4. **Download** — saves the file (`.webm` or `.mp4` depending on codec).

## Project Structure

```
record/
├── src/
│   ├── lib.rs         # WASM entry point + exported functions
│   ├── recorder.rs    # MediaRecorder lifecycle (start/stop/chunks)
│   ├── codec.rs       # Codec detection, priority, bitrate tuning
│   ├── download.rs    # Blob URL management + download trigger
│   └── utils.rs      # Panic hook + logging helpers
├── www/
│   ├── index.html     # UI shell
│   ├── main.js        # Frontend logic (imports WASM module)
│   └── styles.css     # Dark-theme styling
├── pkg/               # Generated WASM output (wasm-pack)
├── Cargo.toml         # Rust crate manifest
├── package.json       # JS scripts + dev deps
└── vite.config.mjs    # Vite config (WASM plugin, root=www)
```

## Codec Priority

| Priority | MIME type | Codec | Bitrate |
|----------|----------|-------|---------|
| 1 | `video/webm;codecs=av01.0.05M.08` | AV1 8-bit | 2.5 Mbps |
| 2 | `video/webm;codecs=av01.0.08M.08` | AV1 10-bit | 2.5 Mbps |
| 3 | `video/webm;codecs=vp9` | VP9 | 3 Mbps |
| 4 | `video/webm;codecs=vp8` | VP8 | 4 Mbps |
| 5 | `video/mp4;codecs=avc1.640028` | H.264 (AVC) | 4 Mbps |
| 6 | `video/webm` | WebM default | 5 Mbps |
| 7 | `video/mp4` | MP4 default | 5 Mbps |

AV1 is preferred when available (Chrome 85+, Firefox 130+) for the best quality/size ratio. Falls back gracefully to VP9, then H.264.

## Build Commands

| Command | Description |
|---------|-------------|
| `wasm-pack build --target web --out-dir pkg` | Compile Rust → WASM |
| `npm install` | Install JS dependencies |
| `npm run dev` | Start Vite dev server |
| `npm run build` | Production build → `dist/` |
| `npm run preview` | Preview production build |
| `npm run wasm` | Alias for `wasm-pack build` |

## Browser Support

| Browser | AV1 | VP9 | H.264 |
|---------|-----|-----|-------|
| Chrome 85+ | ✅ | ✅ | ✅ |
| Firefox 130+ | ✅ | ✅ | ✅ |
| Safari 17+ | ❌ | ❌ | ✅ |

Recording requires `https://` or `localhost`. Camera/mic permission must be granted.

## License

MIT