import {
  AlertTriangle,
  BarChart3,
  CirclePause,
  Equal,
  FolderOpen,
  Star,
  Trophy,
  X,
} from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import type { ArchiveSection } from '@entities/archive/select-games';
import { formatSize } from '@shared/lib/format-size';
import { SideNav } from '@shared/ui/SideNav/SideNav';
import type { SideNavItem } from '@shared/ui/SideNav/SideNav';

import styles from './ArchiveSidebar.module.scss';

export const ArchiveSidebar = observer(() => {
  const { t, i18n } = useTranslation();
  const archive = useArchiveStore();
  const { stats, filter } = archive;

  const item = (
    value: ArchiveSection,
    icon: SideNavItem<ArchiveSection>['icon'],
    count: number | undefined,
    isGroupStart = false
  ): SideNavItem<ArchiveSection> => ({
    value,
    icon,
    count,
    isGroupStart,
    label: t(`archive.section.${value}`),
  });

  return (
    <aside className={styles.sidebar}>
      <SideNav
        label={t('archive.sections')}
        value={filter.section}
        onChange={(section) => {
          archive.setFilter({ section });
        }}
        items={[
          item('all', FolderOpen, stats?.total),
          item('unfinished', CirclePause, stats?.unfinished),
          item('wins', Trophy, stats?.wins, true),
          item('losses', X, stats?.losses),
          item('draws', Equal, stats?.draws),
          item('reviewed', BarChart3, stats?.reviewed, true),
          item('withMistakes', AlertTriangle, stats?.withMistakes),
          item('favorites', Star, stats?.favorites),
        ]}
      />
      {stats ? (
        <div className={styles.storage}>
          <span className={styles.storageTitle}>{t('archive.storage')}</span>
          <span className={styles.storageText}>
            {t('archive.storageText', {
              count: stats.total,
              size: formatSize(stats.sizeBytes, i18n.language),
            })}
          </span>
        </div>
      ) : null}
    </aside>
  );
});
