import styles from './Segmented.module.scss';

interface SegmentOption<Value> {
  value: Value;
  label: string;
}

interface SegmentedProps<Value> {
  options: SegmentOption<Value>[];
  value: Value;
  onChange: (value: Value) => void;
}

export const Segmented = <Value extends string>({
  options,
  value,
  onChange,
}: SegmentedProps<Value>) => (
  <div className={styles.group}>
    {options.map((option) => (
      <button
        key={option.value}
        type="button"
        aria-pressed={option.value === value}
        className={
          option.value === value
            ? `${styles.segment} ${styles.active}`
            : styles.segment
        }
        onClick={() => {
          onChange(option.value);
        }}
      >
        {option.label}
      </button>
    ))}
  </div>
);
