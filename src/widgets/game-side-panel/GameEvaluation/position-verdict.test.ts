import { describe, expect, it } from 'vitest';

import { verdictOf } from './position-verdict';

describe('verdictOf', () => {
  it('reads the size and the side of the score', () => {
    expect(verdictOf({ centipawns: 10, mateIn: null })).toBe('even');
    expect(verdictOf({ centipawns: 38, mateIn: null })).toBe('smallWhite');
    expect(verdictOf({ centipawns: -150, mateIn: null })).toBe('clearBlack');
    expect(verdictOf({ centipawns: 400, mateIn: null })).toBe('winningWhite');
  });

  it('takes a mate as winning for the side that mates', () => {
    expect(verdictOf({ centipawns: null, mateIn: -3 })).toBe('winningBlack');
  });
});
