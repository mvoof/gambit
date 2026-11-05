import type { LucideIcon } from 'lucide-react';

import styles from './GameBanner.module.scss';

export type BannerTone = 'success' | 'danger' | 'neutral';

interface GameBannerProps {
  tone: BannerTone;
  icon: LucideIcon;
  title: string;
  text?: string | null;
}

/** The news of the game, big enough to be seen while looking at the board. */
export const GameBanner = ({
  tone,
  icon: Icon,
  title,
  text = null,
}: GameBannerProps) => (
  <section className={`${styles.banner} ${styles[tone]}`} aria-live="polite">
    <Icon className={styles.icon} aria-hidden />
    <h2 className={styles.title}>{title}</h2>
    {text ? <p className={styles.text}>{text}</p> : null}
  </section>
);
