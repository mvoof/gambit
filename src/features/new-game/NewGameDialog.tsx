import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useEngineStore } from '@entities/engine/engine-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import type {
  ColorChoice,
  NewGameOptions,
} from '@shared/contracts/generated/bindings';
import { timeControlOf } from '@shared/lib/time-control';
import { Button } from '@shared/ui/Button/Button';
import { Dialog } from '@shared/ui/Dialog/Dialog';
import { Segmented } from '@shared/ui/Segmented/Segmented';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import { CUSTOM_PRESET, TIME_PRESETS, UNLIMITED_PRESET } from './time-presets';
import styles from './NewGameDialog.module.scss';

const DEFAULT_PRESET = UNLIMITED_PRESET;
const DEFAULT_ELO = 1500;
const DEFAULT_CUSTOM_MINUTES = 7;
const DEFAULT_CUSTOM_INCREMENT = 5;
const ELO_STEP = 50;
const MAX_CUSTOM_MINUTES = 180;
const MAX_CUSTOM_INCREMENT = 60;

interface NewGameDialogProps {
  isOpen: boolean;
  isTrainingByDefault?: boolean;
  onClose: () => void;
}

export const NewGameDialog = observer(
  ({ isOpen, isTrainingByDefault = false, onClose }: NewGameDialogProps) => {
    const { t } = useTranslation();
    const game = useGameStore();
    const navigation = useNavigationStore();
    const { engine } = useEngineStore();
    const settingsStore = useSettingsStore();
    const [color, setColor] = useState<ColorChoice>('white');
    const [preset, setPreset] = useState(DEFAULT_PRESET);
    const [customMinutes, setCustomMinutes] = useState(DEFAULT_CUSTOM_MINUTES);
    const [customIncrement, setCustomIncrement] = useState(
      DEFAULT_CUSTOM_INCREMENT
    );
    const [elo, setElo] = useState(DEFAULT_ELO);
    const [startFen, setStartFen] = useState('');
    const [isTraining, setTraining] = useState(isTrainingByDefault);
    const eloRange = engine?.eloRange ?? null;
    const shownElo = eloRange
      ? Math.min(Math.max(elo, eloRange.min), eloRange.max)
      : elo;

    const timeOptions = [
      ...Object.keys(TIME_PRESETS).map((value) => ({ value, label: value })),
      { value: CUSTOM_PRESET, label: t('newGame.custom') },
      { value: UNLIMITED_PRESET, label: t('newGame.unlimited') },
    ];

    const timeControl = () => {
      if (preset === UNLIMITED_PRESET) {
        return null;
      }

      if (preset === CUSTOM_PRESET) {
        return timeControlOf(customMinutes, customIncrement);
      }

      return timeControlOf(...TIME_PRESETS[preset]);
    };

    const start = async () => {
      const options: NewGameOptions = {
        color,
        timeControl: timeControl(),
        elo: shownElo,
        startFen: startFen.trim() || null,
        training: isTraining,
      };

      onClose();
      navigation.request('play');
      await game.newGame(options);
      await settingsStore.update({ lastGame: options });
    };

    return (
      <Dialog title={t('newGame.title')} isOpen={isOpen} onClose={onClose}>
        <div className={styles.form}>
          <section className={styles.field}>
            <span className={styles.label}>{t('newGame.color')}</span>
            <Segmented
              value={color}
              onChange={setColor}
              options={[
                { value: 'white', label: t('newGame.white') },
                { value: 'random', label: t('newGame.random') },
                { value: 'black', label: t('newGame.black') },
              ]}
            />
          </section>
          <section className={styles.field}>
            <span className={styles.label}>{t('newGame.time')}</span>
            <Segmented
              value={preset}
              onChange={setPreset}
              options={timeOptions}
            />
          </section>
          {preset === CUSTOM_PRESET ? (
            <section className={styles.customTime}>
              <label className={styles.field}>
                <span className={styles.label}>{t('newGame.minutes')}</span>
                <input
                  className={styles.input}
                  type="number"
                  min={1}
                  max={MAX_CUSTOM_MINUTES}
                  value={customMinutes}
                  onChange={(event) => {
                    setCustomMinutes(event.target.valueAsNumber || 1);
                  }}
                />
              </label>
              <label className={styles.field}>
                <span className={styles.label}>{t('newGame.increment')}</span>
                <input
                  className={styles.input}
                  type="number"
                  min={0}
                  max={MAX_CUSTOM_INCREMENT}
                  value={customIncrement}
                  onChange={(event) => {
                    setCustomIncrement(event.target.valueAsNumber || 0);
                  }}
                />
              </label>
            </section>
          ) : null}
          {eloRange ? (
            <label className={styles.field}>
              <span className={styles.label}>
                {t('newGame.strength', { elo: shownElo })}
              </span>
              <input
                className={styles.slider}
                type="range"
                min={eloRange.min}
                max={eloRange.max}
                step={ELO_STEP}
                value={shownElo}
                onChange={(event) => {
                  setElo(event.target.valueAsNumber);
                }}
              />
            </label>
          ) : null}
          <label className={styles.field}>
            <span className={styles.label}>{t('newGame.fen')}</span>
            <input
              className={styles.input}
              type="text"
              spellCheck={false}
              placeholder={t('newGame.fenPlaceholder')}
              value={startFen}
              onChange={(event) => {
                setStartFen(event.target.value);
              }}
            />
          </label>
          <Toggle
            label={t('newGame.training')}
            isChecked={isTraining}
            onChange={setTraining}
          />
          <div className={styles.actions}>
            <Button variant="ghost" onClick={onClose}>
              {t('common.cancel')}
            </Button>
            <Button variant="primary" disabled={!engine} onClick={start}>
              {t('newGame.start')}
            </Button>
          </div>
        </div>
      </Dialog>
    );
  }
);
