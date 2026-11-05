import { observer } from 'mobx-react-lite';

import { useGameStore } from '@entities/game/game-context';
import { MoveList } from '@entities/game/ui/MoveList/MoveList';
import { useMoveFormat } from '@entities/settings/use-move-format';

import { GameAlert } from './GameAlert/GameAlert';
import { GameEndActions } from './GameEndActions/GameEndActions';
import { GameEvaluation } from './GameEvaluation/GameEvaluation';
import { GamePlayers } from './GamePlayers/GamePlayers';
import { GameStatus } from './GameStatus/GameStatus';
import styles from './GameSidePanel.module.scss';

export const GameSidePanel = observer(() => {
  const { view, browsedPly, browseTo } = useGameStore();
  const formatMove = useMoveFormat();

  if (!view) {
    return null;
  }

  return (
    <aside className={styles.panel}>
      <GamePlayers />
      <GameAlert />
      <GameEndActions />
      <MoveList
        moves={view.moves}
        marks={view.moveMarks}
        activePly={browsedPly ?? view.moves.length}
        formatMove={formatMove}
        onSelect={browseTo}
      />
      {view.isTraining ? <GameEvaluation /> : null}
      <GameStatus />
    </aside>
  );
});
