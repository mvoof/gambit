import { describe, expect, it } from 'vitest';

import { formatClock } from './format-clock';

describe('formatClock', () => {
  it('shows minutes and seconds', () => {
    expect(formatClock(300_000)).toBe('5:00');
    expect(formatClock(61_000)).toBe('1:01');
  });

  it('rounds up, so a full minute reads as one', () => {
    expect(formatClock(59_001)).toBe('1:00');
  });

  it('shows hours for long games', () => {
    expect(formatClock(5_400_000)).toBe('1:30:00');
  });

  it('shows tenths under ten seconds', () => {
    expect(formatClock(9_950)).toBe('0:09.9');
    expect(formatClock(450)).toBe('0:00.4');
  });

  it('never goes below zero', () => {
    expect(formatClock(-5)).toBe('0:00.0');
  });
});
