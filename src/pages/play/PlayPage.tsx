import { observer } from 'mobx-react-lite';
import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useEngineStore } from '@entities/engine/engine-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { NewGameDialog } from '@features/new-game/NewGameDialog';
import { useArrowKeys } from '@shared/lib/use-arrow-keys';
import { Button } from '@shared/ui/Button/Button';
import { GameBoard } from '@widgets/game-board/GameBoard';
import { GameControls } from '@widgets/game-side-panel/GameControls/GameControls';
import { GamePlayers } from '@widgets/game-side-panel/GamePlayers/GamePlayers';
import { GameSidePanel } from '@widgets/game-side-panel/GameSidePanel';

import styles from './PlayPage.module.scss';

const FIRST_PLY = 0;

export const PlayPage = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const engineStore = useEngineStore();
  const navigation = useNavigationStore();
  const [isNewGameOpen, setNewGameOpen] = useState(false);
  const moveCount = game.view?.moves.length ?? 0;

  const showPrevious = useCallback(() => game.browseBy(-1), [game]);
  const showNext = useCallback(() => game.browseBy(1), [game]);
  const showFirst = useCallback(() => game.browseTo(FIRST_PLY), [game]);
  const showLast = useCallback(
    () => game.browseTo(moveCount),
    [game, moveCount]
  );

  useArrowKeys({
    onPrevious: showPrevious,
    onNext: showNext,
    onFirst: showFirst,
    onLast: showLast,
  });

  if (!game.view) {
    return (
      <main className={styles.welcome}>
        <h1 className={styles.title}>{t('app.title')}</h1>
        <p className={styles.tagline}>{t('app.tagline')}</p>
        {game.error ? <p className={styles.error}>{game.error}</p> : null}
        {engineStore.engine ? (
          <Button
            variant="primary"
            size="large"
            onClick={() => {
              setNewGameOpen(true);
            }}
          >
            {t('game.newGame')}
          </Button>
        ) : (
          <>
            <p className={styles.hint}>{t('engine.needed')}</p>
            <Button
              variant="primary"
              size="large"
              onClick={() => {
                navigation.request('settings');
              }}
            >
              {t('engine.setUp')}
            </Button>
          </>
        )}
        <NewGameDialog
          isOpen={isNewGameOpen}
          onClose={() => {
            setNewGameOpen(false);
          }}
        />
      </main>
    );
  }

  return (
    <main className={styles.page}>
      <GamePlayers />
      <div className={styles.center}>
        <GameBoard />
        <GameControls />
      </div>
      <GameSidePanel />
    </main>
  );
});
