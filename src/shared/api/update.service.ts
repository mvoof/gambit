import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';

export type AppUpdate = Update;
export type UpdateDownloadEvent = DownloadEvent;

export const checkForUpdate = () => check();

export const relaunchApp = () => relaunch();
