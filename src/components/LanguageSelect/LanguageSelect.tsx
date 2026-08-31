import type { ChangeEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { SUPPORTED_LANGUAGES, type AppLanguage } from '@/i18n';

import styles from './LanguageSelect.module.scss';

export const LanguageSelect = () => {
  const { t, i18n } = useTranslation();

  const handleChange = (event: ChangeEvent<HTMLSelectElement>) => {
    void i18n.changeLanguage(event.currentTarget.value as AppLanguage);
  };

  return (
    <label className={styles.wrapper}>
      <span className={styles.caption}>{t('app.language')}</span>

      <select
        className={styles.select}
        value={i18n.language}
        onChange={handleChange}
      >
        {SUPPORTED_LANGUAGES.map((language) => (
          <option key={language} value={language}>
            {language}
          </option>
        ))}
      </select>
    </label>
  );
};
