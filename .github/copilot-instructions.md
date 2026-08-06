# Project: Rust WebAssembly Video Recorder

## Overview
A Rust + WebAssembly project that lets users record video in the browser and download it, using best-in-class video optimization and compression algorithms (AV1/VP9/H.264 via WebCodecs + MediaRecorder).

## Tech Stack
- **Rust** + `wasm-bindgen` + `web-sys` for browser interop
- **WebCodecs API** / **MediaRecorder API** for capture + compression
- **Vite** + **vite-plugin-wasm** for dev server + bundling
- `wasm-pack` for building the Rust → WASM artifact

## Build Commands
- Install Rust target: `rustup target add wasm32-unknown-unknown`
- Build WASM: `wasm-pack build --target web --out-dir pkg`
- Install JS deps: `npm install`
- Dev server: `npm run dev`
- Production build: `npm run build`

## Project Structure
- `src/` — Rust source (video recording, compression logic)
- `www/` — Frontend (HTML, JS, CSS)
- `pkg/` — wasm-pack output (generated)
- `vite.config.mjs` — Vite config with WASM plugin

## Conventions
- Use `wasm-bindgen` `#[wasm_bindgen]` attributes for exported functions.
- Prefer `web-sys` types over manual JS interop.
- All exported Rust functions must be panic-safe (use `Result` returns).
- Frontend JS uses ES modules, no bundler-specific syntax.