import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

const host = process.env.TAURI_DEV_HOST;
const isMock = process.env.VITE_MOCK === 'true';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: {
    alias: isMock
      ? {
          '@tauri-apps/api/core': path.resolve('./src/lib/mock-tauri.ts'),
          '@tauri-apps/plugin-dialog': path.resolve('./src/lib/mock-dialog.ts'),
        }
      : {},
  },
  server: {
    port: 1421,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
  },
  optimizeDeps: {
    entries: ['index.html'],
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome105', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
