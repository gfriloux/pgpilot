import type { HTMLAttributes } from 'react';
import styles from './Badge.module.css';
import type { SemanticColor } from '../types';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  color?: SemanticColor;
  size?: 'sm' | 'md';
  children: string;
}

export function Badge({
  color = 'neutral',
  size = 'md',
  children,
  className,
  ...rest
}: BadgeProps) {
  const cls = [
    styles.badge,
    styles[color],
    styles[size],
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <span className={cls} {...rest}>
      {children}
    </span>
  );
}
