import { observer } from 'mobx-react-lite';

import { Board } from '@entities/board/ui/Board/Board';
import { ResizableBoardFrame } from '@entities/board/ui/ResizableBoardFrame/ResizableBoardFrame';
import { useGameStore } from '@entities/game/game-context';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';

/** The live game on the board, or the position being browsed. */
export const GameBoard = observer(() => {
  const game = useGameStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { view, browsedFrame, orientation } = game;

  if (!settings || !view) {
    return null;
  }

  const shown = browsedFrame ?? view;

  return (
    <ResizableBoardFrame>
      <Board
        pieces={shown.pieces}
        images={pieceSets.imagesFor(settings.pieceSet)}
        orientation={orientation}
        settings={settings.board}
        lastMove={shown.lastMove}
        checkSquare={shown.checkSquare}
        moveOptions={browsedFrame ? undefined : view.legalMoves}
        premoveOptions={browsedFrame ? undefined : view.premoveOptions}
        premove={browsedFrame ? null : view.premove}
        onMove={game.move}
        onPremove={game.setPremove}
        onCancelPremove={game.cancelPremove}
      />
    </ResizableBoardFrame>
  );
});
