import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import { EngineLines } from '@entities/evaluation/ui/EngineLines/EngineLines';
import { useGameStore } from '@entities/game/game-context';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { MoveList } from '@entities/game/ui/MoveList/MoveList';
import { PlayerStrip } from '@entities/game/ui/PlayerStrip/PlayerStrip';

import { GameControls } from './GameControls/GameControls';
import { GameStatus } from './GameStatus/GameStatus';
import styles from './GameSidePanel.module.scss';

export const GameSidePanel = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const evaluation = useEvaluationStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { view, browsedPly, browseTo } = game;

  if (!view || !settings) {
    return null;
  }

  const images = pieceSets.imagesFor(settings.pieceSet);
  const human = view.humanColor;
  const opponent = human === 'white' ? 'black' : 'white';
  const shared = { clock: view.clock, images };
  const signedDiff = human === 'white' ? view.materialDiff : -view.materialDiff;

  return (
    <aside className={styles.panel}>
      <PlayerStrip
        {...shared}
        name={view.opponentName}
        color={opponent}
        captured={view.captured[opponent]}
        advantage={-signedDiff}
      />
      <MoveList
        moves={view.moves}
        marks={view.moveMarks}
        activePly={browsedPly ?? view.moves.length}
        onSelect={browseTo}
      />
      {view.isTraining ? (
        <EngineLines
          lines={evaluation.linesFor(game.displayedFen ?? '')}
          turn={evaluation.turnFor(game.displayedFen ?? '')}
        />
      ) : null}
      <GameStatus />
      <GameControls />
      <PlayerStrip
        {...shared}
        name={t('game.you')}
        color={human}
        captured={view.captured[human]}
        advantage={signedDiff}
      />
    </aside>
  );
});
