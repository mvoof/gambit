const CENTIPAWNS_IN_A_PAWN = 100;
const PAWN_DECIMALS = 2;

interface EngineScore {
  centipawns: number | null;
  mateIn: number | null;
}

/** White's point of view: `+0.35`, `-1.20`, `#3`, `#-2`. */
export const formatScore = ({ centipawns, mateIn }: EngineScore) => {
  if (mateIn !== null) {
    return `#${mateIn}`;
  }

  if (centipawns === null) {
    return '';
  }

  const pawns = (centipawns / CENTIPAWNS_IN_A_PAWN).toFixed(PAWN_DECIMALS);

  return centipawns >= 0 ? `+${pawns}` : pawns;
};
