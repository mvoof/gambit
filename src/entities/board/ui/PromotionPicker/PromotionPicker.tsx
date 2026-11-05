import { useTranslation } from 'react-i18next';

import type {
  Color,
  MoveOption,
  PieceKind,
  PieceSetView,
} from '@shared/contracts/generated/bindings';

import { BOARD_SIDE, squareToCell } from '../../board-geometry';
import { pieceCode } from '@shared/lib/piece-code';
import styles from './PromotionPicker.module.scss';

const FULL_PERCENT = 100;
const CELL_PERCENT = FULL_PERCENT / BOARD_SIDE;
const PROMOTION_ORDER: PieceKind[] = ['queen', 'knight', 'rook', 'bishop'];

interface PromotionPickerProps {
  to: string;
  color: Color;
  orientation: Color;
  options: MoveOption[];
  images: PieceSetView['images'];
  onChoose: (uci: string) => void;
  onCancel: () => void;
}

export const PromotionPicker = ({
  to,
  color,
  orientation,
  options,
  images,
  onChoose,
  onCancel,
}: PromotionPickerProps) => {
  const { t } = useTranslation();
  const { column, row } = squareToCell(to, orientation);
  const opensDownward = row === 0;
  const columnStyle = {
    left: `${column * CELL_PERCENT}%`,
    width: `${CELL_PERCENT}%`,
  };

  return (
    <div className={styles.picker}>
      <button
        type="button"
        className={styles.scrim}
        aria-label={t('common.cancel')}
        onClick={onCancel}
      />
      <div
        className={opensDownward ? styles.column : styles.columnUpward}
        style={columnStyle}
      >
        {PROMOTION_ORDER.map((kind) => {
          const option = options.find((entry) => entry.promotion === kind);

          if (!option) {
            return null;
          }

          return (
            <button
              key={kind}
              type="button"
              className={styles.choice}
              aria-label={t(`pieces.${kind}`)}
              onClick={() => {
                onChoose(option.uci);
              }}
            >
              <img src={images[pieceCode(color, kind)]} alt="" />
            </button>
          );
        })}
      </div>
    </div>
  );
};
