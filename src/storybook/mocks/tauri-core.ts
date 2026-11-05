export const invoke = async (): Promise<never> => {
  throw new Error('No Tauri runtime in Storybook');
};
