import { makeAutoObservable, runInAction } from 'mobx';

import {
  dropPremove,
  fetchCurrentGame,
  fetchGameFrames,
  offerDraw,
  playMove,
  resignGame,
  startGame,
  storePremove,
  takeBackMove,
} from '@shared/api/game.service';
import { onGameUpdated } from '@shared/api/events.service';
import type {
  GameView,
  NewGameOptions,
  ReplayFrame,
} from '@shared/contracts/generated/bindings';

export class GameStore {
  view: GameView | null = null;
  frames: ReplayFrame[] = [];
  /** The ply being looked at, `null` for the live position. */
  browsedPly: number | null = null;
  isOrientationFlipped = false;
  drawDeclined = false;
  error: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  start = async () => {
    await onGameUpdated(this.receive);
    this.receive(await fetchCurrentGame());
  };

  private receive = (view: GameView | null) => {
    this.view = view;
    this.browsedPly = null;
  };

  private attempt = async (action: () => Promise<GameView>) => {
    try {
      const view = await action();

      runInAction(() => {
        this.error = null;
        this.receive(view);
      });
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }
  };

  get browsedFrame() {
    return this.browsedPly === null
      ? null
      : (this.frames[this.browsedPly] ?? null);
  }

  /** The position on the board, live or browsed. */
  get displayedFen() {
    return this.browsedFrame?.fen ?? this.view?.fen ?? null;
  }

  get orientation() {
    const humanColor = this.view?.humanColor ?? 'white';
    const opposite = humanColor === 'white' ? 'black' : 'white';

    return this.isOrientationFlipped ? opposite : humanColor;
  }

  get isPlaying() {
    return this.view !== null && this.view.outcome === null;
  }

  flip = () => {
    this.isOrientationFlipped = !this.isOrientationFlipped;
  };

  newGame = async (options: NewGameOptions) => {
    this.drawDeclined = false;
    this.isOrientationFlipped = false;
    await this.attempt(() => startGame(options));
  };

  move = (uci: string) => this.attempt(() => playMove(uci));

  setPremove = (uci: string) => this.attempt(() => storePremove(uci));

  cancelPremove = () => this.attempt(dropPremove);

  takeBack = () => this.attempt(takeBackMove);

  resign = () => this.attempt(resignGame);

  offerDraw = async () => {
    try {
      const isAccepted = await offerDraw();

      runInAction(() => {
        this.drawDeclined = !isAccepted;
      });
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }
  };

  /** `ply` 0 is the start; the last ply is the live position. */
  browseTo = async (ply: number) => {
    const moveCount = this.view?.moves.length ?? 0;

    if (ply < 0 || ply > moveCount) {
      return;
    }

    if (ply === moveCount) {
      this.browsedPly = null;

      return;
    }

    const frames = await fetchGameFrames();

    runInAction(() => {
      this.frames = frames;
      this.browsedPly = ply;
    });
  };

  browseBy = (step: number) => {
    const moveCount = this.view?.moves.length ?? 0;

    return this.browseTo((this.browsedPly ?? moveCount) + step);
  };
}
