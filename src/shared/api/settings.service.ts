import { invoke } from '@tauri-apps/api/core';

import type { AppSettings } from '@shared/contracts/generated/bindings';

export const fetchSettings = () => invoke<AppSettings>('get_settings');

export const saveSettings = (settings: AppSettings) =>
  invoke<AppSettings>('save_app_settings', { settings });
