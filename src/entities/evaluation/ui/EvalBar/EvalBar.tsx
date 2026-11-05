import { observer } from 'mobx-react-lite';

import { useEvaluationStore } from '../../evaluation-context';
import type { Color } from '@shared/contracts/generated/bindings';
import { formatScore } from '@shared/lib/format-score';

import styles from './EvalBar.module.scss';

interface EvalBarProps {
  orientation: Color;
}

export const EvalBar = observer(({ orientation }: EvalBarProps) => {
  const evaluation = useEvaluationStore();
  const { latest, whiteWinPercent } = evaluation;
  const classNames =
    orientation === 'white' ? styles.bar : `${styles.bar} ${styles.flipped}`;

  return (
    <div className={styles.column}>
      <span className={styles.score}>{latest ? formatScore(latest) : ' '}</span>
      <div className={classNames} data-testid="eval-bar">
        <div
          className={styles.white}
          style={{ height: `${whiteWinPercent}%` }}
        />
      </div>
    </div>
  );
});
