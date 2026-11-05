import { getCurrentWindow } from '@tauri-apps/api/window';

export const minimizeWindow = () => getCurrentWindow().minimize();

export const toggleMaximizeWindow = () => getCurrentWindow().toggleMaximize();

export const closeWindow = () => getCurrentWindow().close();

export const isWindowMaximized = () => getCurrentWindow().isMaximized();

export const onWindowResized = (handler: () => void) =>
  getCurrentWindow().onResized(handler);
