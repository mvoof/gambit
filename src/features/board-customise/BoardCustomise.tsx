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
  'animateMoves',
];

const COLORS: BoardColorKey[] = ['lightSquare', 'darkSquare'];

const PRESETS = [
  { name: 'sand', lightSquare: '#f0e4cf', darkSquare: '#c9a97c' },
  { name: 'walnut', lightSquare: '#e3d3b5', darkSquare: '#8a6a4a' },
  { name: 'slate', lightSquare: '#d9d7d2', darkSquare: '#807d78' },
  { name: 'moss', lightSquare: '#e6e8cf', darkSquare: '#7c955b' },
  { name: 'river', lightSquare: '#dfe8ef', darkSquare: '#6f8fae' },
];

export const BoardCustomise = observer(() => {
  const { t } = useTranslation();
  const store = useSettingsStore();
  const board = store.settings?.board;

  if (!board) {
    return null;
  }

  return (
    <div className={styles.customise}>
      <div className={styles.presets}>
        {PRESETS.map(({ name, lightSquare, darkSquare }) => (
          <button
            key={name}
            type="button"
            className={
              board.darkSquare === darkSquare &&
              board.lightSquare === lightSquare
                ? `${styles.preset} ${styles.active}`
                : styles.preset
            }
            aria-label={t(`board.preset.${name}`)}
            title={t(`board.preset.${name}`)}
            // The swatch shows the very colours the preset sets.
            style={{
              background: `linear-gradient(135deg, ${lightSquare} 50%, ${darkSquare} 50%)`,
            }}
            onClick={() => {
              void store.updateBoard({ lightSquare, darkSquare });
            }}
          />
        ))}
      </div>
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
