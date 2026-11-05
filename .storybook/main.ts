import type { StorybookConfig } from '@storybook/react-vite';
import svgr from 'vite-plugin-svgr';

import { SCSS_ADDITIONAL_DATA } from '../scss.config.ts';
import { createStorybookAliases } from '../vite.aliases.ts';

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
    config.resolve.alias = createStorybookAliases();

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
