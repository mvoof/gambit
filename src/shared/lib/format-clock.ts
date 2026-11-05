const MS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
const MINUTES_PER_HOUR = 60;
const TENTHS_DIVISOR = 100;
/** Under this, a clock shows tenths: every one of them counts. */
const TENTHS_THRESHOLD_MS = 10_000;
const PAD_LENGTH = 2;

const pad = (value: number) => String(value).padStart(PAD_LENGTH, '0');

export const formatClock = (remainingMs: number) => {
  const safeMs = Math.max(0, remainingMs);

  if (safeMs < TENTHS_THRESHOLD_MS) {
    const seconds = Math.floor(safeMs / MS_PER_SECOND);
    const tenths = Math.floor((safeMs % MS_PER_SECOND) / TENTHS_DIVISOR);

    return `0:${pad(seconds)}.${tenths}`;
  }

  // Rounded up: a clock that says 1:00 has not started losing that minute.
  const totalSeconds = Math.ceil(safeMs / MS_PER_SECOND);
  const seconds = totalSeconds % SECONDS_PER_MINUTE;
  const totalMinutes = Math.floor(totalSeconds / SECONDS_PER_MINUTE);

  if (totalMinutes < MINUTES_PER_HOUR) {
    return `${totalMinutes}:${pad(seconds)}`;
  }

  const hours = Math.floor(totalMinutes / MINUTES_PER_HOUR);
  const minutes = totalMinutes % MINUTES_PER_HOUR;

  return `${hours}:${pad(minutes)}:${pad(seconds)}`;
};
