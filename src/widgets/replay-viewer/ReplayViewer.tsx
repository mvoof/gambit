import {
  ChevronFirst,
  ChevronLast,
  ChevronLeft,
  ChevronRight,
  Pause,
  Play,
  X,
} from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { Board } from '@entities/board/ui/Board/Board';
import { ResizableBoardFrame } from '@entities/board/ui/ResizableBoardFrame/ResizableBoardFrame';
import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import { EngineLines } from '@entities/evaluation/ui/EngineLines/EngineLines';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { Button } from '@shared/ui/Button/Button';
import { MoveList } from '@entities/game/ui/MoveList/MoveList';

import { ReplayNote } from './ReplayNote/ReplayNote';

import styles from './ReplayViewer.module.scss';

const FIRST_FRAME = 0;
const STEP_FORWARD = 1;
const STEP_BACK = -1;

export const ReplayViewer = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const evaluation = useEvaluationStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { replay, frame, frameIndex, lastFrameIndex } = archive;

  if (!replay || !frame || !settings) {
    return null;
  }

  return (
    <>
      <ResizableBoardFrame>
        <Board
          pieces={frame.pieces}
          images={pieceSets.imagesFor(settings.pieceSet)}
          orientation="white"
          settings={settings.board}
          lastMove={frame.lastMove}
          checkSquare={frame.checkSquare}
          hint={evaluation.bestMoveFor(frame.fen)}
        />
      </ResizableBoardFrame>
      <aside className={styles.panel}>
        <header className={styles.header}>
          <div>
            <p className={styles.players}>
              {replay.white} — {replay.black}
            </p>
            <p className={styles.result}>{replay.result}</p>
          </div>
          <Button
            variant="ghost"
            aria-label={t('common.close')}
            onClick={archive.close}
          >
            <X className={styles.icon} />
          </Button>
        </header>
        <MoveList
          moves={replay.moves}
          marks={archive.marks}
          activePly={frameIndex}
          onSelect={archive.goTo}
        />
        <ReplayNote />
        <EngineLines
          lines={evaluation.linesFor(frame.fen)}
          turn={evaluation.turnFor(frame.fen)}
        />
        <div className={styles.controls}>
          <Button
            aria-label={t('replay.first')}
            onClick={() => {
              archive.goTo(FIRST_FRAME);
            }}
          >
            <ChevronFirst className={styles.icon} />
          </Button>
          <Button
            aria-label={t('replay.previous')}
            onClick={() => {
              archive.step(STEP_BACK);
            }}
          >
            <ChevronLeft className={styles.icon} />
          </Button>
          <Button
            variant="primary"
            aria-label={t('replay.autoplay')}
            onClick={archive.toggleAutoplay}
          >
            {archive.isAutoplaying ? (
              <Pause className={styles.icon} />
            ) : (
              <Play className={styles.icon} />
            )}
          </Button>
          <Button
            aria-label={t('replay.next')}
            onClick={() => {
              archive.step(STEP_FORWARD);
            }}
          >
            <ChevronRight className={styles.icon} />
          </Button>
          <Button
            aria-label={t('replay.last')}
            onClick={() => {
              archive.goTo(lastFrameIndex);
            }}
          >
            <ChevronLast className={styles.icon} />
          </Button>
        </div>
      </aside>
    </>
  );
});
