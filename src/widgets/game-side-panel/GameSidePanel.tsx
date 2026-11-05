import { observer } from 'mobx-react-lite';

import { useGameStore } from '@entities/game/game-context';
import { MoveList } from '@entities/game/ui/MoveList/MoveList';

import { GameAlert } from './GameAlert/GameAlert';
import { GameEndActions } from './GameEndActions/GameEndActions';
import { GameEvaluation } from './GameEvaluation/GameEvaluation';
import { GamePlayers } from './GamePlayers/GamePlayers';
import { GameStatus } from './GameStatus/GameStatus';
import styles from './GameSidePanel.module.scss';

export const GameSidePanel = observer(() => {
  const { view, browsedPly, browseTo } = useGameStore();

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
        onSelect={browseTo}
      />
      {view.isTraining ? <GameEvaluation /> : null}
      <GameStatus />
    </aside>
  );
});
