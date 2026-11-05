import { listen } from '@tauri-apps/api/event';

import {
  EVENT_ENGINE_INSTALL,
  EVENT_EVALUATION,
  EVENT_GAME_REVIEW,
  EVENT_GAME_UPDATED,
} from '@shared/contracts/generated/bindings';
import type {
  EngineInstallProgress,
  Evaluation,
  GameReview,
  GameView,
} from '@shared/contracts/generated/bindings';

type Unlisten = () => void;

const subscribe = <Payload>(
  eventName: string,
  handler: (payload: Payload) => void
): Promise<Unlisten> =>
  listen<Payload>(eventName, (event) => {
    handler(event.payload);
  });

export const onGameUpdated = (handler: (view: GameView) => void) =>
  subscribe(EVENT_GAME_UPDATED, handler);

export const onEvaluation = (handler: (evaluation: Evaluation) => void) =>
  subscribe(EVENT_EVALUATION, handler);

export const onGameReview = (handler: (review: GameReview) => void) =>
  subscribe(EVENT_GAME_REVIEW, handler);

export const onEngineInstall = (
  handler: (progress: EngineInstallProgress) => void
) => subscribe(EVENT_ENGINE_INSTALL, handler);
