import { describe, expect, it } from 'vitest';

import { formatNumber, normalizeSpaces } from './format';

describe('formatNumber', () => {
  it('groups thousands for the given locale', () => {
    expect(formatNumber(1234567, 'en')).toBe('1,234,567');
  });

  it('leaves small numbers untouched', () => {
    expect(formatNumber(7, 'en')).toBe('7');
  });
});

describe('normalizeSpaces', () => {
  it('collapses whitespace runs', () => {
    expect(normalizeSpaces('  hello   world  ')).toBe('hello world');
  });
});
