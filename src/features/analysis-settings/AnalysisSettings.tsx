import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useSettingsStore } from '@entities/settings/settings-context';
import { Select } from '@shared/ui/Select/Select';

import styles from './AnalysisSettings.module.scss';

const DEPTHS = ['16', '20', '24', '28', '32', '40'];
const LINE_COUNTS = ['1', '2', '3', '4', '5'];

const toOptions = (values: string[]) =>
  values.map((value) => ({ value, label: value }));

export const AnalysisSettings = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const { settings } = store;

  if (!settings) {
    return null;
  }

  return (
    <dl className={styles.fields}>
      <dt>{t('settings.analysisDepth')}</dt>
      <dd>
        <Select
          label={t('settings.analysisDepth')}
          value={String(settings.analysisDepth)}
          options={toOptions(DEPTHS)}
          onChange={(depth) => {
            void store.update({ analysisDepth: Number(depth) });
          }}
        />
      </dd>
      <dt>{t('settings.analysisLines')}</dt>
      <dd>
        <Select
          label={t('settings.analysisLines')}
          value={String(settings.analysisLines)}
          options={toOptions(LINE_COUNTS)}
          onChange={(lines) => {
            void store.update({ analysisLines: Number(lines) });
          }}
        />
      </dd>
    </dl>
  );
});
