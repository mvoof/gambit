import { invoke } from '@tauri-apps/api/core';

import type { EngineInfo } from '@shared/contracts/generated/bindings';

export const fetchEngineStatus = () =>
  invoke<EngineInfo | null>('engine_status');

export const downloadStockfish = () => invoke<EngineInfo>('download_stockfish');

export const chooseEngine = (path: string) =>
  invoke<EngineInfo>('choose_engine', { path });
