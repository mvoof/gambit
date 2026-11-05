import { useTranslation } from 'react-i18next';

import { Button } from '@shared/ui/Button/Button';
import { Dialog } from '@shared/ui/Dialog/Dialog';

import styles from './LeaveGamePrompt.module.scss';

interface LeaveGamePromptProps {
  isOpen: boolean;
  leaveLabel: string;
  onStay: () => void;
  onLeave: () => void;
}

/** Says that a game being left is kept and can be played on. */
export const LeaveGamePrompt = ({
  isOpen,
  leaveLabel,
  onStay,
  onLeave,
}: LeaveGamePromptProps) => {
  const { t } = useTranslation();

  return (
    <Dialog title={t('leaveGame.title')} isOpen={isOpen} onClose={onStay}>
      <p className={styles.text}>{t('leaveGame.text')}</p>
      <div className={styles.actions}>
        <Button variant="ghost" onClick={onStay}>
          {t('leaveGame.stay')}
        </Button>
        <Button variant="primary" onClick={onLeave}>
          {leaveLabel}
        </Button>
      </div>
    </Dialog>
  );
};
