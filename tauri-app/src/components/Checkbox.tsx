import { useRef, useEffect } from 'react';
import styles from './Checkbox.module.css';

export interface CheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  indeterminate?: boolean;
  id?: string;
}

export function Checkbox({
  label,
  checked,
  onChange,
  disabled = false,
  indeterminate = false,
  id,
}: CheckboxProps) {
  const nativeRef = useRef<HTMLInputElement>(null);
  const fieldId = id ?? `checkbox-${label.toLowerCase().replace(/\s+/g, '-')}`;

  useEffect(() => {
    if (nativeRef.current) {
      nativeRef.current.indeterminate = indeterminate;
    }
  }, [indeterminate]);

  const stateClass = indeterminate
    ? styles.indeterminate
    : checked
      ? styles.checked
      : '';

  return (
    <label
      htmlFor={fieldId}
      className={[
        styles.wrapper,
        disabled ? styles.disabled : undefined,
        stateClass,
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <input
        ref={nativeRef}
        id={fieldId}
        type="checkbox"
        className={styles.native}
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.currentTarget.checked)}
        aria-checked={indeterminate ? 'mixed' : checked}
      />
      <span className={styles.box} aria-hidden="true">
        {indeterminate && <span className={styles.mark}>&#8212;</span>}
        {!indeterminate && checked && <span className={styles.mark}>&#10003;</span>}
      </span>
      <span className={styles.label}>{label}</span>
    </label>
  );
}
