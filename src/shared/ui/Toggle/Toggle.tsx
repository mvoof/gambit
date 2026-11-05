import styles from './Toggle.module.scss';

interface ToggleProps {
  label: string;
  isChecked: boolean;
  onChange: (isChecked: boolean) => void;
}

export const Toggle = ({ label, isChecked, onChange }: ToggleProps) => (
  <label className={styles.toggle}>
    <input
      type="checkbox"
      className={styles.input}
      checked={isChecked}
      onChange={(event) => {
        onChange(event.target.checked);
      }}
    />
    <span>{label}</span>
  </label>
);
