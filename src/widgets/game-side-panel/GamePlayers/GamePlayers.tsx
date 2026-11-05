import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { PlayerStrip } from '@entities/game/ui/PlayerStrip/PlayerStrip';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import type { Color } from '@shared/contracts/generated/bindings';
import { Panel } from '@shared/ui/Panel/Panel';

import styles from './GamePlayers.module.scss';

/** Each player stands on their own side of the board. */
export const GamePlayers = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { view, orientation } = game;

  if (!view || !settings) {
    return null;
  }

  const images = pieceSets.imagesFor(settings.pieceSet);
  const signedDiff =
    view.humanColor === 'white' ? view.materialDiff : -view.materialDiff;

  const strip = (color: Color) => {
    const isHuman = color === view.humanColor;

    return (
      <PlayerStrip
        clock={view.clock}
        images={images}
        name={isHuman ? t('game.you') : view.opponentName}
        elo={isHuman ? null : view.opponentElo}
        color={color}
        captured={view.captured[color]}
        advantage={isHuman ? signedDiff : -signedDiff}
      />
    );
  };

  return (
    <Panel className={styles.players}>
      {strip(orientation === 'white' ? 'black' : 'white')}
      {strip(orientation)}
    </Panel>
  );
});
