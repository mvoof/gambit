import { ChevronDown } from 'lucide-react';

import styles from './Select.module.scss';

interface SelectOption<Value> {
  value: Value;
  label: string;
}

interface SelectProps<Value> {
  /** What the control chooses, for assistive technology. */
  label: string;
  options: SelectOption<Value>[];
  value: Value;
  onChange: (value: Value) => void;
}

export const Select = <Value extends string>({
  label,
  options,
  value,
  onChange,
}: SelectProps<Value>) => (
  <span className={styles.field}>
    <select
      className={styles.select}
      aria-label={label}
      value={value}
      onChange={(event) => {
        const chosen = options.find(
          (option) => option.value === event.target.value
        );

        if (chosen) {
          onChange(chosen.value);
        }
      }}
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
    <ChevronDown className={styles.chevron} aria-hidden />
  </span>
);
