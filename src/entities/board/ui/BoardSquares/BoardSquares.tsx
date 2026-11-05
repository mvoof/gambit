import { memo, useMemo } from 'react';

import type {
  BoardSettings,
  Color,
  LastMove,
  MoveOption,
} from '@shared/contracts/generated/bindings';

import {
  BOARD_SIDE,
  isLightCell,
  orderedSquares,
  squareToCell,
} from '../../board-geometry';
import styles from './BoardSquares.module.scss';

const LAST_INDEX = BOARD_SIDE - 1;
const RANK_CHAR_INDEX = 1;

interface MoveTarget {
  square: string;
  isCapture: boolean;
}

interface BoardSquaresProps {
  orientation: Color;
  settings: BoardSettings;
  selected: string | null;
  targets: MoveTarget[];
  marks: string[];
  lastMove: LastMove | null;
  checkSquare: string | null;
  premove: MoveOption | null;
}

const joinClasses = (entries: [string, boolean][]) =>
  entries
    .filter(([, isOn]) => isOn)
    .map(([className]) => className)
    .join(' ');

const BoardSquaresBase = ({
  orientation,
  settings,
  selected,
  targets,
  marks,
  lastMove,
  checkSquare,
  premove,
}: BoardSquaresProps) => {
  const squares = useMemo(() => orderedSquares(orientation), [orientation]);

  return (
    <div className={styles.grid}>
      {squares.map((square) => {
        const cell = squareToCell(square, orientation);
        const isLight = isLightCell(cell);
        const target = targets.find((entry) => entry.square === square);
        const isLast =
          settings.showLastMove &&
          (lastMove?.from === square || lastMove?.to === square);
        const showsFile = settings.showCoordinates && cell.row === LAST_INDEX;
        const showsRank = settings.showCoordinates && cell.column === 0;

        return (
          <div
            key={square}
            data-square={square}
            className={joinClasses([
              [styles.square, true],
              [styles.light, isLight],
              [styles.dark, !isLight],
              [styles.lastMove, isLast],
              [styles.selected, selected === square],
              [
                styles.premove,
                premove?.from === square || premove?.to === square,
              ],
              [styles.check, settings.showCheck && checkSquare === square],
              [styles.marked, marks.includes(square)],
            ])}
          >
            {showsRank ? (
              <span className={styles.rankLabel}>
                {square.charAt(RANK_CHAR_INDEX)}
              </span>
            ) : null}
            {showsFile ? (
              <span className={styles.fileLabel}>{square.charAt(0)}</span>
            ) : null}
            {target && settings.showLegalMoves ? (
              <span
                className={
                  target.isCapture ? styles.captureCorners : styles.dot
                }
              />
            ) : null}
          </div>
        );
      })}
    </div>
  );
};

export const BoardSquares = memo(BoardSquaresBase);
