import { useTranslation } from 'react-i18next';

import styles from './WinRing.module.scss';

const VIEW_SIZE = 100;
const CENTER = VIEW_SIZE / 2;
const RADIUS = 42;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
const FULL_PERCENT = 100;
const START_ANGLE = -90;

interface WinRingProps {
  percent: number;
}

export const WinRing = ({ percent }: WinRingProps) => {
  const { t } = useTranslation();
  const filled = (percent / FULL_PERCENT) * CIRCUMFERENCE;

  return (
    <figure className={styles.ring}>
      <svg viewBox={`0 0 ${VIEW_SIZE} ${VIEW_SIZE}`} aria-hidden>
        <circle className={styles.track} cx={CENTER} cy={CENTER} r={RADIUS} />
        <circle
          className={styles.value}
          cx={CENTER}
          cy={CENTER}
          r={RADIUS}
          strokeDasharray={`${filled} ${CIRCUMFERENCE - filled}`}
          transform={`rotate(${START_ANGLE} ${CENTER} ${CENTER})`}
        />
      </svg>
      <figcaption className={styles.caption}>
        <strong className={styles.percent}>{percent}%</strong>
        <span className={styles.label}>{t('home.winsShare')}</span>
      </figcaption>
    </figure>
  );
};
