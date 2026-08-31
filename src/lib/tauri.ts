import { invoke } from '@tauri-apps/api/core';

/**
 * Every Tauri command the frontend may call is wrapped once here, so the rest
 * of the app never talks to `invoke` directly and the argument/return types
 * live in one place.
 */
export interface AppInfo {
  name: string;
  version: string;
  tauriVersion: string;
}

export const greet = (name: string) => invoke<string>('greet', { name });

export const getAppInfo = () => invoke<AppInfo>('get_app_info');
