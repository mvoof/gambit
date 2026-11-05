import type { CSSProperties } from 'react';

import type { PieceSetView } from '@shared/contracts/generated/bindings';
import { placedPieces } from '@shared/lib/fen-placement';

import { BOARD_SIDE } from '../../board-geometry';
import styles from './MiniBoard.module.scss';

const SQUARES = Array.from(
  { length: BOARD_SIDE * BOARD_SIDE },
  (_, index) => index
);
const FULL_PERCENT = 100;

interface MiniBoardProps {
  fen: string;
  images: PieceSetView['images'];
  lightSquare: string;
  darkSquare: string;
  /** Shows the frame around the position. */
  className?: string;
}

const isDarkSquare = (index: number) =>
  (Math.floor(index / BOARD_SIDE) + (index % BOARD_SIDE)) % 2 === 1;

/** A picture of a position, for a list or a preview. */
export const MiniBoard = ({
  fen,
  images,
  lightSquare,
  darkSquare,
  className,
}: MiniBoardProps) => {
  // The colours are the user's own, so they cannot be classes.
  const boardColors = {
    '--square-light': lightSquare,
    '--square-dark': darkSquare,
  } as CSSProperties;

  return (
    <div
      className={className ? `${styles.board} ${className}` : styles.board}
      style={boardColors}
      aria-hidden
    >
      {SQUARES.map((index) => (
        <span
          key={index}
          className={isDarkSquare(index) ? styles.dark : styles.light}
        />
      ))}
      {placedPieces(fen).map(({ code, column, row }) => (
        <img
          key={`${code}${column}${row}`}
          className={styles.piece}
          src={images[code]}
          alt=""
          style={{
            left: `${(column * FULL_PERCENT) / BOARD_SIDE}%`,
            top: `${(row * FULL_PERCENT) / BOARD_SIDE}%`,
          }}
        />
      ))}
    </div>
  );
};
