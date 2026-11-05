import { invoke } from '@tauri-apps/api/core';

import type {
  GameSummary,
  ReplayGame,
} from '@shared/contracts/generated/bindings';

export const fetchGames = () => invoke<GameSummary[]>('list_games');

export const fetchReplay = (id: number) =>
  invoke<ReplayGame>('load_replay', { id });

export const setReviewGame = (id: number | null) =>
  invoke<void>('set_review_game', { id });

export const deleteGame = (id: number) => invoke<void>('delete_game', { id });

export const exportGame = (id: number, path: string) =>
  invoke<void>('export_game', { id, path });

export const importGame = (path: string) =>
  invoke<GameSummary[]>('import_game', { path });
