import { useEffect, useState } from 'react';

import type { ClockView, Color } from '@shared/contracts/generated/bindings';

const TICK_MS = 100;

interface Countdown {
  syncMs: number;
  elapsedMs: number;
}

/** Display only: Rust owns the clock and decides flag fall. */
export const useCountdown = (clock: ClockView | null, color: Color) => {
  const [countdown, setCountdown] = useState<Countdown | null>(null);
  const running = clock?.running ?? null;
  const syncMs = clock?.syncMs ?? 0;

  useEffect(() => {
    if (running !== color) {
      return;
    }

    const anchor = performance.now();
    const timer = setInterval(() => {
      setCountdown({ syncMs, elapsedMs: performance.now() - anchor });
    }, TICK_MS);

    return () => {
      clearInterval(timer);
    };
  }, [running, syncMs, color]);

  const stored = color === 'white' ? clock?.whiteMs : clock?.blackMs;
  // A countdown from an older snapshot says nothing about this one.
  const elapsedMs =
    running === color && countdown?.syncMs === syncMs ? countdown.elapsedMs : 0;

  return (stored ?? 0) - elapsedMs;
};
