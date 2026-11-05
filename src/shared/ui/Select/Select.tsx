import styles from './Select.module.scss';

interface SelectOption<Value> {
  value: Value;
  label: string;
}

interface SelectProps<Value> {
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
  <label className={styles.field}>
    <span className={styles.label}>{label}</span>
    <select
      className={styles.select}
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
  </label>
);
