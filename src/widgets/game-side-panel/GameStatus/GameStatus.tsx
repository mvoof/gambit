import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import type { GameOutcome } from '@shared/contracts/generated/bindings';

import styles from './GameStatus.module.scss';

const RESULT_KEYS: Record<GameOutcome['result'], string> = {
  whiteWins: 'result.whiteWins',
  blackWins: 'result.blackWins',
  draw: 'result.draw',
};

export const GameStatus = observer(() => {
  const { t } = useTranslation();
  const { view, drawDeclined, error } = useGameStore();

  if (error) {
    return <p className={styles.problem}>{error}</p>;
  }

  if (view?.engineFailed) {
    return <p className={styles.problem}>{t('game.engineFailed')}</p>;
  }

  if (view?.outcome) {
    return (
      <p className={styles.outcome}>
        <strong>{t(RESULT_KEYS[view.outcome.result])}</strong>
        <span>{t(`reason.${view.outcome.reason}`)}</span>
      </p>
    );
  }

  if (drawDeclined) {
    return <p className={styles.note}>{t('game.drawDeclined')}</p>;
  }

  return (
    <p className={styles.note}>
      {view?.engineThinking ? t('game.thinking') : ' '}
    </p>
  );
});
