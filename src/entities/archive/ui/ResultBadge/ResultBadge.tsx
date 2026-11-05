import { Circle, CirclePause, Equal, Trophy, X } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import type { PlayerResult } from '@shared/contracts/generated/bindings';

import styles from './ResultBadge.module.scss';

const ICONS: Record<PlayerResult, LucideIcon> = {
  win: Trophy,
  loss: X,
  draw: Equal,
  unfinished: CirclePause,
  unknown: Circle,
};

interface ResultBadgeProps {
  result: PlayerResult;
  /** The score of a game with no known player. */
  score: string;
}

export const ResultBadge = ({ result, score }: ResultBadgeProps) => {
  const { t } = useTranslation();
  const Icon = ICONS[result];

  return (
    <span className={`${styles.badge} ${styles[result]}`}>
      <Icon className={styles.icon} aria-hidden />
      {result === 'unknown' ? score : t(`playerResult.${result}`)}
    </span>
  );
};
