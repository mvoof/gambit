export const UNLIMITED_PRESET = 'unlimited';
export const CUSTOM_PRESET = 'custom';

/** Minutes and increment seconds, in the `5+3` notation players use. */
export const TIME_PRESETS: Record<string, [number, number]> = {
  '1+0': [1, 0],
  '3+2': [3, 2],
  '5+0': [5, 0],
  '10+0': [10, 0],
  '15+10': [15, 10],
};
