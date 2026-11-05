import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useSettingsStore } from '@entities/settings/settings-context';
import { Segmented } from '@shared/ui/Segmented/Segmented';

import styles from './GeneralSettings.module.scss';

export const GeneralSettings = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const { settings } = store;

  if (!settings) {
    return null;
  }

  return (
    <div className={styles.general}>
      <span className={styles.label}>{t('settings.language')}</span>
      <div className={styles.language}>
        <Segmented
          value={settings.language}
          onChange={(language) => {
            void store.update({ language });
          }}
          options={[
            { value: 'ru', label: 'Русский' },
            { value: 'en', label: 'English' },
          ]}
        />
      </div>
    </div>
  );
});
