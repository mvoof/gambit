import { describe, expect, it } from 'vitest';

import { formatSize } from './format-size';

describe('formatSize', () => {
  it('uses megabytes for a large archive', () => {
    expect(formatSize(13_002_342, 'en')).toBe('12.4 MB');
  });

  it('uses whole kilobytes for a small one', () => {
    expect(formatSize(86_016, 'en')).toBe('84 kB');
  });
});
