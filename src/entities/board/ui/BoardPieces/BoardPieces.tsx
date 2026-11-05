import type {
  BoardPiece,
  Color,
  PieceSetView,
} from '@shared/contracts/generated/bindings';

import { BOARD_SIDE, squareToCell } from '../../board-geometry';
import { pieceCode } from '@shared/lib/piece-code';
import styles from './BoardPieces.module.scss';

const FULL_CELL_PERCENT = 100;
const HALF_CELL_PERCENT = 50;

interface DragState {
  square: string;
  pointerX: number;
  pointerY: number;
}

interface BoardPiecesProps {
  pieces: BoardPiece[];
  images: PieceSetView['images'];
  orientation: Color;
  drag: DragState | null;
}

/** Translated in percent of the piece itself, so it needs no board size. */
const restingTransform = (square: string, orientation: Color) => {
  const { column, row } = squareToCell(square, orientation);

  return `translate(${column * FULL_CELL_PERCENT}%, ${row * FULL_CELL_PERCENT}%)`;
};

/** Centred on the pointer: the pointer is in pixels, the centring in percent. */
const dragTransform = ({ pointerX, pointerY }: DragState) =>
  `translate(calc(${pointerX}px - ${HALF_CELL_PERCENT}%), calc(${pointerY}px - ${HALF_CELL_PERCENT}%))`;

export const BoardPieces = ({
  pieces,
  images,
  orientation,
  drag,
}: BoardPiecesProps) => (
  <div className={styles.layer}>
    {pieces.map((piece) => {
      const isDragged = drag?.square === piece.square;
      const classNames = isDragged
        ? `${styles.piece} ${styles.dragged}`
        : styles.piece;

      return (
        <img
          key={piece.id}
          className={classNames}
          src={images[pieceCode(piece.color, piece.kind)]}
          alt=""
          draggable={false}
          style={{
            width: `${FULL_CELL_PERCENT / BOARD_SIDE}%`,
            transform: isDragged
              ? dragTransform(drag)
              : restingTransform(piece.square, orientation),
          }}
        />
      );
    })}
  </div>
);
