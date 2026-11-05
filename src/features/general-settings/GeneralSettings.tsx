import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useSettingsStore } from '@entities/settings/settings-context';
import { Segmented } from '@shared/ui/Segmented/Segmented';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import styles from './GeneralSettings.module.scss';

const MAX_VOLUME = 100;

export const GeneralSettings = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const { settings } = store;

  if (!settings) {
    return null;
  }

  return (
    <div className={styles.general}>
      <Segmented
        value={settings.language}
        onChange={(language) => {
          void store.update({ language });
        }}
        options={[
          { value: 'en', label: 'English' },
          { value: 'ru', label: 'Русский' },
        ]}
      />
      <label className={styles.volume}>
        <span>{t('settings.volume', { volume: settings.soundVolume })}</span>
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
      </label>
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
