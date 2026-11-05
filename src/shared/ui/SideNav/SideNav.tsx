import type { LucideIcon } from 'lucide-react';

import styles from './SideNav.module.scss';

export interface SideNavItem<Value> {
  value: Value;
  label: string;
  icon: LucideIcon;
  count?: number;
  /** Starts a new group of items. */
  isGroupStart?: boolean;
}

interface SideNavProps<Value> {
  label: string;
  items: SideNavItem<Value>[];
  value: Value;
  onChange: (value: Value) => void;
}

export const SideNav = <Value extends string>({
  label,
  items,
  value,
  onChange,
}: SideNavProps<Value>) => (
  <nav className={styles.nav} aria-label={label}>
    {items.map(
      ({ value: itemValue, label: itemLabel, icon: Icon, ...item }) => {
        const classNames = [
          styles.item,
          itemValue === value ? styles.active : '',
          item.isGroupStart ? styles.groupStart : '',
        ]
          .filter(Boolean)
          .join(' ');

        return (
          <button
            key={itemValue}
            type="button"
            className={classNames}
            aria-current={itemValue === value ? 'true' : undefined}
            onClick={() => {
              onChange(itemValue);
            }}
          >
            <Icon className={styles.icon} aria-hidden />
            <span className={styles.label}>{itemLabel}</span>
            {item.count === undefined ? null : (
              <span className={styles.count}>{item.count}</span>
            )}
          </button>
        );
      }
    )}
  </nav>
);
