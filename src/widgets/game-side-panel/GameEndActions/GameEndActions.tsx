import { BarChart3, Plus } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { NewGameDialog } from '@features/new-game/NewGameDialog';
import { Button } from '@shared/ui/Button/Button';

import styles from './GameEndActions.module.scss';

/** What comes after a game. A game ended with no move has nothing to analyse. */
export const GameEndActions = observer(() => {
  const { t } = useTranslation();
  const { view } = useGameStore();
  const archive = useArchiveStore();
  const navigation = useNavigationStore();
  const [isNewGameOpen, setNewGameOpen] = useState(false);

  if (!view?.outcome) {
    return null;
  }

  const { archiveId } = view;

  const analyse = async () => {
    if (archiveId !== null) {
      await archive.open(archiveId);
      navigation.request('archive');
    }
  };

  return (
    <div className={styles.actions}>
      <Button
        variant="primary"
        size="large"
        onClick={() => {
          setNewGameOpen(true);
        }}
      >
        <Plus aria-hidden />
        {t('game.newGame')}
      </Button>
      {archiveId === null ? null : (
        <Button size="large" onClick={analyse}>
          <BarChart3 aria-hidden />
          {t('game.analyse')}
        </Button>
      )}
      <NewGameDialog
        isOpen={isNewGameOpen}
        onClose={() => {
          setNewGameOpen(false);
        }}
      />
    </div>
  );
});
