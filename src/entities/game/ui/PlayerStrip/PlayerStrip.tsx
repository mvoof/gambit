import { User } from 'lucide-react';

import { pieceCode } from '@shared/lib/piece-code';
import { ClockFace } from '../ClockFace/ClockFace';
import type {
  ClockView,
  Color,
  PieceKind,
  PieceSetView,
} from '@shared/contracts/generated/bindings';

import styles from './PlayerStrip.module.scss';

interface PlayerStripProps {
  name: string;
  /** The opponent's strength; a player has none to show. */
  elo?: number | null;
  color: Color;
  clock: ClockView | null;
  /** The pieces this player has taken. */
  captured: PieceKind[];
  /** Points this player is ahead by; shown only when positive. */
  advantage: number;
  images: PieceSetView['images'];
}

export const PlayerStrip = ({
  name,
  elo = null,
  color,
  clock,
  captured,
  advantage,
  images,
}: PlayerStripProps) => {
  const capturedColor = color === 'white' ? 'black' : 'white';

  return (
    <div className={styles.strip}>
      <span className={styles.avatar}>
        <User aria-hidden />
      </span>
      <div className={styles.details}>
        <div className={styles.heading}>
          <span className={styles.name}>{name}</span>
          {elo === null ? null : <span className={styles.elo}>≈ {elo}</span>}
        </div>
        <div className={styles.captured}>
          {captured.map((kind, index) => (
            <img
              key={`${kind}${index}`}
              className={styles.capturedPiece}
              src={images[pieceCode(capturedColor, kind)]}
              alt=""
            />
          ))}
          {advantage > 0 ? (
            <span className={styles.advantage}>+{advantage}</span>
          ) : null}
        </div>
      </div>
      {clock ? <ClockFace clock={clock} color={color} /> : null}
    </div>
  );
};
