import { makeAutoObservable } from 'mobx';

import { onEvaluation } from '@shared/api/events.service';
import { analysePosition } from '@shared/api/game.service';
import type { Evaluation } from '@shared/contracts/generated/bindings';

const EVEN_PERCENT = 50;

export class EvaluationStore {
  latest: Evaluation | null = null;
  targetFen: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  start = async () => {
    await onEvaluation(this.receive);
  };

  /** A reply for another position arrives late and is dropped. */
  private receive = (evaluation: Evaluation) => {
    if (evaluation.fen === this.targetFen) {
      this.latest = evaluation;
    }
  };

  setTarget = async (fen: string | null) => {
    if (fen === this.targetFen) {
      return;
    }

    this.targetFen = fen;
    await analysePosition(fen);
  };

  /** The last reply may still be about the position before this one. */
  bestMoveFor = (fen: string) =>
    this.latest?.fen === fen ? this.latest.bestMove : null;

  linesFor = (fen: string) =>
    this.latest?.fen === fen ? this.latest.lines : [];

  get whiteWinPercent() {
    return this.latest?.whiteWinPercent ?? EVEN_PERCENT;
  }
}
