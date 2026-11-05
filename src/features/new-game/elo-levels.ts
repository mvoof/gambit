import type { EloRange } from '@shared/contracts/generated/bindings';

const LEVEL_STEP = 100;

export const FULL_STRENGTH = 'full';

/** Round steps inside what the engine supports. */
export const eloLevels = ({ min, max }: EloRange) => {
  const levels: number[] = [];
  const first = Math.ceil(min / LEVEL_STEP) * LEVEL_STEP;

  for (let elo = first; elo <= max; elo += LEVEL_STEP) {
    levels.push(elo);
  }

  return levels;
};

export const nearestLevel = (levels: number[], elo: number) => {
  const nearest = levels.reduce<number | null>(
    (best, level) =>
      best === null || Math.abs(level - elo) < Math.abs(best - elo)
        ? level
        : best,
    null
  );

  return nearest === null ? FULL_STRENGTH : String(nearest);
};
