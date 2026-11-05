import { Clock3, Flame, Infinity as Unlimited, Play, Zap } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useEngineStore } from '@entities/engine/engine-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { NewGameDialog } from './NewGameDialog';
import type { ColorChoice } from '@shared/contracts/generated/bindings';
import { timeControlOf } from '@shared/lib/time-control';
import { Button } from '@shared/ui/Button/Button';
import { Panel } from '@shared/ui/Panel/Panel';
import { Segmented } from '@shared/ui/Segmented/Segmented';
import { Select } from '@shared/ui/Select/Select';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import { eloLevels, FULL_STRENGTH, nearestLevel } from './elo-levels';
import styles from './StartGameCard.module.scss';

const DEFAULT_ELO = 1500;

interface TimePreset {
  id: string;
  icon: LucideIcon;
  /** Minutes and increment seconds; `null` is a game with no clock. */
  time: [number, number] | null;
}

const TIME_PRESETS: TimePreset[] = [
  { id: 'unlimited', icon: Unlimited, time: null },
  { id: '10 + 0', icon: Zap, time: [10, 0] },
  { id: '5 + 3', icon: Flame, time: [5, 3] },
  { id: '15 + 10', icon: Clock3, time: [15, 10] },
];

/** The one place a game starts from; the rarer options are in the dialog. */
export const StartGameCard = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const navigation = useNavigationStore();
  const { engine } = useEngineStore();
  const settingsStore = useSettingsStore();
  const last = settingsStore.settings?.lastGame;
  const eloRange = engine?.eloRange ?? null;
  const levels = eloRange ? eloLevels(eloRange) : [];
  const [preset, setPreset] = useState(TIME_PRESETS[0]);
  const [color, setColor] = useState<ColorChoice>('random');
  const [level, setLevel] = useState<string | null>(null);
  const [isTraining, setTraining] = useState(false);
  const [isOptionsOpen, setOptionsOpen] = useState(false);
  const shownLevel = level ?? nearestLevel(levels, last?.elo ?? DEFAULT_ELO);

  const start = async () => {
    const options = {
      color,
      timeControl: preset.time ? timeControlOf(...preset.time) : null,
      elo: shownLevel === FULL_STRENGTH ? null : Number(shownLevel),
      startFen: null,
      training: isTraining,
    };

    navigation.request('play');
    await game.newGame(options);
    await settingsStore.update({ lastGame: options });
  };

  return (
    <Panel className={styles.card} title={t('startGame.title')} icon={Play}>
      <div className={styles.presets}>
        {TIME_PRESETS.map((entry) => (
          <button
            key={entry.id}
            type="button"
            aria-pressed={entry === preset}
            className={
              entry === preset
                ? `${styles.preset} ${styles.active}`
                : styles.preset
            }
            onClick={() => {
              setPreset(entry);
            }}
          >
            <entry.icon className={styles.icon} aria-hidden />
            {entry.time ? entry.id : t('newGame.unlimited')}
          </button>
        ))}
      </div>
      <dl className={styles.fields}>
        <dt>{t('startGame.engine')}</dt>
        <dd className={styles.engine}>{engine?.name ?? t('engine.none')}</dd>
        {eloRange ? (
          <>
            <dt>{t('startGame.strength')}</dt>
            <dd>
              <Select
                label={t('startGame.strength')}
                value={shownLevel}
                onChange={setLevel}
                options={[
                  ...levels.map((elo) => ({
                    value: String(elo),
                    label: t('startGame.level', { elo }),
                  })),
                  { value: FULL_STRENGTH, label: t('startGame.fullStrength') },
                ]}
              />
            </dd>
          </>
        ) : null}
        <dt>{t('newGame.color')}</dt>
        <dd>
          <Segmented
            value={color}
            onChange={setColor}
            options={[
              { value: 'white', label: t('newGame.white') },
              { value: 'random', label: t('newGame.random') },
              { value: 'black', label: t('newGame.black') },
            ]}
          />
        </dd>
      </dl>
      <Toggle
        label={t('startGame.training')}
        isChecked={isTraining}
        onChange={setTraining}
      />
      <Button variant="primary" size="large" disabled={!engine} onClick={start}>
        <Play aria-hidden />
        {t('startGame.play')}
      </Button>
      <button
        type="button"
        className={styles.options}
        onClick={() => {
          setOptionsOpen(true);
        }}
      >
        {t('startGame.options')}
      </button>
      <NewGameDialog
        isOpen={isOptionsOpen}
        onClose={() => {
          setOptionsOpen(false);
        }}
      />
    </Panel>
  );
});
