import { RotateCcw } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { formatSize } from '@shared/lib/format-size';
import { Button } from '@shared/ui/Button/Button';

import styles from './DataSettings.module.scss';

export const DataSettings = observer(() => {
  const { t, i18n } = useTranslation();
  const archive = useArchiveStore();
  const settingsStore = useSettingsStore();
  const { refresh, stats } = archive;

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className={styles.data}>
      {stats ? (
        <p className={styles.storage}>
          {t('archive.storageText', {
            count: stats.total,
            size: formatSize(stats.sizeBytes, i18n.language),
          })}
        </p>
      ) : null}
      <div>
        <Button onClick={settingsStore.reset}>
          <RotateCcw aria-hidden />
          {t('settings.reset')}
        </Button>
      </div>
      <p className={styles.note}>{t('settings.resetNote')}</p>
    </div>
  );
});
