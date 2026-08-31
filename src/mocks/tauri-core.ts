/**
 * Storybook runs in a plain browser with no Tauri runtime, so `invoke` is
 * swapped for this stub through the alias in `.storybook/main.ts`.
 */
interface MockedArgs {
  name?: string;
}

export const invoke = async (command: string, args?: MockedArgs) => {
  if (command === 'greet') {
    return `Hello, ${args?.name ?? 'world'}! (mocked)`;
  }

  return undefined;
};
