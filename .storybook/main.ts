import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { StorybookConfig } from '@storybook/react-vite';
import svgr from 'vite-plugin-svgr';

import { SCSS_ADDITIONAL_DATA } from '../scss.config.ts';

// This config is ESM, so `__dirname` does not exist here.
const currentDir = dirname(fileURLToPath(import.meta.url));

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: ['@storybook/addon-docs', '@storybook/addon-a11y'],
  framework: '@storybook/react-vite',
  viteFinal: (config) => {
    config.plugins = [
      ...(config.plugins ?? []),
      svgr({ svgrOptions: { icon: true } }),
    ];

    config.resolve = config.resolve ?? {};
    config.resolve.alias = {
      // No Tauri runtime in a browser — the backend calls are stubbed.
      '@tauri-apps/api/core': resolve(currentDir, '../src/mocks/tauri-core.ts'),
      '@': resolve(currentDir, '../src'),
    };

    config.css = config.css ?? {};
    config.css.preprocessorOptions = {
      scss: {
        additionalData: SCSS_ADDITIONAL_DATA,
      },
    };

    return config;
  },
};

export default config;
