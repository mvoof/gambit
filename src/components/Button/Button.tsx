import type { ButtonHTMLAttributes, ReactNode } from 'react';

import styles from './Button.module.scss';

type ButtonVariant = 'primary' | 'secondary' | 'danger';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  children: ReactNode;
}

export const Button = ({
  variant = 'primary',
  children,
  ...buttonProps
}: ButtonProps) => (
  <button
    type="button"
    className={`${styles.button} ${styles[variant]}`}
    {...buttonProps}
  >
    {children}
  </button>
);
