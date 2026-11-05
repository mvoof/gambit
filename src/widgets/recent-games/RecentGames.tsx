import { ArrowRight, Play } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useGameStore } from '@entities/game/game-context';
import { ResultBadge } from '@entities/archive/ui/ResultBadge/ResultBadge';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { GameIdentity } from '@features/game-identity/GameIdentity';
import type { GameSummary } from '@shared/contracts/generated/bindings';
import { formatPlayedAt } from '@shared/lib/format-date';
import { Button } from '@shared/ui/Button/Button';
import { Panel } from '@shared/ui/Panel/Panel';

import styles from './RecentGames.module.scss';

const RECENT_COUNT = 3;

export const RecentGames = observer(() => {
  const { t, i18n } = useTranslation();
  const archive = useArchiveStore();
  const gameStore = useGameStore();
  const navigation = useNavigationStore();
  const games = archive.games.slice(0, RECENT_COUNT);

  /** An unfinished game is played on; a finished one is reviewed. */
  const openGame = async (game: GameSummary) => {
    if (game.playerResult === 'unfinished') {
      navigation.request('play');
      await gameStore.resume(game.id);
    } else {
      await archive.open(game.id);
      navigation.request('archive');
    }
  };

  return (
    <Panel
      title={t('home.recent')}
      action={
        <button
          type="button"
          className={styles.all}
          onClick={() => {
            navigation.request('archive');
          }}
        >
          {t('home.allGames')}
          <ArrowRight aria-hidden />
        </button>
      }
    >
      {games.length === 0 ? (
        <p className={styles.empty}>{t('archive.empty')}</p>
      ) : null}
      <ul className={styles.list}>
        {games.map((game) => (
          <li key={game.id} className={styles.game}>
            <GameIdentity game={game} />
            <span>
              <ResultBadge result={game.playerResult} score={game.result} />
            </span>
            <span className={styles.detail}>
              {t('archive.moves', { count: Math.ceil(game.plyCount / 2) })}
            </span>
            <span className={styles.detail}>
              {formatPlayedAt(game.playedAt, i18n.language)}
            </span>
            <Button
              aria-label={t(
                game.playerResult === 'unfinished'
                  ? 'archive.playOn'
                  : 'archive.review'
              )}
              title={t(
                game.playerResult === 'unfinished'
                  ? 'archive.playOn'
                  : 'archive.review'
              )}
              onClick={() => {
                void openGame(game);
              }}
            >
              <Play aria-hidden />
            </Button>
          </li>
        ))}
      </ul>
    </Panel>
  );
});
