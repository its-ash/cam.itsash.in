import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
  root: 'www',
  server: {
    port: 5173,
    open: true,
  },
  build: {
    outDir: '../docs',
    emptyOutDir: true,
  },
});