import { useTranslation } from 'react-i18next';

import type { CatalogEntry } from '@shared/contracts/generated/bindings';
import { Button } from '@shared/ui/Button/Button';

import styles from './EngineRow.module.scss';

const BYTES_IN_MEGABYTE = 1_000_000;

interface EngineRowProps {
  entry: CatalogEntry;
  isActive: boolean;
  isBusy: boolean;
  /** The install stage in words, while this engine is being installed. */
  installLabel: string | null;
  onInstall: (id: string) => void;
  onActivate: (path: string) => void;
  onRemove: (id: string) => void;
  onOpenHomepage: (url: string) => void;
}

export const EngineRow = ({
  entry,
  isActive,
  isBusy,
  installLabel,
  onInstall,
  onActivate,
  onRemove,
  onOpenHomepage,
}: EngineRowProps) => {
  const { t } = useTranslation();
  const { id, name, version, licence, homepage, sizeBytes, path } = entry;
  const size = Math.round(sizeBytes / BYTES_IN_MEGABYTE);

  const renderActions = () => {
    if (installLabel) {
      return <span className={styles.progress}>{installLabel}</span>;
    }

    if (isActive) {
      return <span className={styles.inUse}>{t('engine.inUse')}</span>;
    }

    if (!path) {
      return (
        <Button
          disabled={isBusy}
          onClick={() => {
            onInstall(id);
          }}
        >
          {t('engine.install')}
        </Button>
      );
    }

    return (
      <>
        <Button
          variant="ghost"
          disabled={isBusy}
          onClick={() => {
            onRemove(id);
          }}
        >
          {t('engine.remove')}
        </Button>
        <Button
          disabled={isBusy}
          onClick={() => {
            onActivate(path);
          }}
        >
          {t('engine.activate')}
        </Button>
      </>
    );
  };

  return (
    <li className={isActive ? `${styles.row} ${styles.active}` : styles.row}>
      <div className={styles.about}>
        <span className={styles.name}>
          {name} <span className={styles.version}>{version}</span>
        </span>
        <span className={styles.details}>
          {licence} · {t('engine.size', { size })} ·{' '}
          <button
            type="button"
            className={styles.link}
            onClick={() => {
              onOpenHomepage(homepage);
            }}
          >
            {t('engine.homepage')}
          </button>
        </span>
      </div>
      <div className={styles.actions}>{renderActions()}</div>
    </li>
  );
};
