import { Trash2 } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { Button } from '@shared/ui/Button/Button';

import styles from './PieceSetPicker.module.scss';

const SAMPLE_PIECES = ['wK', 'wQ', 'bN', 'bP'];

export const PieceSetPicker = observer(() => {
  const { t } = useTranslation();
  const pieces = usePiecesStore();
  const settingsStore = useSettingsStore();
  const activeId = settingsStore.settings?.pieceSet;

  return (
    <div className={styles.sets}>
      {pieces.allSets.map((set) => (
        <div
          key={set.id}
          className={
            set.id === activeId ? `${styles.set} ${styles.active}` : styles.set
          }
        >
          <button
            type="button"
            className={styles.choose}
            onClick={() => {
              void settingsStore.update({ pieceSet: set.id });
            }}
          >
            <span className={styles.sample}>
              {SAMPLE_PIECES.map((code) => (
                <img key={code} src={set.images[code]} alt="" />
              ))}
            </span>
            <span>{set.name}</span>
          </button>
          {pieces.customSets.includes(set) ? (
            <Button
              variant="ghost"
              aria-label={t('pieces.delete')}
              onClick={() => {
                void pieces.remove(set.id);
              }}
            >
              <Trash2 className={styles.deleteIcon} />
            </Button>
          ) : null}
        </div>
      ))}
    </div>
  );
});
