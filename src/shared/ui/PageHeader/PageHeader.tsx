import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import styles from './PageHeader.module.scss';

interface PageHeaderProps {
  icon: LucideIcon;
  title: string;
  subtitle: string;
  children?: ReactNode;
}

export const PageHeader = ({
  icon: Icon,
  title,
  subtitle,
  children,
}: PageHeaderProps) => (
  <header className={styles.header}>
    <Icon className={styles.icon} aria-hidden />
    <div className={styles.text}>
      <h1 className={styles.title}>{title}</h1>
      <p className={styles.subtitle}>{subtitle}</p>
    </div>
    {children ? <div className={styles.actions}>{children}</div> : null}
  </header>
);
