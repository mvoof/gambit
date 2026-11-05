const FIGURINES: Record<string, string> = {
  K: '♚',
  Q: '♛',
  R: '♜',
  B: '♝',
  N: '♞',
};

const PIECE_LETTERS = /[KQRBN]/g;

/** `Nf3` as `♞f3`. Files are lower case, so `b` never reads as a bishop, and
 * the filled glyphs take the colour of the text around them. */
export const figurineSan = (san: string) =>
  san.replace(PIECE_LETTERS, (letter) => FIGURINES[letter]);
