import { describe, expect, it } from 'vitest';

import { formatScore } from './format-score';

describe('formatScore', () => {
  it('shows pawns with a sign', () => {
    expect(formatScore({ centipawns: 35, mateIn: null })).toBe('+0.35');
    expect(formatScore({ centipawns: -120, mateIn: null })).toBe('-1.20');
    expect(formatScore({ centipawns: 0, mateIn: null })).toBe('+0.00');
  });

  it('prefers a mate', () => {
    expect(formatScore({ centipawns: null, mateIn: 3 })).toBe('#3');
    expect(formatScore({ centipawns: null, mateIn: -2 })).toBe('#-2');
  });
});
