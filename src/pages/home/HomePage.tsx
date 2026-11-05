import { observer } from 'mobx-react-lite';
import { useEffect } from 'react';

import { useArchiveStore } from '@entities/archive/archive-context';
import { StartGameCard } from '@features/new-game/StartGameCard';
import { HomeHero } from '@widgets/home-hero/HomeHero';
import { HomeShortcuts } from '@widgets/home-shortcuts/HomeShortcuts';
import { PlayerStats } from '@widgets/player-stats/PlayerStats';
import { RecentGames } from '@widgets/recent-games/RecentGames';

import styles from './HomePage.module.scss';

export const HomePage = observer(() => {
  const { refresh } = useArchiveStore();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <main className={styles.page}>
      <div className={styles.grid}>
        <div className={styles.lead}>
          <HomeHero />
          <HomeShortcuts />
        </div>
        <StartGameCard />
        <RecentGames />
        <PlayerStats />
      </div>
    </main>
  );
});
