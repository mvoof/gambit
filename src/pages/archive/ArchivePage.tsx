import { observer } from 'mobx-react-lite';
import { useEffect } from 'react';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useArrowKeys } from '@shared/lib/use-arrow-keys';
import { GameList } from '@widgets/game-list/GameList';
import { ReplayViewer } from '@widgets/replay-viewer/ReplayViewer';

const STEP_FORWARD = 1;
const STEP_BACK = -1;
const FIRST_FRAME = 0;

export const ArchivePage = observer(() => {
  const archive = useArchiveStore();
  const { refresh, step, goTo, lastFrameIndex } = archive;

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useArrowKeys({
    onPrevious: () => {
      step(STEP_BACK);
    },
    onNext: () => {
      step(STEP_FORWARD);
    },
    onFirst: () => {
      goTo(FIRST_FRAME);
    },
    onLast: () => {
      goTo(lastFrameIndex);
    },
  });

  return archive.replay ? <ReplayViewer /> : <GameList />;
});
