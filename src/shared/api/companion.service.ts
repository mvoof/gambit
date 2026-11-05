import { invoke } from '@tauri-apps/api/core';

import type { CompanionView } from '@shared/contracts/generated/bindings';

export const fetchLlmModels = (baseUrl: string) =>
  invoke<string[]>('llm_models', { baseUrl });

export const fetchCompanionView = () => invoke<CompanionView>('companion_view');

export const askCompanion = (text: string, ply: number | null) =>
  invoke<void>('companion_ask', { text, ply });

export const requestAdvice = () => invoke<void>('companion_advise');

export const stopCompanion = () => invoke<void>('companion_stop');
