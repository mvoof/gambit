import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import brandMark from '@assets/brand/gambit-mark.png';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import type { AppPage } from '@entities/navigation/navigation.store';

import { WindowControls } from './WindowControls';
import styles from './AppHeader.module.scss';

const PAGES: AppPage[] = ['play', 'archive', 'settings'];

/** The window has no native title bar: this one drags, and holds its buttons. */
export const AppHeader = observer(() => {
  const { t } = useTranslation();
  const navigation = useNavigationStore();

  return (
    <header className={styles.header} data-tauri-drag-region>
      <img
        className={styles.mark}
        src={brandMark}
        alt=""
        data-tauri-drag-region
      />
      <span className={styles.brand} data-tauri-drag-region>
        {t('app.title')}
      </span>
      <nav className={styles.nav}>
        {PAGES.map((page) => (
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
              navigation.open(page);
            }}
          >
            {t(`nav.${page}`)}
          </button>
        ))}
      </nav>
      <WindowControls />
    </header>
  );
});
