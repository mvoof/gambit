import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { usePiecesStore } from '@entities/pieces/pieces-context';
import type { SheetColumnOrder } from '@shared/contracts/generated/bindings';
import { Button } from '@shared/ui/Button/Button';
import { Segmented } from '@shared/ui/Segmented/Segmented';

import styles from './PieceImport.module.scss';

const ORDER_OPTIONS: { value: SheetColumnOrder; label: string }[] = [
  { value: 'kqrbnp', label: 'K Q R B N P' },
  { value: 'kqbnrp', label: 'K Q B N R P' },
];

export const PieceImport = observer(() => {
  const { t } = useTranslation();
  const pieces = usePiecesStore();
  const [name, setName] = useState('');
  const [sheetPath, setSheetPath] = useState<string | null>(null);
  const [order, setOrder] = useState<SheetColumnOrder>('kqrbnp');
  const setLabel = name.trim() || t('pieces.defaultName');

  const chooseSheet = async () => {
    const path = await pieces.pickSheet();

    if (path) {
      setSheetPath(path);
      await pieces.previewSheet(path, order);
    }
  };

  const changeOrder = async (nextOrder: SheetColumnOrder) => {
    setOrder(nextOrder);

    if (sheetPath) {
      await pieces.previewSheet(sheetPath, nextOrder);
    }
  };

  const saveSheet = async () => {
    if (sheetPath) {
      await pieces.importSheet(setLabel, sheetPath, order);
      setSheetPath(null);
      setName('');
    }
  };

  const importFolder = async () => {
    const folder = await pieces.pickFolder();

    if (folder) {
      await pieces.importFolder(setLabel, folder);
      setName('');
    }
  };

  return (
    <div className={styles.import}>
      <input
        className={styles.name}
        type="text"
        placeholder={t('pieces.namePlaceholder')}
        value={name}
        onChange={(event) => {
          setName(event.target.value);
        }}
      />
      <div className={styles.actions}>
        <Button onClick={chooseSheet}>{t('pieces.importSheet')}</Button>
        <Button onClick={importFolder}>{t('pieces.importFolder')}</Button>
      </div>
      {sheetPath ? (
        <>
          <Segmented
            value={order}
            onChange={changeOrder}
            options={ORDER_OPTIONS}
          />
          <div className={styles.preview}>
            {Object.entries(pieces.preview ?? {}).map(([code, url]) => (
              <img key={code} src={url} alt={code} />
            ))}
          </div>
          <Button variant="primary" onClick={saveSheet}>
            {t('pieces.save')}
          </Button>
        </>
      ) : null}
      {pieces.error ? <p className={styles.error}>{pieces.error}</p> : null}
    </div>
  );
});
