import type { Color, PieceKind } from '@shared/contracts/generated/bindings';

const KIND_LETTERS: Record<PieceKind, string> = {
  king: 'K',
  queen: 'Q',
  rook: 'R',
  bishop: 'B',
  knight: 'N',
  pawn: 'P',
};

/** `wK`, the key of a piece in every piece set. */
export const pieceCode = (color: Color, kind: PieceKind) =>
  `${color === 'white' ? 'w' : 'b'}${KIND_LETTERS[kind]}`;
