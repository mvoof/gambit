import { BarChart3, Equal, Trophy, X } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { Panel } from '@shared/ui/Panel/Panel';

import { WinRing } from './WinRing/WinRing';
import styles from './PlayerStats.module.scss';

interface StatRowProps {
  icon: LucideIcon;
  label: string;
  value: number;
  tone: 'win' | 'loss' | 'draw';
}

const StatRow = ({ icon: Icon, label, value, tone }: StatRowProps) => (
  <div className={styles.row}>
    <dt>
      <Icon className={`${styles.icon} ${styles[tone]}`} aria-hidden />
      {label}
    </dt>
    <dd>{value}</dd>
  </div>
);

/** Every count comes from the archive; there is no rating to show. */
export const PlayerStats = observer(() => {
  const { t } = useTranslation();
  const { stats } = useArchiveStore();

  return (
    <Panel title={t('home.stats')} icon={BarChart3}>
      <div className={styles.body}>
        <WinRing percent={stats?.winPercent ?? 0} />
        <dl className={styles.rows}>
          <StatRow
            icon={Trophy}
            label={t('archive.section.wins')}
            value={stats?.wins ?? 0}
            tone="win"
          />
          <StatRow
            icon={X}
            label={t('archive.section.losses')}
            value={stats?.losses ?? 0}
            tone="loss"
          />
          <StatRow
            icon={Equal}
            label={t('archive.section.draws')}
            value={stats?.draws ?? 0}
            tone="draw"
          />
        </dl>
      </div>
    </Panel>
  );
});
