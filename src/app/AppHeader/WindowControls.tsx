import { Copy, Minus, Square, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import styles from './WindowControls.module.scss';
import { useWindowControls } from './use-window-controls';

export const WindowControls = () => {
  const { t } = useTranslation();
  const { isMaximized, minimize, toggleMaximize, close } = useWindowControls();
  const maximizeLabel = t(isMaximized ? 'window.restore' : 'window.maximize');

  return (
    <div className={styles.controls}>
      <button
        type="button"
        className={styles.control}
        aria-label={t('window.minimize')}
        title={t('window.minimize')}
        onClick={minimize}
      >
        <Minus />
      </button>
      <button
        type="button"
        className={styles.control}
        aria-label={maximizeLabel}
        title={maximizeLabel}
        onClick={toggleMaximize}
      >
        {isMaximized ? <Copy /> : <Square />}
      </button>
      <button
        type="button"
        className={`${styles.control} ${styles.close}`}
        aria-label={t('window.close')}
        title={t('window.close')}
        onClick={close}
      >
        <X />
      </button>
    </div>
  );
};
