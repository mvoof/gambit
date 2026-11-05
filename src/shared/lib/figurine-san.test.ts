import { describe, expect, it } from 'vitest';

import { figurineSan } from './figurine-san';

describe('figurineSan', () => {
  it('writes the piece as a figurine', () => {
    expect(figurineSan('Nf3')).toBe('♞f3');
    expect(figurineSan('Bxf5+')).toBe('♝xf5+');
    expect(figurineSan('Qg7#')).toBe('♛g7#');
  });

  it('leaves pawn moves and files alone', () => {
    expect(figurineSan('e4')).toBe('e4');
    expect(figurineSan('bxc3')).toBe('bxc3');
  });

  it('keeps castling, which names no piece', () => {
    expect(figurineSan('O-O-O')).toBe('O-O-O');
  });

  it('writes the promoted piece too', () => {
    expect(figurineSan('e8=Q+')).toBe('e8=♛+');
  });

  it('keeps the numbers of a line', () => {
    expect(figurineSan('12... Nc6')).toBe('12... ♞c6');
  });
});
