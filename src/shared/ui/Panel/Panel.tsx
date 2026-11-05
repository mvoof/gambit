import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import styles from './Panel.module.scss';

interface PanelProps {
  title?: string;
  icon?: LucideIcon;
  /** Sits at the far end of the heading: a link or a filter. */
  action?: ReactNode;
  className?: string;
  children: ReactNode;
}

/** The framed block every page is built from. */
export const Panel = ({
  title,
  icon: Icon,
  action,
  className,
  children,
}: PanelProps) => (
  <section
    className={className ? `${styles.panel} ${className}` : styles.panel}
  >
    {title ? (
      <header className={styles.header}>
        {Icon ? <Icon className={styles.icon} aria-hidden /> : null}
        <h2 className={styles.title}>{title}</h2>
        {action ? <div className={styles.action}>{action}</div> : null}
      </header>
    ) : null}
    {children}
  </section>
);
