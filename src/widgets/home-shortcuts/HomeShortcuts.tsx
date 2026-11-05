import { BarChart3, ChevronRight, Upload } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';

import styles from './HomeShortcuts.module.scss';

interface ShortcutProps {
  icon: LucideIcon;
  title: string;
  text: string;
  onClick: () => void;
}

const Shortcut = ({ icon: Icon, title, text, onClick }: ShortcutProps) => (
  <button type="button" className={styles.shortcut} onClick={onClick}>
    <Icon className={styles.icon} aria-hidden />
    <span className={styles.text}>
      <span className={styles.title}>{title}</span>
      <span className={styles.description}>{text}</span>
    </span>
    <ChevronRight className={styles.chevron} aria-hidden />
  </button>
);

export const HomeShortcuts = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const navigation = useNavigationStore();

  return (
    <div className={styles.shortcuts}>
      <Shortcut
        icon={BarChart3}
        title={t('home.review')}
        text={t('home.reviewText')}
        onClick={() => {
          navigation.request('archive');
        }}
      />
      <Shortcut
        icon={Upload}
        title={t('home.import')}
        text={t('home.importText')}
        onClick={archive.importFromFile}
      />
    </div>
  );
});
