import type { ButtonHTMLAttributes } from 'react';

import styles from './Button.module.scss';

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

export const Button = ({
  variant = 'secondary',
  className,
  type = 'button',
  ...buttonProps
}: ButtonProps) => {
  const classNames = [styles.button, styles[variant], className]
    .filter(Boolean)
    .join(' ');

  return <button type={type} className={classNames} {...buttonProps} />;
};
