import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useMoveFormat } from '@entities/settings/use-move-format';

import styles from './ReplayNote.module.scss';

/** The line under the moves: the mark of the move on the board, or else how the
 * measuring of the game is going. */
export const ReplayNote = observer(() => {
  const { t } = useTranslation();
  const { frameMark, activeReview } = useArchiveStore();
  const formatMove = useMoveFormat();

  if (frameMark) {
    return (
      <p className={styles.mark}>
        {t(`game.mark.${frameMark.quality}`, {
          move: formatMove(frameMark.bestSan),
        })}
      </p>
    );
  }

  if (activeReview?.status === 'measuring') {
    return (
      <p className={styles.note}>
        {t('replay.measuring', {
          measured: activeReview.measured,
          total: activeReview.total,
        })}
      </p>
    );
  }

  if (activeReview?.status === 'failed') {
    return <p className={styles.note}>{t('replay.measuringFailed')}</p>;
  }

  return <p className={styles.note}> </p>;
});
