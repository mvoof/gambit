import { invoke } from '@tauri-apps/api/core';

import type {
  CatalogEntry,
  EngineInfo,
} from '@shared/contracts/generated/bindings';

export const fetchEngineStatus = () =>
  invoke<EngineInfo | null>('engine_status');

export const fetchEngineCatalog = () =>
  invoke<CatalogEntry[]>('engine_catalog');

export const installEngine = (id: string) =>
  invoke<EngineInfo>('install_engine', { id });

export const removeEngine = (id: string) =>
  invoke<null>('remove_engine', { id });

export const chooseEngine = (path: string) =>
  invoke<EngineInfo>('choose_engine', { path });
