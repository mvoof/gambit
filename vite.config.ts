/// <reference types="vitest/config" />
import { resolve } from 'node:path';

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import svgr from 'vite-plugin-svgr';

import { SCSS_ADDITIONAL_DATA } from './scss.config';

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    react(),
    svgr({
      svgrOptions: {
        icon: true,
      },
    }),
  ],
  base: './',

  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },

  css: {
    preprocessorOptions: {
      scss: {
        additionalData: SCSS_ADDITIONAL_DATA,
      },
    },
  },

  test: {
    environment: 'node',
    include: ['src/**/*.test.{ts,tsx}'],
  },

  // Vite options tailored for Tauri development and only applied in
  // `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: {
      protocol: 'ws',
      host,
      port: 1421,
      overlay: false,
    },
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}));
