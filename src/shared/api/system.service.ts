import { open, save } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';

const singlePath = (selection: string | string[] | null) =>
  typeof selection === 'string' ? selection : null;

export const pickFile = async (extensions?: string[]) =>
  singlePath(
    await open({
      multiple: false,
      filters: extensions ? [{ name: '', extensions }] : undefined,
    })
  );

export const pickFolder = async () =>
  singlePath(await open({ multiple: false, directory: true }));

export const pickSavePath = (defaultPath: string, extensions: string[]) =>
  save({ defaultPath, filters: [{ name: '', extensions }] });

export const openExternal = (url: string) => openUrl(url);
