import { observer } from 'mobx-react-lite';

import { useArchiveStore } from '@entities/archive/archive-context';
import { Board } from '@entities/board/ui/Board/Board';
import { ResizableBoardFrame } from '@entities/board/ui/ResizableBoardFrame/ResizableBoardFrame';
import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import { EvalBar } from '@entities/evaluation/ui/EvalBar/EvalBar';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';

import { ReplayPanel } from './ReplayPanel/ReplayPanel';
import { ReplayPlayers } from './ReplayPlayers/ReplayPlayers';
import { ReplaySummary } from './ReplaySummary/ReplaySummary';
import styles from './ReplayViewer.module.scss';

export const ReplayViewer = observer(() => {
  const archive = useArchiveStore();
  const evaluation = useEvaluationStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { replay, frame, replayOrientation } = archive;

  if (!replay || !frame || !settings) {
    return null;
  }

  return (
    <main className={styles.page}>
      <div className={styles.stage}>
        <ReplayPlayers />
        <div className={styles.boardRow}>
          {settings.evalBarEnabled ? (
            <EvalBar orientation={replayOrientation} />
          ) : null}
          <ResizableBoardFrame>
            <Board
              pieces={frame.pieces}
              images={pieceSets.imagesFor(settings.pieceSet)}
              orientation={replayOrientation}
              settings={settings.board}
              lastMove={frame.lastMove}
              checkSquare={frame.checkSquare}
              hint={evaluation.bestMoveFor(frame.fen)}
            />
          </ResizableBoardFrame>
        </div>
        <ReplaySummary />
      </div>
      <ReplayPanel />
    </main>
  );
});
