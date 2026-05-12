import type { InputHTMLAttributes } from 'react';
import styles from './Input.module.css';

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label: string;
  type?: 'text' | 'password' | 'email';
  error?: string;
  hint?: string;
}

export function Input({
  label,
  type = 'text',
  error,
  hint,
  disabled,
  id,
  className,
  ...rest
}: InputProps) {
  const fieldId = id ?? `input-${label.toLowerCase().replace(/\s+/g, '-')}`;
  const errorId = error ? `${fieldId}-error` : undefined;
  const hintId = hint ? `${fieldId}-hint` : undefined;

  const described = [errorId, hintId].filter(Boolean).join(' ') || undefined;

  const inputCls = [
    styles.input,
    error ? styles.hasError : undefined,
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={styles.field}>
      <label className={styles.label} htmlFor={fieldId}>
        {label}
      </label>
      <input
        id={fieldId}
        type={type}
        className={inputCls}
        disabled={disabled}
        aria-invalid={error ? true : undefined}
        aria-describedby={described}
        {...rest}
      />
      {hint && !error && (
        <span id={hintId} className={styles.hint}>
          {hint}
        </span>
      )}
      {error && (
        <span id={errorId} className={styles.error} role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
