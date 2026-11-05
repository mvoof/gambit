import { FlipVertical2, Flag, Handshake, Plus, Undo2 } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { LeaveGamePrompt } from '@features/leave-game/LeaveGamePrompt';
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
  const [isLeavePromptOpen, setLeavePromptOpen] = useState(false);
  const { view, isPlaying, canBeLost } = game;

  const askForNewGame = () => {
    if (canBeLost) {
      setLeavePromptOpen(true);
    } else {
      setNewGameOpen(true);
    }
  };

  /** The running game is saved when the new one starts, so it can still go on. */
  const continueToNewGame = () => {
    setLeavePromptOpen(false);
    setNewGameOpen(true);
  };

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
        onClick={askForNewGame}
      >
        <Plus aria-hidden />
        {t('game.newGame')}
      </Button>
      <LeaveGamePrompt
        isOpen={isLeavePromptOpen}
        leaveLabel={t('leaveGame.startNew')}
        onStay={() => {
          setLeavePromptOpen(false);
        }}
        onLeave={continueToNewGame}
      />
      <NewGameDialog
        isOpen={isNewGameOpen}
        onClose={() => {
          setNewGameOpen(false);
        }}
      />
    </div>
  );
});
