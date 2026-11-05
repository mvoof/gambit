import type { CSSProperties } from 'react';

import type {
  BoardPiece,
  BoardSettings,
  Color,
  LastMove,
  MoveOption,
  PieceSetView,
} from '@shared/contracts/generated/bindings';

import { useBoardGestures } from '../../use-board-gestures';
import { BoardArrows } from '../BoardArrows/BoardArrows';
import { BoardPieces } from '../BoardPieces/BoardPieces';
import { BoardSquares } from '../BoardSquares/BoardSquares';
import { PromotionPicker } from '../PromotionPicker/PromotionPicker';
import styles from './Board.module.scss';

const NO_OPTIONS: MoveOption[] = [];

const doNothing = () => undefined;

interface BoardProps {
  pieces: BoardPiece[];
  images: PieceSetView['images'];
  orientation: Color;
  settings: BoardSettings;
  moveOptions?: MoveOption[];
  premoveOptions?: MoveOption[];
  premove?: MoveOption | null;
  lastMove?: LastMove | null;
  checkSquare?: string | null;
  onMove?: (uci: string) => void;
  onPremove?: (uci: string) => void;
  onCancelPremove?: () => void;
}

export const Board = ({
  pieces,
  images,
  orientation,
  settings,
  moveOptions = NO_OPTIONS,
  premoveOptions = NO_OPTIONS,
  premove = null,
  lastMove = null,
  checkSquare = null,
  onMove = doNothing,
  onPremove = doNothing,
  onCancelPremove = doNothing,
}: BoardProps) => {
  const gestures = useBoardGestures({
    pieces,
    orientation,
    moveOptions,
    premoveOptions,
    hasPremove: premove !== null,
    onMove,
    onPremove,
    onCancelPremove,
  });

  // The colours are the user's own, so they cannot be classes.
  const boardColors = {
    '--square-light': settings.lightSquare,
    '--square-dark': settings.darkSquare,
  } as CSSProperties;

  const promotionColor = pieces.find(
    (piece) => piece.square === gestures.promotion?.options[0]?.from
  )?.color;

  return (
    <div className={styles.frame} data-testid="board">
      <div
        className={styles.board}
        style={boardColors}
        {...gestures.boardHandlers}
      >
        <BoardSquares
          orientation={orientation}
          settings={settings}
          selected={gestures.selected}
          targets={gestures.targets}
          marks={gestures.marks}
          lastMove={lastMove}
          checkSquare={checkSquare}
          premove={premove}
        />
        <BoardPieces
          pieces={pieces}
          images={images}
          orientation={orientation}
          drag={gestures.drag}
        />
        <BoardArrows arrows={gestures.arrows} orientation={orientation} />
      </div>
      {gestures.promotion && promotionColor ? (
        <PromotionPicker
          to={gestures.promotion.to}
          color={promotionColor}
          orientation={orientation}
          options={gestures.promotion.options}
          images={images}
          onChoose={gestures.choosePromotion}
          onCancel={gestures.cancelPromotion}
        />
      ) : null}
    </div>
  );
};
