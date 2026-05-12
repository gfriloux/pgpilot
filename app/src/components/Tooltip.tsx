import type { ReactNode } from 'react';
import styles from './Tooltip.module.css';
import type { Placement } from '../types';

export interface TooltipProps {
  content: string;
  placement?: Placement;
  children: ReactNode;
}

export function Tooltip({ content, placement = 'top', children }: TooltipProps) {
  return (
    <span className={styles.wrapper}>
      {children}
      <span
        role="tooltip"
        className={[styles.tip, styles[placement]].join(' ')}
      >
        {content}
      </span>
    </span>
  );
}
