import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useEngineStore } from '@entities/engine/engine-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { Button } from '@shared/ui/Button/Button';

import { EngineLinks } from './EngineLinks';
import styles from './EngineSetup.module.scss';

const PERCENT = 100;

export const EngineSetup = observer(() => {
  const { t } = useTranslation();
  const engineStore = useEngineStore();
  const settingsStore = useSettingsStore();
  const { engine, isInstalling, progress, error } = engineStore;

  const percentDone = progress
    ? Math.round((progress.downloadedBytes / progress.totalBytes) * PERCENT)
    : 0;

  const pickEngine = async () => {
    if (await engineStore.pickOwnEngine()) {
      await settingsStore.reload();
    }
  };

  return (
    <div className={styles.setup}>
      <p className={styles.current}>
        {engine ? t('engine.current', { name: engine.name }) : t('engine.none')}
      </p>
      <div className={styles.actions}>
        <Button
          variant={engine ? 'secondary' : 'primary'}
          disabled={isInstalling}
          onClick={engineStore.download}
        >
          {t('engine.download')}
        </Button>
        <Button disabled={isInstalling} onClick={pickEngine}>
          {t('engine.pick')}
        </Button>
      </div>
      {isInstalling && progress ? (
        <p className={styles.progress}>
          {t(`engine.stage.${progress.stage}`, { percent: percentDone })}
        </p>
      ) : null}
      {error ? <p className={styles.error}>{error}</p> : null}
      <EngineLinks />
    </div>
  );
});
