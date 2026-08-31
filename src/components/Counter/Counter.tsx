import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/components/Button/Button';
import { Card } from '@/components/Card/Card';
import { formatNumber } from '@/lib/format';

import styles from './Counter.module.scss';

const STEP = 1;

export const Counter = () => {
  const { t, i18n } = useTranslation();
  const [count, setCount] = useState(0);

  const handleIncrement = () => setCount((current) => current + STEP);
  const handleReset = () => setCount(0);

  return (
    <Card title={t('counter.title')}>
      <p className={styles.value}>{formatNumber(count, i18n.language)}</p>

      <div className={styles.actions}>
        <Button onClick={handleIncrement}>{t('counter.increment')}</Button>
        <Button variant="secondary" onClick={handleReset}>
          {t('counter.reset')}
        </Button>
      </div>
    </Card>
  );
};
