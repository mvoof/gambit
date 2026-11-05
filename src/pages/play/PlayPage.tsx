import { observer } from 'mobx-react-lite';
import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useEngineStore } from '@entities/engine/engine-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { NewGameDialog } from '@features/new-game/NewGameDialog';
import { useArrowKeys } from '@shared/lib/use-arrow-keys';
import { Button } from '@shared/ui/Button/Button';
import { EvalBar } from '@widgets/eval-bar/EvalBar';
import { GameBoard } from '@widgets/game-board/GameBoard';
import { GameSidePanel } from '@widgets/game-side-panel/GameSidePanel';

import styles from './PlayPage.module.scss';

const FIRST_PLY = 0;

export const PlayPage = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const engineStore = useEngineStore();
  const navigation = useNavigationStore();
  const { settings } = useSettingsStore();
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
        {engineStore.engine ? (
          <Button
            variant="primary"
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
              onClick={() => {
                navigation.open('settings');
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
      {settings?.evalBarEnabled && game.view.isTraining ? (
        <EvalBar orientation={game.orientation} />
      ) : null}
      <GameBoard />
      <GameSidePanel />
    </main>
  );
});
