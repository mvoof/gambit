import { useEffect, useRef } from 'react';

import type {
  MoveMark,
  MoveQuality,
  MoveRecord,
} from '@shared/contracts/generated/bindings';

import styles from './MoveList.module.scss';

const PLIES_PER_ROW = 2;
const QUALITY_GLYPHS: Record<MoveQuality, string> = {
  inaccuracy: '?!',
  mistake: '?',
  blunder: '??',
};
const NO_MARKS: MoveMark[] = [];

interface MoveListProps {
  moves: MoveRecord[];
  marks?: MoveMark[];
  /** The ply shown on the board: 0 before the first move. */
  activePly: number;
  onSelect: (ply: number) => void;
}

const toRows = (moves: MoveRecord[]) => {
  const rows: MoveRecord[][] = [];
  const startsWithBlack = moves[0]?.color === 'black';
  const padded = startsWithBlack ? [null, ...moves] : moves;

  for (let index = 0; index < padded.length; index += PLIES_PER_ROW) {
    rows.push(
      padded
        .slice(index, index + PLIES_PER_ROW)
        .filter((move): move is MoveRecord => move !== null)
    );
  }

  return rows;
};

const MoveGlyph = ({ quality }: { quality?: MoveQuality }) =>
  quality ? (
    <span className={styles[quality]}>{QUALITY_GLYPHS[quality]}</span>
  ) : null;

export const MoveList = ({
  moves,
  marks = NO_MARKS,
  activePly,
  onSelect,
}: MoveListProps) => {
  const activeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    activeRef.current?.scrollIntoView({ block: 'nearest' });
  }, [activePly]);

  return (
    <ol className={styles.list}>
      {toRows(moves).map((row, rowIndex) => (
        <li key={row[0].ply} className={styles.row}>
          <span className={styles.number}>{rowIndex + 1}.</span>
          {row.map((move) => (
            <button
              key={move.ply}
              ref={move.ply === activePly ? activeRef : null}
              type="button"
              className={
                move.ply === activePly
                  ? `${styles.move} ${styles.active}`
                  : styles.move
              }
              data-color={move.color}
              onClick={() => {
                onSelect(move.ply);
              }}
            >
              {move.san}
              <MoveGlyph
                quality={marks.find((mark) => mark.ply === move.ply)?.quality}
              />
            </button>
          ))}
        </li>
      ))}
    </ol>
  );
};
