import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { MoveList } from '@entities/game/ui/MoveList/MoveList';
import { PlayerStrip } from '@entities/game/ui/PlayerStrip/PlayerStrip';
import { CompanionChat } from '@features/companion-chat/CompanionChat';
import { Segmented } from '@shared/ui/Segmented/Segmented';

import { GameControls } from './GameControls/GameControls';
import { GameStatus } from './GameStatus/GameStatus';
import styles from './GameSidePanel.module.scss';

type SideTab = 'moves' | 'companion';

export const GameSidePanel = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const [tab, setTab] = useState<SideTab>('moves');
  const { view, browsedPly, browseTo } = game;

  if (!view || !settings) {
    return null;
  }

  const images = pieceSets.imagesFor(settings.pieceSet);
  const human = view.humanColor;
  const opponent = human === 'white' ? 'black' : 'white';
  const shared = { clock: view.clock, images };
  const signedDiff = human === 'white' ? view.materialDiff : -view.materialDiff;
  const hasCompanion = settings.llm.enabled;

  return (
    <aside className={styles.panel}>
      <PlayerStrip
        {...shared}
        name={view.opponentName}
        color={opponent}
        captured={view.captured[opponent]}
        advantage={-signedDiff}
      />
      {hasCompanion ? (
        <Segmented
          value={tab}
          onChange={setTab}
          options={[
            { value: 'moves', label: t('companion.tabMoves') },
            { value: 'companion', label: t('companion.tabCompanion') },
          ]}
        />
      ) : null}
      {hasCompanion && tab === 'companion' ? (
        <CompanionChat />
      ) : (
        <MoveList
          moves={view.moves}
          activePly={browsedPly ?? view.moves.length}
          onSelect={browseTo}
        />
      )}
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
