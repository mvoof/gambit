import type { Color } from '@shared/contracts/generated/bindings';

import { BOARD_SIDE, squareToCell } from '../../board-geometry';
import type { BoardArrow } from '../../use-board-gestures';
import styles from './BoardArrows.module.scss';

const CELL_UNITS = 100;
const HALF_CELL = CELL_UNITS / 2;
const BOARD_UNITS = BOARD_SIDE * CELL_UNITS;
const ARROW_MARKER_ID = 'board-arrow-head';
const HINT_MARKER_ID = 'board-hint-head';
/** The head must end on the centre of the square, not past it. */
const HEAD_LENGTH = 26;

interface BoardArrowsProps {
  arrows: BoardArrow[];
  /** The engine's move, apart from the player's own drawings. */
  hint?: BoardArrow | null;
  orientation: Color;
}

const cellCenter = (square: string, orientation: Color) => {
  const { column, row } = squareToCell(square, orientation);

  return {
    centerX: column * CELL_UNITS + HALF_CELL,
    centerY: row * CELL_UNITS + HALF_CELL,
  };
};

const arrowLine = ({ from, to }: BoardArrow, orientation: Color) => {
  const start = cellCenter(from, orientation);
  const end = cellCenter(to, orientation);
  const deltaX = end.centerX - start.centerX;
  const deltaY = end.centerY - start.centerY;
  const length = Math.hypot(deltaX, deltaY);
  const shorten = (length - HEAD_LENGTH) / length;

  return {
    x1: start.centerX,
    y1: start.centerY,
    x2: start.centerX + deltaX * shorten,
    y2: start.centerY + deltaY * shorten,
  };
};

export const BoardArrows = ({
  arrows,
  hint,
  orientation,
}: BoardArrowsProps) => (
  <svg
    className={styles.layer}
    viewBox={`0 0 ${BOARD_UNITS} ${BOARD_UNITS}`}
    aria-hidden="true"
  >
    <defs>
      <marker
        id={ARROW_MARKER_ID}
        markerWidth="3"
        markerHeight="3"
        refX="0.5"
        refY="1.5"
        orient="auto"
      >
        <path className={styles.head} d="M0,0 L3,1.5 L0,3 Z" />
      </marker>
      <marker
        id={HINT_MARKER_ID}
        markerWidth="3"
        markerHeight="3"
        refX="0.5"
        refY="1.5"
        orient="auto"
      >
        <path className={styles.hintHead} d="M0,0 L3,1.5 L0,3 Z" />
      </marker>
    </defs>
    {arrows.map((arrow) => (
      <line
        key={`${arrow.from}${arrow.to}`}
        className={styles.line}
        markerEnd={`url(#${ARROW_MARKER_ID})`}
        {...arrowLine(arrow, orientation)}
      />
    ))}
    {hint ? (
      <line
        className={styles.hintLine}
        markerEnd={`url(#${HINT_MARKER_ID})`}
        {...arrowLine(hint, orientation)}
      />
    ) : null}
  </svg>
);
