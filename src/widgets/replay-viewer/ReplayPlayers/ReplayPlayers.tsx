import { User } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import type { Color } from '@shared/contracts/generated/bindings';

import styles from './ReplayPlayers.module.scss';

/** White and Black above the board, in the order they sit around it. */
export const ReplayPlayers = observer(() => {
  const { t } = useTranslation();
  const { replay, replayOrientation } = useArchiveStore();

  if (!replay) {
    return null;
  }

  const nameOf = (color: Color) => {
    if (color === replay.humanColor) {
      return t('game.you');
    }

    return color === 'white' ? replay.white : replay.black;
  };
  const order: Color[] =
    replayOrientation === 'white' ? ['white', 'black'] : ['black', 'white'];

  return (
    <div className={styles.players}>
      {order.map((color) => (
        <div key={color} className={styles.player}>
          <span className={styles.avatar}>
            <User aria-hidden />
          </span>
          <span className={styles.text}>
            <span className={styles.name}>{nameOf(color)}</span>
            <span className={styles.side}>{t(`newGame.${color}`)}</span>
          </span>
        </div>
      ))}
    </div>
  );
});
