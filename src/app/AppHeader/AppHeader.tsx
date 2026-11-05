import { Archive, Home, Settings } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import brandMark from '@assets/brand/gambit-mark.png';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import type { AppPage } from '@entities/navigation/navigation.store';
import { UpdateNotice } from '@features/app-update/UpdateNotice';

import { WindowControls } from './WindowControls';
import styles from './AppHeader.module.scss';

const PAGES: { page: AppPage; icon: LucideIcon }[] = [
  { page: 'home', icon: Home },
  { page: 'archive', icon: Archive },
  { page: 'settings', icon: Settings },
];

/** The window has no native title bar: this one drags, and holds its buttons. */
export const AppHeader = observer(() => {
  const { t } = useTranslation();
  const navigation = useNavigationStore();

  return (
    <header className={styles.header} data-tauri-drag-region>
      <div className={styles.brandBlock} data-tauri-drag-region>
        <img
          className={styles.mark}
          src={brandMark}
          alt=""
          data-tauri-drag-region
        />
        <span className={styles.brand} data-tauri-drag-region>
          {t('app.title')}
        </span>
        <UpdateNotice />
      </div>
      <nav className={styles.nav}>
        {PAGES.map(({ page, icon: Icon }) => (
          <button
            key={page}
            type="button"
            aria-current={navigation.page === page ? 'page' : undefined}
            className={
              navigation.page === page
                ? `${styles.link} ${styles.active}`
                : styles.link
            }
            onClick={() => {
              navigation.request(page);
            }}
          >
            <Icon className={styles.icon} aria-hidden />
            {t(`nav.${page}`)}
          </button>
        ))}
      </nav>
      <div className={styles.windowBlock} data-tauri-drag-region>
        <WindowControls />
      </div>
    </header>
  );
});
