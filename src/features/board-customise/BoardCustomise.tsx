import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useSettingsStore } from '@entities/settings/settings-context';
import type { BoardSettings } from '@shared/contracts/generated/bindings';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import styles from './BoardCustomise.module.scss';

type BoardToggleKey = Exclude<
  keyof BoardSettings,
  'lightSquare' | 'darkSquare'
>;
type BoardColorKey = 'lightSquare' | 'darkSquare';

const TOGGLES: BoardToggleKey[] = [
  'showCoordinates',
  'showLegalMoves',
  'showLastMove',
  'showCheck',
];

const COLORS: BoardColorKey[] = ['lightSquare', 'darkSquare'];

export const BoardCustomise = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const board = store.settings?.board;

  if (!board) {
    return null;
  }

  return (
    <div className={styles.customise}>
      <div className={styles.colors}>
        {COLORS.map((key) => (
          <label key={key} className={styles.color}>
            <input
              type="color"
              className={styles.swatch}
              value={board[key]}
              onChange={(event) => {
                void store.updateBoard({ [key]: event.target.value });
              }}
            />
            <span>{t(`board.${key}`)}</span>
          </label>
        ))}
      </div>
      {TOGGLES.map((key) => (
        <Toggle
          key={key}
          label={t(`board.${key}`)}
          isChecked={board[key]}
          onChange={(isChecked) => {
            void store.updateBoard({ [key]: isChecked });
          }}
        />
      ))}
    </div>
  );
});
