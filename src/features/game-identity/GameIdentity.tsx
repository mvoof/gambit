import { observer } from 'mobx-react-lite';

import { MiniBoard } from '@entities/board/ui/MiniBoard/MiniBoard';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import type { GameSummary } from '@shared/contracts/generated/bindings';

import styles from './GameIdentity.module.scss';

interface GameIdentityProps {
  game: GameSummary;
}

const START_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

/** The picture of the last position, with the opponent next to it. */
export const GameIdentity = observer(({ game }: GameIdentityProps) => {
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();

  if (!settings) {
    return null;
  }

  return (
    <div className={styles.identity}>
      <MiniBoard
        className={styles.thumbnail}
        fen={game.finalFen ?? START_FEN}
        images={pieceSets.imagesFor(settings.pieceSet)}
        lightSquare={settings.board.lightSquare}
        darkSquare={settings.board.darkSquare}
      />
      <div className={styles.text}>
        <span className={styles.name}>{game.opponent}</span>
        {game.opponentElo === null ? null : (
          <span className={styles.elo}>≈ {game.opponentElo}</span>
        )}
      </div>
    </div>
  );
});
