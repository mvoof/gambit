import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useSettingsStore } from '@entities/settings/settings-context';
import type { Theme } from '@shared/contracts/generated/bindings';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import styles from './ThemeSettings.module.scss';

const THEMES: Theme[] = ['dark', 'light', 'system'];

export const ThemeSettings = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const { settings } = store;

  if (!settings) {
    return null;
  }

  return (
    <div className={styles.settings}>
      <div className={styles.themes}>
        {THEMES.map((theme) => (
          <button
            key={theme}
            type="button"
            aria-pressed={settings.theme === theme}
            className={
              settings.theme === theme
                ? `${styles.theme} ${styles.active}`
                : styles.theme
            }
            onClick={() => {
              void store.update({ theme });
            }}
          >
            <span className={`${styles.preview} ${styles[theme]}`} />
            {t(`settings.theme.${theme}`)}
          </button>
        ))}
      </div>
      <Toggle
        label={t('settings.evalBar')}
        isChecked={settings.evalBarEnabled}
        onChange={(evalBarEnabled) => {
          void store.update({ evalBarEnabled });
        }}
      />
    </div>
  );
});
