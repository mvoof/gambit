import path from 'node:path';

interface AliasEntry {
  find: string;
  replacement: string;
}

const fromRoot = (relativePath: string) =>
  path.resolve(import.meta.dirname, relativePath);

/**
 * Longest prefix first: Vite matches string aliases by prefix, so the layers
 * come before the catch-all `@`. Shared by vite.config.ts and .storybook/main.ts.
 */
export const createLayerAliases = (): AliasEntry[] => [
  { find: '@app', replacement: fromRoot('./src/app') },
  { find: '@pages', replacement: fromRoot('./src/pages') },
  { find: '@widgets', replacement: fromRoot('./src/widgets') },
  { find: '@features', replacement: fromRoot('./src/features') },
  { find: '@entities', replacement: fromRoot('./src/entities') },
  { find: '@shared', replacement: fromRoot('./src/shared') },
  { find: '@assets', replacement: fromRoot('./src/assets') },
  { find: '@', replacement: fromRoot('./src') },
];

const MOCKED_TAURI_MODULES: Record<string, string> = {
  '@tauri-apps/api/core': 'tauri-core',
  '@tauri-apps/api/event': 'tauri-event',
};

/** Storybook has no Tauri runtime, so the backend modules are stubbed first. */
export const createStorybookAliases = (): AliasEntry[] => [
  ...Object.entries(MOCKED_TAURI_MODULES).map(([moduleId, mockName]) => ({
    find: moduleId,
    replacement: fromRoot(`./src/storybook/mocks/${mockName}.ts`),
  })),
  ...createLayerAliases(),
];
