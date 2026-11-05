import { useTranslation } from 'react-i18next';

import { UpdateSettings } from '@features/app-update/UpdateSettings';
import { BoardCustomise } from '@features/board-customise/BoardCustomise';
import { EngineSetup } from '@features/engine-setup/EngineSetup';
import { GeneralSettings } from '@features/general-settings/GeneralSettings';
import { PieceImport } from '@features/piece-import/PieceImport';
import { PieceSetPicker } from '@features/piece-import/PieceSetPicker';

import styles from './SettingsPage.module.scss';

export const SettingsPage = () => {
  const { t } = useTranslation();

  return (
    <main className={styles.page}>
      <h1 className={styles.title}>{t('settings.title')}</h1>
      <section className={styles.section}>
        <h2 className={styles.heading}>{t('settings.engine')}</h2>
        <EngineSetup />
      </section>
      <section className={styles.section}>
        <h2 className={styles.heading}>{t('settings.general')}</h2>
        <GeneralSettings />
      </section>
      <section className={styles.section}>
        <h2 className={styles.heading}>{t('settings.board')}</h2>
        <BoardCustomise />
      </section>
      <section className={styles.section}>
        <h2 className={styles.heading}>{t('settings.pieces')}</h2>
        <PieceSetPicker />
        <PieceImport />
      </section>
      <section className={styles.section}>
        <h2 className={styles.heading}>{t('settings.updates')}</h2>
        <UpdateSettings />
      </section>
    </main>
  );
};
