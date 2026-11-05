import { describe, expect, it } from 'vitest';

import { eloLevels, FULL_STRENGTH, nearestLevel } from './elo-levels';

describe('eloLevels', () => {
  it('stays inside the range, on round steps', () => {
    expect(eloLevels({ min: 1320, max: 1600 })).toEqual([1400, 1500, 1600]);
  });
});

describe('nearestLevel', () => {
  it('picks the closest step', () => {
    expect(nearestLevel([1400, 1500], 1480)).toBe('1500');
  });

  it('falls back to full strength with no steps', () => {
    expect(nearestLevel([], 1500)).toBe(FULL_STRENGTH);
  });
});
