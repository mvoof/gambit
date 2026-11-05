import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';

import styles from './GameStatus.module.scss';

/** A mark stays until the player's next move. */
const PLIES_OF_A_ROUND = 2;

export const GameStatus = observer(() => {
  const { t } = useTranslation();
  const { view, drawDeclined, error } = useGameStore();

  if (error) {
    return <p className={styles.problem}>{error}</p>;
  }

  if (view?.engineFailed) {
    return <p className={styles.problem}>{t('game.engineFailed')}</p>;
  }

  if (drawDeclined) {
    return <p className={styles.note}>{t('game.drawDeclined')}</p>;
  }

  const lastMark = view?.moveMarks.at(-1);
  const isFresh =
    lastMark && lastMark.ply + PLIES_OF_A_ROUND > (view?.moves.length ?? 0);

  if (lastMark && isFresh) {
    return (
      <p className={styles.problem}>
        {t(`game.mark.${lastMark.quality}`, { move: lastMark.bestSan })}
      </p>
    );
  }

  return (
    <p className={styles.note}>
      {view?.engineThinking ? t('game.thinking') : ' '}
    </p>
  );
});
