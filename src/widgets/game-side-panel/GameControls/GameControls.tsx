import { FlipVertical2, Flag, Handshake, Plus, Undo2 } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { NewGameDialog } from '@features/new-game/NewGameDialog';
import { Button } from '@shared/ui/Button/Button';

import styles from './GameControls.module.scss';

interface ControlProps {
  icon: LucideIcon;
  label: string;
  title: string;
  isDisabled?: boolean;
  isDanger?: boolean;
  onClick: () => void;
}

const Control = ({
  icon: Icon,
  label,
  title,
  isDisabled = false,
  isDanger = false,
  onClick,
}: ControlProps) => (
  <Button
    variant={isDanger ? 'danger' : 'secondary'}
    className={styles.control}
    title={title}
    disabled={isDisabled}
    onClick={onClick}
  >
    <Icon aria-hidden />
    {label}
  </Button>
);

export const GameControls = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const [isNewGameOpen, setNewGameOpen] = useState(false);
  const { view, isPlaying } = game;

  return (
    <div className={styles.controls}>
      <Control
        icon={Undo2}
        label={t('game.takeBackShort')}
        title={t('game.takeBack')}
        isDisabled={!view?.canTakeback}
        onClick={game.takeBack}
      />
      <Control
        icon={Handshake}
        label={t('game.offerDrawShort')}
        title={t('game.offerDraw')}
        isDisabled={!isPlaying}
        onClick={game.offerDraw}
      />
      <Control
        icon={FlipVertical2}
        label={t('game.flipShort')}
        title={t('game.flip')}
        onClick={game.flip}
      />
      <Control
        icon={Flag}
        label={t('game.resign')}
        title={t('game.resign')}
        isDisabled={!isPlaying}
        isDanger
        onClick={game.resign}
      />
      <Button
        variant="primary"
        className={styles.newGame}
        onClick={() => {
          setNewGameOpen(true);
        }}
      >
        <Plus aria-hidden />
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
