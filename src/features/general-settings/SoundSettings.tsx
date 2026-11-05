import { Volume2 } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useSettingsStore } from '@entities/settings/settings-context';

import styles from './SoundSettings.module.scss';

const MAX_VOLUME = 100;

export const SoundSettings = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const { settings } = store;

  if (!settings) {
    return null;
  }

  return (
    <label className={styles.volume}>
      <span className={styles.label}>{t('settings.volumeLabel')}</span>
      <Volume2 className={styles.icon} aria-hidden />
      <input
        className={styles.slider}
        type="range"
        min={0}
        max={MAX_VOLUME}
        value={settings.soundVolume}
        onChange={(event) => {
          void store.update({ soundVolume: event.target.valueAsNumber });
        }}
      />
      <span className={styles.value}>{settings.soundVolume}%</span>
    </label>
  );
});
