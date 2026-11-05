import { invoke } from '@tauri-apps/api/core';

import type {
  GameView,
  NewGameOptions,
  ReplayFrame,
} from '@shared/contracts/generated/bindings';

export const startGame = (options: NewGameOptions) =>
  invoke<GameView>('new_game', { options });

export const suspendGame = () => invoke<void>('suspend_game');

export const resumeGame = (id: number) =>
  invoke<GameView>('resume_game', { id });

export const fetchCurrentGame = () => invoke<GameView | null>('current_game');

export const fetchGameFrames = () => invoke<ReplayFrame[]>('game_frames');

export const playMove = (uci: string) => invoke<GameView>('make_move', { uci });

export const storePremove = (uci: string) =>
  invoke<GameView>('set_premove', { uci });

export const dropPremove = () => invoke<GameView>('clear_premove');

export const takeBackMove = () => invoke<GameView>('take_back');

export const resignGame = () => invoke<GameView>('resign');

export const offerDraw = () => invoke<boolean>('offer_draw');

export const analysePosition = (fen: string | null) =>
  invoke<void>('set_analysis_position', { fen });
