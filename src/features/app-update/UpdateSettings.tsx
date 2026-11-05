import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useUpdateStore } from '@entities/update/update-context';
import { Button } from '@shared/ui/Button/Button';

import { UpdateNotice } from './UpdateNotice';
import styles from './UpdateSettings.module.scss';

export const UpdateSettings = observer(() => {
  const { t } = useTranslation();
  const update = useUpdateStore();

  return (
    <div className={styles.settings}>
      <Button disabled={update.status === 'checking'} onClick={update.check}>
        {t('update.check')}
      </Button>
      {update.status === 'upToDate' ? (
        <p className={styles.status}>{t('update.upToDate')}</p>
      ) : null}
      {update.error ? <p className={styles.error}>{update.error}</p> : null}
      <UpdateNotice />
    </div>
  );
});
