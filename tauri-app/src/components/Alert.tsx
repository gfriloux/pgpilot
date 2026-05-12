import { useState } from 'react';
import styles from './Alert.module.css';
import type { SemanticColor } from '../types';

const ICON: Record<Exclude<SemanticColor, 'neutral'>, string> = {
  success: '✓',
  error:   '✕',
  warning: '!',
  info:    'i',
};

export interface AlertProps {
  variant: Exclude<SemanticColor, 'neutral'>;
  message: string;
  title?: string;
  dismissible?: boolean;
}

export function Alert({ variant, message, title, dismissible = false }: AlertProps) {
  const [dismissed, setDismissed] = useState(false);

  if (dismissed) return null;

  return (
    <div
      className={[styles.alert, styles[variant]].join(' ')}
      role="alert"
      aria-live="polite"
    >
      <span className={styles.icon} aria-hidden="true">
        {ICON[variant]}
      </span>
      <div className={styles.body}>
        {title && <span className={styles.title}>{title}</span>}
        <span className={styles.message}>{message}</span>
      </div>
      {dismissible && (
        <button
          className={styles.dismiss}
          onClick={() => setDismissed(true)}
          aria-label="Dismiss alert"
        >
          &times;
        </button>
      )}
    </div>
  );
}
