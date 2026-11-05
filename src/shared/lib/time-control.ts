import type { TimeControl } from '@shared/contracts/generated/bindings';

const MS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;

export const timeControlOf = (
  minutes: number,
  incrementSeconds: number
): TimeControl => ({
  baseMs: minutes * SECONDS_PER_MINUTE * MS_PER_SECOND,
  incrementMs: incrementSeconds * MS_PER_SECOND,
});
