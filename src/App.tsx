import { useTranslation } from 'react-i18next';

import { Counter } from '@/components/Counter/Counter';
import { GreetPanel } from '@/components/GreetPanel/GreetPanel';
import { LanguageSelect } from '@/components/LanguageSelect/LanguageSelect';

import styles from './App.module.scss';

export const App = () => {
  const { t } = useTranslation();

  return (
    <main className={styles.app}>
      <header className={styles.header}>
        <h1 className={styles.title}>{t('app.title')}</h1>
        <LanguageSelect />
      </header>

      <p className={styles.subtitle}>{t('app.subtitle')}</p>

      <section className={styles.grid}>
        <Counter />
        <GreetPanel />
      </section>
    </main>
  );
};
