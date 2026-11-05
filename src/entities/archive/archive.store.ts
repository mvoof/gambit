import { makeAutoObservable, runInAction } from 'mobx';

import {
  deleteGame,
  exportGame,
  fetchArchiveStats,
  fetchGames,
  fetchReplay,
  importGame,
  setGameFavorite,
  setReviewGame,
} from '@shared/api/archive.service';
import { onGameReview } from '@shared/api/events.service';
import { pickFile, pickSavePath } from '@shared/api/system.service';
import type {
  ArchiveStats,
  Color,
  EvalPoint,
  GameReview,
  GameSummary,
  MoveMark,
  ReplayGame,
} from '@shared/contracts/generated/bindings';

import type { ArchiveFilter } from './select-games';

const PGN_EXTENSIONS = ['pgn'];
const AUTOPLAY_INTERVAL_MS = 900;
const NO_MARKS: MoveMark[] = [];
const NO_POINTS: EvalPoint[] = [];

export const GAMES_PER_PAGE = 8;
export type ReplayTab = 'all' | 'mistakes' | 'best';

export class ArchiveStore {
  games: GameSummary[] = [];
  stats: ArchiveStats | null = null;
  filter: ArchiveFilter = {
    section: 'all',
    query: '',
    color: 'any',
    sort: 'newest',
  };
  pageIndex = 0;
  replay: ReplayGame | null = null;
  replayId: number | null = null;
  replayTab: ReplayTab = 'all';
  isReplayFlipped = false;
  frameIndex = 0;
  isAutoplaying = false;
  autoplaySpeed = 1;
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
      const [games, stats] = await Promise.all([
        fetchGames(),
        fetchArchiveStats(),
      ]);

      runInAction(() => {
        this.games = games;
        this.stats = stats;
      });
    });

  setFilter = (patch: Partial<ArchiveFilter>) => {
    this.filter = { ...this.filter, ...patch };
    this.pageIndex = 0;
  };

  setPageIndex = (pageIndex: number) => {
    this.pageIndex = pageIndex;
  };

  open = (id: number) =>
    this.attempt(async () => {
      const replay = await fetchReplay(id);

      runInAction(() => {
        this.stopAutoplay();
        this.replay = replay;
        this.replayId = id;
        this.replayTab = 'all';
        this.isReplayFlipped = false;
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

  toggleFavorite = (game: GameSummary) =>
    this.attempt(async () => {
      await setGameFavorite(game.id, !game.isFavorite);
      await this.refresh();
    });

  importFromFile = () =>
    this.attempt(async () => {
      const path = await pickFile(PGN_EXTENSIONS);

      if (path) {
        await importGame(path);
        await this.refresh();
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

  get points() {
    return this.activeReview?.points ?? NO_POINTS;
  }

  get frameMark() {
    return this.marks.find((mark) => mark.ply === this.frameIndex) ?? null;
  }

  get replayOrientation(): Color {
    const base = this.replay?.humanColor ?? 'white';
    const opposite = base === 'white' ? 'black' : 'white';

    return this.isReplayFlipped ? opposite : base;
  }

  private isPlayersMove = (color: Color) => {
    const human = this.replay?.humanColor ?? null;

    return human === null || human === color;
  };

  /** A game with no known player counts both sides. */
  get playerMistakePlies() {
    return this.marks
      .filter((mark) => this.isPlayersMove(mark.color))
      .map((mark) => mark.ply);
  }

  get playerBestPlies() {
    const moves = this.replay?.moves ?? [];

    return this.points
      .filter(
        ({ ply, isBest }) =>
          isBest &&
          moves[ply - 1] !== undefined &&
          this.isPlayersMove(moves[ply - 1].color)
      )
      .map(({ ply }) => ply);
  }

  /** The tab's moves stand out in the list; `null` shows them all alike. */
  get highlightedPlies() {
    if (this.replayTab === 'mistakes') {
      return this.playerMistakePlies;
    }

    return this.replayTab === 'best' ? this.playerBestPlies : null;
  }

  get lastFrameIndex() {
    return (this.replay?.frames.length ?? 1) - 1;
  }

  setReplayTab = (tab: ReplayTab) => {
    this.replayTab = tab;
  };

  flipReplay = () => {
    this.isReplayFlipped = !this.isReplayFlipped;
  };

  goTo = (index: number) => {
    this.frameIndex = Math.min(Math.max(0, index), this.lastFrameIndex);
  };

  step = (offset: number) => {
    this.goTo(this.frameIndex + offset);
  };

  /** Round to the first one after the last. */
  goToNextMistake = () => {
    const plies = this.playerMistakePlies;
    const next = plies.find((ply) => ply > this.frameIndex) ?? plies.at(0);

    if (next !== undefined) {
      this.goTo(next);
    }
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
    this.autoplayTimer = setInterval(
      this.advance,
      AUTOPLAY_INTERVAL_MS / this.autoplaySpeed
    );
  };

  setAutoplaySpeed = (speed: number) => {
    this.autoplaySpeed = speed;

    if (this.isAutoplaying) {
      this.startAutoplay();
    }
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
