import { Download, Play, RotateCcw, Star, Trash2 } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { ResultBadge } from '@entities/archive/ui/ResultBadge/ResultBadge';
import { useEngineStore } from '@entities/engine/engine-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { GameIdentity } from '@features/game-identity/GameIdentity';
import type { GameSummary } from '@shared/contracts/generated/bindings';
import { formatPlayedAt } from '@shared/lib/format-date';
import { Button } from '@shared/ui/Button/Button';

import styles from './GameRow.module.scss';

interface GameRowProps {
  game: GameSummary;
}

const PLIES_PER_MOVE = 2;

export const GameRow = observer(({ game }: GameRowProps) => {
  const { t, i18n } = useTranslation();
  const archive = useArchiveStore();
  const gameStore = useGameStore();
  const navigation = useNavigationStore();
  const { engine } = useEngineStore();
  const isUnfinished = game.playerResult === 'unfinished';

  const playOn = async () => {
    navigation.request('play');
    await gameStore.resume(game.id);
  };

  return (
    <li className={styles.row}>
      <div className={styles.identity}>
        <GameIdentity game={game} />
        <button
          type="button"
          className={
            game.isFavorite ? `${styles.star} ${styles.starred}` : styles.star
          }
          aria-pressed={game.isFavorite}
          aria-label={t('archive.favorite')}
          title={t('archive.favorite')}
          onClick={() => {
            void archive.toggleFavorite(game);
          }}
        >
          <Star aria-hidden />
        </button>
      </div>
      <span>
        <ResultBadge result={game.playerResult} score={game.result} />
      </span>
      <span className={styles.detail}>
        {Math.ceil(game.plyCount / PLIES_PER_MOVE)}
      </span>
      <span className={styles.detail}>
        {formatPlayedAt(game.playedAt, i18n.language)}
      </span>
      <div className={styles.actions}>
        {isUnfinished ? (
          <Button variant="primary" disabled={!engine} onClick={playOn}>
            <RotateCcw aria-hidden />
            {t('archive.playOn')}
          </Button>
        ) : (
          <Button
            onClick={() => {
              void archive.open(game.id);
            }}
          >
            <Play aria-hidden />
            {t('archive.review')}
          </Button>
        )}
        <Button
          variant="ghost"
          className={styles.iconButton}
          aria-label={t('archive.export')}
          title={t('archive.export')}
          onClick={() => {
            void archive.exportToFile(game.id);
          }}
        >
          <Download aria-hidden />
        </Button>
        <Button
          variant="ghost"
          className={styles.iconButton}
          aria-label={t('archive.delete')}
          title={t('archive.delete')}
          onClick={() => {
            void archive.remove(game.id);
          }}
        >
          <Trash2 aria-hidden />
        </Button>
      </div>
    </li>
  );
});
