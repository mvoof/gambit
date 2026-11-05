import { Check } from 'lucide-react';
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

const asWritten = (san: string) => san;

interface MoveListProps {
  moves: MoveRecord[];
  marks?: MoveMark[];
  /** The ply shown on the board: 0 before the first move. */
  activePly: number;
  /** Only the rows with these plies stay, and the rest of a row fades. */
  highlightedPlies?: number[] | null;
  /** Moves that get a tick: what the engine itself would have played. */
  bestPlies?: number[];
  /** Writes a move the way the player wants to read it. */
  formatMove?: (san: string) => string;
  onSelect: (ply: number) => void;
}

interface MoveRow {
  number: number;
  moves: MoveRecord[];
}

const toRows = (moves: MoveRecord[]): MoveRow[] => {
  const rows: MoveRow[] = [];
  const startsWithBlack = moves[0]?.color === 'black';
  const padded = startsWithBlack ? [null, ...moves] : moves;

  for (let index = 0; index < padded.length; index += PLIES_PER_ROW) {
    rows.push({
      number: index / PLIES_PER_ROW + 1,
      moves: padded
        .slice(index, index + PLIES_PER_ROW)
        .filter((move): move is MoveRecord => move !== null),
    });
  }

  return rows;
};

const MoveGlyph = ({ quality }: { quality?: MoveQuality }) =>
  quality ? (
    <span className={styles[quality]}>{QUALITY_GLYPHS[quality]}</span>
  ) : null;

const moveClass = (isActive: boolean, isFaded: boolean) =>
  [styles.move, isActive ? styles.active : '', isFaded ? styles.faded : '']
    .filter(Boolean)
    .join(' ');

export const MoveList = ({
  moves,
  marks = NO_MARKS,
  activePly,
  highlightedPlies = null,
  bestPlies,
  formatMove = asWritten,
  onSelect,
}: MoveListProps) => {
  const activeRef = useRef<HTMLButtonElement>(null);
  const rows = toRows(moves).filter(
    (row) =>
      highlightedPlies === null ||
      row.moves.some((move) => highlightedPlies.includes(move.ply))
  );

  useEffect(() => {
    activeRef.current?.scrollIntoView({ block: 'nearest' });
  }, [activePly]);

  return (
    <ol className={styles.list}>
      {rows.map((row) => (
        <li key={row.number} className={styles.row}>
          <span className={styles.number}>{row.number}.</span>
          {row.moves.map((move) => (
            <button
              key={move.ply}
              ref={move.ply === activePly ? activeRef : null}
              type="button"
              className={moveClass(
                move.ply === activePly,
                highlightedPlies !== null &&
                  !highlightedPlies.includes(move.ply)
              )}
              data-color={move.color}
              onClick={() => {
                onSelect(move.ply);
              }}
            >
              {bestPlies?.includes(move.ply) ? (
                <Check className={styles.best} aria-hidden />
              ) : null}
              {formatMove(move.san)}
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
