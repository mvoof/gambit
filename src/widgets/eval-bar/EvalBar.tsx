import { observer } from 'mobx-react-lite';

import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import type { Color } from '@shared/contracts/generated/bindings';

import styles from './EvalBar.module.scss';

const CENTIPAWNS_PER_PAWN = 100;
const SCORE_DECIMALS = 1;

interface EvalBarProps {
  orientation: Color;
}

const scoreText = (centipawns: number | null, mateIn: number | null) => {
  if (mateIn !== null) {
    return `M${Math.abs(mateIn)}`;
  }

  if (centipawns === null) {
    return '';
  }

  return Math.abs(centipawns / CENTIPAWNS_PER_PAWN).toFixed(SCORE_DECIMALS);
};

export const EvalBar = observer(({ orientation }: EvalBarProps) => {
  const evaluation = useEvaluationStore();
  const { latest, whiteWinPercent } = evaluation;
  const classNames =
    orientation === 'white' ? styles.bar : `${styles.bar} ${styles.flipped}`;

  return (
    <div className={classNames} data-testid="eval-bar">
      <div className={styles.white} style={{ height: `${whiteWinPercent}%` }} />
      <span className={styles.score}>
        {scoreText(latest?.centipawns ?? null, latest?.mateIn ?? null)}
      </span>
    </div>
  );
});
