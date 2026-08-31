/**
 * Injected into every SCSS module, so a component never imports the token files
 * by hand. Shared by vite.config.ts and .storybook/main.ts, so the app and the
 * stories can never drift apart.
 */
export const SCSS_ADDITIONAL_DATA = `
  @use "@/styles/functions" as *;
  @use "@/styles/variables" as *;
  @use "@/styles/tokens" as *;
`;
