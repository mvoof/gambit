import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useUpdateStore } from '@entities/update/update-context';
import { Button } from '@shared/ui/Button/Button';

import styles from './UpdateNotice.module.scss';

/** Offered, never forced: installing restarts the app, and a game may be on. */
export const UpdateNotice = observer(() => {
  const { t } = useTranslation();
  const update = useUpdateStore();

  if (update.status === 'installing') {
    return (
      <p className={styles.notice}>
        {t('update.installing', { percent: update.percentDone })}
      </p>
    );
  }

  if (update.status !== 'available') {
    return null;
  }

  return (
    <div className={styles.notice}>
      <span>{t('update.available', { version: update.version })}</span>
      <Button variant="primary" onClick={update.install}>
        {t('update.install')}
      </Button>
    </div>
  );
});
