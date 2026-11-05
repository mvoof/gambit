import type { Color } from '@shared/contracts/generated/bindings';

export const BOARD_SIDE = 8;

const FILES = 'abcdefgh';
const LAST_INDEX = BOARD_SIDE - 1;
const FIRST_RANK = 1;
const RANK_INDEX = 1;

export interface BoardCell {
  column: number;
  row: number;
}

/** Column and row from the top-left corner as the player sees the board. */
export const squareToCell = (square: string, orientation: Color): BoardCell => {
  const fileIndex = FILES.indexOf(square.charAt(0));
  const rankIndex = Number(square.charAt(RANK_INDEX)) - FIRST_RANK;

  if (orientation === 'white') {
    return { column: fileIndex, row: LAST_INDEX - rankIndex };
  }

  return { column: LAST_INDEX - fileIndex, row: rankIndex };
};

export const cellToSquare = (cell: BoardCell, orientation: Color) => {
  const fileIndex =
    orientation === 'white' ? cell.column : LAST_INDEX - cell.column;
  const rankIndex = orientation === 'white' ? LAST_INDEX - cell.row : cell.row;

  return `${FILES.charAt(fileIndex)}${rankIndex + FIRST_RANK}`;
};

/** `null` for a point outside the board. */
export const squareAtPoint = (
  offsetX: number,
  offsetY: number,
  boardSize: number,
  orientation: Color
) => {
  const column = Math.floor((offsetX / boardSize) * BOARD_SIDE);
  const row = Math.floor((offsetY / boardSize) * BOARD_SIDE);

  if (column < 0 || column > LAST_INDEX || row < 0 || row > LAST_INDEX) {
    return null;
  }

  return cellToSquare({ column, row }, orientation);
};

export const isLightCell = ({ column, row }: BoardCell) =>
  (column + row) % 2 === 0;

/** Every square, top-left first, as the player sees the board. */
export const orderedSquares = (orientation: Color) =>
  Array.from({ length: BOARD_SIDE * BOARD_SIDE }, (_, index) =>
    cellToSquare(
      { column: index % BOARD_SIDE, row: Math.floor(index / BOARD_SIDE) },
      orientation
    )
  );
