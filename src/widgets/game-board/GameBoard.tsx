import { observer } from 'mobx-react-lite';

import { Board } from '@entities/board/ui/Board/Board';
import { ResizableBoardFrame } from '@entities/board/ui/ResizableBoardFrame/ResizableBoardFrame';
import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import { useGameStore } from '@entities/game/game-context';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';

/** The live game on the board, or the position being browsed. */
export const GameBoard = observer(() => {
  const game = useGameStore();
  const evaluation = useEvaluationStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { view, browsedFrame, orientation } = game;

  if (!settings || !view) {
    return null;
  }

  const shown = browsedFrame ?? view;
  const canHint =
    view.isTraining &&
    !browsedFrame &&
    !view.outcome &&
    view.turn === view.humanColor;

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
        hint={canHint ? evaluation.bestMoveFor(view.fen) : null}
        onMove={game.move}
        onPremove={game.setPremove}
        onCancelPremove={game.cancelPremove}
      />
    </ResizableBoardFrame>
  );
});
