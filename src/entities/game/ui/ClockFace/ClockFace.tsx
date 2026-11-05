import type { ClockView, Color } from '@shared/contracts/generated/bindings';
import { formatClock } from '@shared/lib/format-clock';

import { useCountdown } from '../../use-countdown';
import styles from './ClockFace.module.scss';

const LOW_TIME_MS = 20_000;

interface ClockFaceProps {
  clock: ClockView | null;
  color: Color;
}

export const ClockFace = ({ clock, color }: ClockFaceProps) => {
  const remainingMs = useCountdown(clock, color);
  const isRunning = clock?.running === color;
  const isLow = isRunning && remainingMs < LOW_TIME_MS;
  const classNames = [
    styles.clock,
    isRunning ? styles.running : '',
    isLow ? styles.low : '',
  ]
    .filter(Boolean)
    .join(' ');

  return <span className={classNames}>{formatClock(remainingMs)}</span>;
};
