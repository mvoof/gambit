import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/components/Button/Button';
import { Card } from '@/components/Card/Card';
import { greet } from '@/lib/tauri';
import { normalizeSpaces } from '@/lib/format';

import styles from './GreetPanel.module.scss';

/** Round trip into Rust: the one place the demo touches the backend. */
export const GreetPanel = () => {
  const { t } = useTranslation();
  const [name, setName] = useState('');
  const [message, setMessage] = useState('');

  const handleGreet = async () => {
    const response = await greet(normalizeSpaces(name));

    setMessage(response);
  };

  return (
    <Card title={t('greet.title')}>
      <label className={styles.label} htmlFor="greet-name">
        {t('greet.label')}
      </label>

      <input
        id="greet-name"
        className={styles.input}
        value={name}
        placeholder={t('greet.placeholder')}
        onChange={(event) => setName(event.currentTarget.value)}
      />

      <Button
        onClick={handleGreet}
        disabled={normalizeSpaces(name).length === 0}
      >
        {t('greet.action')}
      </Button>

      {message !== '' && <p className={styles.message}>{message}</p>}
    </Card>
  );
};
