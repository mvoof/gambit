import { observer } from 'mobx-react-lite';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCompanionStore } from '@entities/companion/companion-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { Button } from '@shared/ui/Button/Button';
import { Select } from '@shared/ui/Select/Select';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import styles from './LlmSetup.module.scss';

const NO_MODEL = '';
const FOLLOW_APP = 'app';

export const LlmSetup = observer(() => {
  const { t } = useTranslation();
  const settingsStore = useSettingsStore();
  const companion = useCompanionStore();
  const { settings } = settingsStore;
  const llm = settings?.llm;
  const [address, setAddress] = useState(llm?.baseUrl ?? '');
  const isEnabled = llm?.enabled ?? false;
  const baseUrl = llm?.baseUrl ?? '';
  const { checkConnection } = companion;

  useEffect(() => {
    if (isEnabled) {
      void checkConnection(baseUrl);
    }
  }, [isEnabled, baseUrl, checkConnection]);

  if (!llm) {
    return null;
  }

  const saveAddress = async () => {
    await settingsStore.updateLlm({ baseUrl: address });

    setAddress(settingsStore.settings?.llm.baseUrl ?? address);
  };

  const modelIds = [
    ...new Set([...companion.models, ...(llm.model ? [llm.model] : [])]),
  ];

  return (
    <div className={styles.setup}>
      <p className={styles.hint}>{t('llm.hint')}</p>
      <Toggle
        label={t('llm.enabled')}
        isChecked={llm.enabled}
        onChange={(enabled) => {
          void settingsStore.updateLlm({ enabled });
        }}
      />
      {llm.enabled ? (
        <>
          <label className={styles.field}>
            <span className={styles.label}>{t('llm.address')}</span>
            <input
              className={styles.input}
              value={address}
              onChange={(event) => {
                setAddress(event.target.value);
              }}
              onBlur={saveAddress}
            />
          </label>
          <Select
            label={t('llm.model')}
            value={llm.model ?? NO_MODEL}
            options={[
              { value: NO_MODEL, label: t('llm.chooseModel') },
              ...modelIds.map((model) => ({ value: model, label: model })),
            ]}
            onChange={(model) => {
              void settingsStore.updateLlm({ model: model || null });
            }}
          />
          <Select
            label={t('llm.language')}
            value={llm.language ?? FOLLOW_APP}
            options={[
              { value: FOLLOW_APP, label: t('llm.languageApp') },
              { value: 'en', label: 'English' },
              { value: 'ru', label: 'Русский' },
            ]}
            onChange={(language) => {
              void settingsStore.updateLlm({
                language: language === FOLLOW_APP ? null : language,
              });
            }}
          />
          <Button
            disabled={companion.isChecking}
            onClick={() => {
              void checkConnection(baseUrl);
            }}
          >
            {t('llm.check')}
          </Button>
          {companion.connectionError ? (
            <p className={styles.error}>{companion.connectionError}</p>
          ) : null}
        </>
      ) : null}
    </div>
  );
});
