import type { ButtonHTMLAttributes } from 'react';

import styles from './Button.module.scss';

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';
type ButtonSize = 'medium' | 'large';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export const Button = ({
  variant = 'secondary',
  size = 'medium',
  className,
  type = 'button',
  ...buttonProps
}: ButtonProps) => {
  const classNames = [styles.button, styles[variant], styles[size], className]
    .filter(Boolean)
    .join(' ');

  return <button type={type} className={classNames} {...buttonProps} />;
};
