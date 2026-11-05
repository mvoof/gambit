const FILES = 'abcdefgh';
const RANK_COUNT = 8;
const BLACK_LETTERS = 'kqrbnp';

export interface PlacedPiece {
  /** `wK…bP`, the keys of a piece set. */
  code: string;
  /** From the top left of the board, White at the bottom. */
  column: number;
  row: number;
}

const pieceCodeOf = (letter: string) => {
  const color = BLACK_LETTERS.includes(letter) ? 'b' : 'w';

  return `${color}${letter.toUpperCase()}`;
};

const isFileSkip = (character: string) => /\d/.test(character);

/** The pieces of a FEN, for a picture of the position and nothing more. */
export const placedPieces = (fen: string): PlacedPiece[] => {
  const ranks = fen.split(' ')[0].split('/').slice(0, RANK_COUNT);

  return ranks.flatMap((rank, row) => {
    let column = 0;

    return rank.split('').flatMap((character) => {
      if (isFileSkip(character)) {
        column += Number(character);

        return [];
      }

      const piece = { code: pieceCodeOf(character), column, row };

      column += 1;

      return column <= FILES.length ? [piece] : [];
    });
  });
};
