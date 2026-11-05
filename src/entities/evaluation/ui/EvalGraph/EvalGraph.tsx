import { useState } from 'react';
import type { MouseEvent } from 'react';
import { useTranslation } from 'react-i18next';

import type { EvalPoint } from '@shared/contracts/generated/bindings';
import { formatScore } from '@shared/lib/format-score';

import styles from './EvalGraph.module.scss';

const WIDTH = 400;
const HEIGHT = 80;
const MIDDLE_Y = HEIGHT / 2;
const FULL_PERCENT = 100;
const MARKER_RADIUS = 3.5;
const EVEN_PERCENT = 50;

interface EvalGraphProps {
  points: EvalPoint[];
  /** Plies of the moves that lost something. */
  mistakePlies: number[];
  activePly: number;
  /** The last ply of the game: the axis does not change as points arrive. */
  lastPly: number;
  onSelect: (ply: number) => void;
}

const xOf = (ply: number, lastPly: number) =>
  lastPly === 0 ? 0 : (ply / lastPly) * WIDTH;

const yOf = (whiteWinPercent: number | null) =>
  HEIGHT - ((whiteWinPercent ?? EVEN_PERCENT) / FULL_PERCENT) * HEIGHT;

const linePath = (points: EvalPoint[], lastPly: number) =>
  points
    .map(({ ply, whiteWinPercent }, index) => {
      const command = index === 0 ? 'M' : 'L';

      return `${command}${xOf(ply, lastPly)} ${yOf(whiteWinPercent)}`;
    })
    .join(' ');

const areaPath = (points: EvalPoint[], lastPly: number) => {
  const first = points.at(0);
  const last = points.at(-1);

  if (!first || !last) {
    return '';
  }

  return `${linePath(points, lastPly)} L${xOf(last.ply, lastPly)} ${MIDDLE_Y} L${xOf(first.ply, lastPly)} ${MIDDLE_Y} Z`;
};

const nearestPoint = (points: EvalPoint[], ply: number) =>
  points.reduce<EvalPoint | null>(
    (nearest, point) =>
      nearest === null ||
      Math.abs(point.ply - ply) < Math.abs(nearest.ply - ply)
        ? point
        : nearest,
    null
  );

/** White's winning chances over the game: above the middle, White is better. */
export const EvalGraph = ({
  points,
  mistakePlies,
  activePly,
  lastPly,
  onSelect,
}: EvalGraphProps) => {
  const { t } = useTranslation();
  const [hovered, setHovered] = useState<EvalPoint | null>(null);
  const active = points.find((point) => point.ply === activePly) ?? null;

  const plyAtPointer = (event: MouseEvent<HTMLButtonElement>) => {
    const { left, width } = event.currentTarget.getBoundingClientRect();

    return Math.round(((event.clientX - left) / width) * lastPly);
  };

  if (points.length === 0) {
    return <p className={styles.empty}>{t('replay.graphPending')}</p>;
  }

  return (
    <div className={styles.graph}>
      <button
        type="button"
        className={styles.plot}
        aria-label={t('replay.graph')}
        onPointerMove={(event) => {
          setHovered(nearestPoint(points, plyAtPointer(event)));
        }}
        onPointerLeave={() => {
          setHovered(null);
        }}
        onClick={(event) => {
          onSelect(nearestPoint(points, plyAtPointer(event))?.ply ?? activePly);
        }}
      >
        <svg
          viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
          preserveAspectRatio="none"
          aria-hidden
        >
          <path className={styles.area} d={areaPath(points, lastPly)} />
          <line
            className={styles.middle}
            x1={0}
            x2={WIDTH}
            y1={MIDDLE_Y}
            y2={MIDDLE_Y}
          />
          <path className={styles.line} d={linePath(points, lastPly)} />
          {active ? (
            <line
              className={styles.cursor}
              x1={xOf(active.ply, lastPly)}
              x2={xOf(active.ply, lastPly)}
              y1={0}
              y2={HEIGHT}
            />
          ) : null}
          {hovered ? (
            <line
              className={styles.crosshair}
              x1={xOf(hovered.ply, lastPly)}
              x2={xOf(hovered.ply, lastPly)}
              y1={0}
              y2={HEIGHT}
            />
          ) : null}
          {points
            .filter((point) => mistakePlies.includes(point.ply))
            .map((point) => (
              <circle
                key={point.ply}
                className={styles.mistake}
                cx={xOf(point.ply, lastPly)}
                cy={yOf(point.whiteWinPercent)}
                r={MARKER_RADIUS}
              />
            ))}
        </svg>
      </button>
      {hovered ? (
        <span className={styles.tooltip}>
          {t('replay.graphPoint', {
            ply: hovered.ply,
            score: formatScore(hovered),
          })}
        </span>
      ) : null}
    </div>
  );
};
