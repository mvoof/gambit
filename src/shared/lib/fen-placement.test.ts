import { describe, expect, it } from 'vitest';

import { placedPieces } from './fen-placement';

describe('placedPieces', () => {
  it('reads the starting position', () => {
    const pieces = placedPieces(
      'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
    );

    expect(pieces).toHaveLength(32);
    expect(pieces[0]).toEqual({ code: 'bR', column: 0, row: 0 });
    expect(pieces.at(-1)).toEqual({ code: 'wR', column: 7, row: 7 });
  });

  it('skips the empty squares', () => {
    expect(placedPieces('8/8/3k4/8/8/4K3/8/8 w - - 0 1')).toEqual([
      { code: 'bK', column: 3, row: 2 },
      { code: 'wK', column: 4, row: 5 },
    ]);
  });
});
