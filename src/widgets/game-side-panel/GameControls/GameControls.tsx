import { FlipVertical2, Flag, Handshake, Plus, Undo2 } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { NewGameDialog } from '@features/new-game/NewGameDialog';
import { Button } from '@shared/ui/Button/Button';

import styles from './GameControls.module.scss';

export const GameControls = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const [isNewGameOpen, setNewGameOpen] = useState(false);
  const { view, isPlaying } = game;

  return (
    <div className={styles.controls}>
      <Button
        aria-label={t('game.takeBack')}
        title={t('game.takeBack')}
        disabled={!view?.canTakeback}
        onClick={game.takeBack}
      >
        <Undo2 />
      </Button>
      <Button
        aria-label={t('game.offerDraw')}
        title={t('game.offerDraw')}
        disabled={!isPlaying}
        onClick={game.offerDraw}
      >
        <Handshake />
      </Button>
      <Button
        aria-label={t('game.resign')}
        title={t('game.resign')}
        disabled={!isPlaying}
        onClick={game.resign}
      >
        <Flag />
      </Button>
      <Button
        aria-label={t('game.flip')}
        title={t('game.flip')}
        onClick={game.flip}
      >
        <FlipVertical2 />
      </Button>
      <Button
        variant="primary"
        className={styles.newGame}
        onClick={() => {
          setNewGameOpen(true);
        }}
      >
        <Plus />
        {t('game.newGame')}
      </Button>
      <NewGameDialog
        isOpen={isNewGameOpen}
        onClose={() => {
          setNewGameOpen(false);
        }}
      />
    </div>
  );
});
