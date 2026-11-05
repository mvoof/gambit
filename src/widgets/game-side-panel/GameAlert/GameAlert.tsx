import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useGameStore } from '@entities/game/game-context';
import { GameBanner } from '@entities/game/ui/GameBanner/GameBanner';

import { gameAlertOf } from './game-alert';

export const GameAlert = observer(() => {
  const { t } = useTranslation();
  const { view, browsedPly } = useGameStore();
  const alert = view ? gameAlertOf(view, browsedPly !== null) : null;

  if (!alert) {
    return null;
  }

  return (
    <GameBanner
      tone={alert.tone}
      icon={alert.icon}
      title={t(alert.titleKey)}
      text={alert.textKey ? t(alert.textKey) : null}
    />
  );
});
