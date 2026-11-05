import { FlipVertical2, Flag, Handshake, Undo2 } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
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
  const { view, isPlaying } = game;

  return (
    <div className={styles.controls}>
      {view?.isTraining ? (
        <Control
          icon={Undo2}
          label={t('game.takeBackShort')}
          title={t('game.takeBack')}
          isDisabled={!view.canTakeback}
          onClick={game.takeBack}
        />
      ) : null}
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
    </div>
  );
});
