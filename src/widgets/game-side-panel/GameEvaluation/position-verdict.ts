const SMALL_EDGE_CENTIPAWNS = 25;
const CLEAR_EDGE_CENTIPAWNS = 100;
const WINNING_CENTIPAWNS = 300;

export type PositionVerdict =
  | 'even'
  | 'smallWhite'
  | 'smallBlack'
  | 'clearWhite'
  | 'clearBlack'
  | 'winningWhite'
  | 'winningBlack';

interface EngineScore {
  centipawns: number | null;
  mateIn: number | null;
}

const bySide = (
  isWhite: boolean,
  white: PositionVerdict,
  black: PositionVerdict
) => (isWhite ? white : black);

/** The words for the number the engine gave, White's point of view. */
export const verdictOf = ({
  centipawns,
  mateIn,
}: EngineScore): PositionVerdict => {
  if (mateIn !== null) {
    return bySide(mateIn > 0, 'winningWhite', 'winningBlack');
  }

  const score = centipawns ?? 0;
  const size = Math.abs(score);

  if (size < SMALL_EDGE_CENTIPAWNS) {
    return 'even';
  }

  if (size < CLEAR_EDGE_CENTIPAWNS) {
    return bySide(score > 0, 'smallWhite', 'smallBlack');
  }

  if (size < WINNING_CENTIPAWNS) {
    return bySide(score > 0, 'clearWhite', 'clearBlack');
  }

  return bySide(score > 0, 'winningWhite', 'winningBlack');
};
