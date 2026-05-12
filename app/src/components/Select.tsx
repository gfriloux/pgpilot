import type { SelectHTMLAttributes } from 'react';
import styles from './Select.module.css';

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, 'onChange'> {
  label: string;
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
}

export function Select({
  label,
  options,
  value,
  onChange,
  disabled,
  id,
  ...rest
}: SelectProps) {
  const fieldId = id ?? `select-${label.toLowerCase().replace(/\s+/g, '-')}`;

  return (
    <div className={styles.field}>
      <label className={styles.label} htmlFor={fieldId}>
        {label}
      </label>
      <div className={styles.wrapper}>
        <select
          id={fieldId}
          className={styles.select}
          value={value}
          disabled={disabled}
          onChange={(e) => onChange(e.currentTarget.value)}
          {...rest}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <span className={styles.chevron} aria-hidden="true">&#9660;</span>
      </div>
    </div>
  );
}
