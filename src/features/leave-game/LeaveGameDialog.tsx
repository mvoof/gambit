import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';

import { LeaveGamePrompt } from './LeaveGamePrompt';

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
    <LeaveGamePrompt
      isOpen={navigation.pendingPage !== null}
      leaveLabel={t('leaveGame.leave')}
      onStay={navigation.cancelLeave}
      onLeave={leave}
    />
  );
});
