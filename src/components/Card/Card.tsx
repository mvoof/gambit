import type { ReactNode } from 'react';

import styles from './Card.module.scss';

interface CardProps {
  title: string;
  children: ReactNode;
}

export const Card = ({ title, children }: CardProps) => (
  <section className={styles.card}>
    <h2 className={styles.title}>{title}</h2>
    {children}
  </section>
);
