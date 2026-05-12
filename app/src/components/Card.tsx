import type { ReactNode, HTMLAttributes } from 'react';
import styles from './Card.module.css';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'elevated';
  children: ReactNode;
}

export function Card({
  variant = 'default',
  children,
  className,
  ...rest
}: CardProps) {
  const cls = [
    styles.card,
    variant === 'elevated' ? styles.elevated : undefined,
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={cls} {...rest}>
      {children}
    </div>
  );
}
