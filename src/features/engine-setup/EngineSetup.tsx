import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useEngineStore } from '@entities/engine/engine-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { Button } from '@shared/ui/Button/Button';

import { EngineLinks } from './EngineLinks';
import { EngineRow } from './EngineRow/EngineRow';
import styles from './EngineSetup.module.scss';
import { useEngineLinks } from './use-engine-links';

const PERCENT = 100;

export const EngineSetup = observer(() => {
  const { t } = useTranslation();
  const engineStore = useEngineStore();
  const settingsStore = useSettingsStore();
  const { openLink } = useEngineLinks();
  const { catalog, engine, installingId, isInstalling, progress, error } =
    engineStore;

  const percentDone = progress
    ? Math.round((progress.downloadedBytes / progress.totalBytes) * PERCENT)
    : 0;

  const installEngine = async (id: string) => {
    await engineStore.install(id);
    await settingsStore.reload();
  };

  const activateEngine = async (path: string) => {
    await engineStore.activate(path);
    await settingsStore.reload();
  };

  const pickEngine = async () => {
    if (await engineStore.pickOwnEngine()) {
      await settingsStore.reload();
    }
  };

  return (
    <div className={styles.setup}>
      <div>
        <p className={styles.current}>
          {engine
            ? t('engine.current', { name: engine.name })
            : t('engine.none')}
        </p>
        {engine ? (
          <p className={styles.strength}>
            {engine.eloRange
              ? t('engine.strengthElo', engine.eloRange)
              : t('engine.strengthDepth')}
          </p>
        ) : null}
      </div>
      <ul className={styles.catalog}>
        {catalog.map((entry) => (
          <EngineRow
            key={entry.id}
            entry={entry}
            isActive={entry.path !== null && entry.path === engine?.path}
            isBusy={isInstalling}
            installLabel={
              installingId === entry.id && progress
                ? t(`engine.stage.${progress.stage}`, { percent: percentDone })
                : null
            }
            onInstall={installEngine}
            onActivate={activateEngine}
            onRemove={engineStore.remove}
            onOpenHomepage={openLink}
          />
        ))}
      </ul>
      <div className={styles.actions}>
        <Button disabled={isInstalling} onClick={pickEngine}>
          {t('engine.pick')}
        </Button>
      </div>
      {error ? <p className={styles.error}>{error}</p> : null}
      <EngineLinks />
    </div>
  );
});
