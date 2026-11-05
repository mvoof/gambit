import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { Button } from '@shared/ui/Button/Button';
import { Dialog } from '@shared/ui/Dialog/Dialog';

import styles from './LeaveGameDialog.module.scss';

/** Asks before a running game stops. The game itself is already saved. */
export const LeaveGameDialog = observer(() => {
  const { t } = useTranslation();
  const navigation = useNavigationStore();
  const game = useGameStore();

  const leave = async () => {
    await game.suspend();
    navigation.confirmLeave();
  };

  return (
    <Dialog
      title={t('leaveGame.title')}
      isOpen={navigation.pendingPage !== null}
      onClose={navigation.cancelLeave}
    >
      <p className={styles.text}>{t('leaveGame.text')}</p>
      <div className={styles.actions}>
        <Button variant="ghost" onClick={navigation.cancelLeave}>
          {t('leaveGame.stay')}
        </Button>
        <Button variant="primary" onClick={leave}>
          {t('leaveGame.leave')}
        </Button>
      </div>
    </Dialog>
  );
});
