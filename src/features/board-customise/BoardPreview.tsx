import { observer } from 'mobx-react-lite';

import { MiniBoard } from '@entities/board/ui/MiniBoard/MiniBoard';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';

import styles from './BoardPreview.module.scss';

const PREVIEW_FEN =
  'r3k2r/pp3ppp/2n1b3/3qp3/3P4/2N1BN2/PP3PPP/R2Q1RK1 w - - 0 1';

/** The board and the pieces as they will look in a game. */
export const BoardPreview = observer(() => {
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();

  if (!settings) {
    return null;
  }

  return (
    <MiniBoard
      className={styles.preview}
      fen={PREVIEW_FEN}
      images={pieceSets.imagesFor(settings.pieceSet)}
      lightSquare={settings.board.lightSquare}
      darkSquare={settings.board.darkSquare}
    />
  );
});
