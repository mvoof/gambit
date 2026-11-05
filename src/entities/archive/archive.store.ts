import { makeAutoObservable, runInAction } from 'mobx';

import {
  deleteGame,
  exportGame,
  fetchGames,
  fetchReplay,
  importGame,
  setReviewGame,
} from '@shared/api/archive.service';
import { onGameReview } from '@shared/api/events.service';
import { pickFile, pickSavePath } from '@shared/api/system.service';
import type {
  GameReview,
  GameSummary,
  MoveMark,
  ReplayGame,
} from '@shared/contracts/generated/bindings';

const PGN_EXTENSIONS = ['pgn'];
const AUTOPLAY_INTERVAL_MS = 900;
const NO_MARKS: MoveMark[] = [];

export class ArchiveStore {
  games: GameSummary[] = [];
  replay: ReplayGame | null = null;
  replayId: number | null = null;
  frameIndex = 0;
  isAutoplaying = false;
  review: GameReview | null = null;
  error: string | null = null;

  private autoplayTimer: ReturnType<typeof setInterval> | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  start = async () => {
    await onGameReview(this.receiveReview);
  };

  private receiveReview = (review: GameReview) => {
    this.review = review;
  };

  private attempt = async (action: () => Promise<void>) => {
    try {
      await action();
      runInAction(() => {
        this.error = null;
      });
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }
  };

  refresh = () =>
    this.attempt(async () => {
      const games = await fetchGames();

      runInAction(() => {
        this.games = games;
      });
    });

  open = (id: number) =>
    this.attempt(async () => {
      const replay = await fetchReplay(id);

      runInAction(() => {
        this.stopAutoplay();
        this.replay = replay;
        this.replayId = id;
        this.frameIndex = 0;
      });
    });

  close = () => {
    this.stopAutoplay();
    this.replay = null;
    this.replayId = null;
  };

  /** Measuring follows the replay on screen; `null` stops it. */
  setReviewTarget = (id: number | null) =>
    this.attempt(() => setReviewGame(id));

  remove = (id: number) =>
    this.attempt(async () => {
      await deleteGame(id);

      if (this.replayId === id) {
        runInAction(this.close);
      }

      await this.refresh();
    });

  importFromFile = () =>
    this.attempt(async () => {
      const path = await pickFile(PGN_EXTENSIONS);

      if (path) {
        const games = await importGame(path);

        runInAction(() => {
          this.games = games;
        });
      }
    });

  exportToFile = (id: number) =>
    this.attempt(async () => {
      const path = await pickSavePath(`game-${id}.pgn`, PGN_EXTENSIONS);

      if (path) {
        await exportGame(id, path);
      }
    });

  get frame() {
    return this.replay?.frames[this.frameIndex] ?? null;
  }

  /** The last report may still be about the game opened before this one. */
  get activeReview() {
    return this.review?.gameId === this.replayId ? this.review : null;
  }

  get marks() {
    return this.activeReview?.marks ?? NO_MARKS;
  }

  get frameMark() {
    return this.marks.find((mark) => mark.ply === this.frameIndex) ?? null;
  }

  get lastFrameIndex() {
    return (this.replay?.frames.length ?? 1) - 1;
  }

  goTo = (index: number) => {
    this.frameIndex = Math.min(Math.max(0, index), this.lastFrameIndex);
  };

  step = (offset: number) => {
    this.goTo(this.frameIndex + offset);
  };

  private advance = () => {
    if (this.frameIndex >= this.lastFrameIndex) {
      this.stopAutoplay();
    } else {
      this.step(1);
    }
  };

  startAutoplay = () => {
    this.stopAutoplay();
    this.isAutoplaying = true;
    this.autoplayTimer = setInterval(this.advance, AUTOPLAY_INTERVAL_MS);
  };

  stopAutoplay = () => {
    if (this.autoplayTimer !== null) {
      clearInterval(this.autoplayTimer);
    }

    this.autoplayTimer = null;
    this.isAutoplaying = false;
  };

  toggleAutoplay = () => {
    if (this.isAutoplaying) {
      this.stopAutoplay();
    } else {
      this.startAutoplay();
    }
  };
}
