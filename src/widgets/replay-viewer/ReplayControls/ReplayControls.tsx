import {
  AlertTriangle,
  ChevronFirst,
  ChevronLast,
  ChevronLeft,
  ChevronRight,
  FlipVertical2,
  Pause,
  Play,
} from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { Button } from '@shared/ui/Button/Button';
import { Select } from '@shared/ui/Select/Select';

import styles from './ReplayControls.module.scss';

const FIRST_FRAME = 0;
const STEP_FORWARD = 1;
const STEP_BACK = -1;
const SPEEDS = ['0.5', '1', '2'];

export const ReplayControls = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();

  return (
    <div className={styles.controls}>
      <div className={styles.row}>
        <Button
          aria-label={t('replay.first')}
          onClick={() => {
            archive.goTo(FIRST_FRAME);
          }}
        >
          <ChevronFirst aria-hidden />
        </Button>
        <Button
          aria-label={t('replay.previous')}
          onClick={() => {
            archive.step(STEP_BACK);
          }}
        >
          <ChevronLeft aria-hidden />
        </Button>
        <Button
          variant="primary"
          aria-label={t('replay.autoplay')}
          onClick={archive.toggleAutoplay}
        >
          {archive.isAutoplaying ? <Pause aria-hidden /> : <Play aria-hidden />}
        </Button>
        <Button
          aria-label={t('replay.next')}
          onClick={() => {
            archive.step(STEP_FORWARD);
          }}
        >
          <ChevronRight aria-hidden />
        </Button>
        <Button
          aria-label={t('replay.last')}
          onClick={() => {
            archive.goTo(archive.lastFrameIndex);
          }}
        >
          <ChevronLast aria-hidden />
        </Button>
        <div className={styles.speed}>
          <Select
            label={t('replay.speed')}
            value={String(archive.autoplaySpeed)}
            onChange={(speed) => {
              archive.setAutoplaySpeed(Number(speed));
            }}
            options={SPEEDS.map((speed) => ({
              value: speed,
              label: `${speed}×`,
            }))}
          />
        </div>
      </div>
      <div className={styles.row}>
        <Button
          variant="danger"
          className={styles.nextMistake}
          disabled={archive.playerMistakePlies.length === 0}
          onClick={archive.goToNextMistake}
        >
          <AlertTriangle aria-hidden />
          {t('replay.nextMistake')}
        </Button>
        <Button
          aria-label={t('game.flip')}
          title={t('game.flip')}
          onClick={archive.flipReplay}
        >
          <FlipVertical2 aria-hidden />
        </Button>
      </div>
    </div>
  );
});
