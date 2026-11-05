import { CircleAlert, Crown, Handshake } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

import type { BannerTone } from '@entities/game/ui/GameBanner/GameBanner';
import type {
  Color,
  GameOutcome,
  GameView,
} from '@shared/contracts/generated/bindings';

const RESULT_KEYS: Record<GameOutcome['result'], string> = {
  whiteWins: 'result.whiteWins',
  blackWins: 'result.blackWins',
  draw: 'result.draw',
};

export interface GameAlertContent {
  tone: BannerTone;
  icon: LucideIcon;
  titleKey: string;
  textKey: string | null;
}

type AlertedView = Pick<
  GameView,
  'outcome' | 'humanColor' | 'turn' | 'checkSquare'
>;

const outcomeAlert = (
  outcome: GameOutcome,
  humanColor: Color
): GameAlertContent => {
  const resultKey = RESULT_KEYS[outcome.result];
  const reasonKey = `reason.${outcome.reason}`;

  if (outcome.result === 'draw') {
    return {
      tone: 'neutral',
      icon: Handshake,
      titleKey: resultKey,
      textKey: reasonKey,
    };
  }

  const humanWon =
    (outcome.result === 'whiteWins') === (humanColor === 'white');
  const tone = humanWon ? 'success' : 'danger';

  if (outcome.reason === 'checkmate') {
    return { tone, icon: Crown, titleKey: 'alert.mate', textKey: resultKey };
  }

  return { tone, icon: Crown, titleKey: resultKey, textKey: reasonKey };
};

/** A check belongs to the live position, so browsing back hides it. */
export const gameAlertOf = (
  view: AlertedView,
  isBrowsing: boolean
): GameAlertContent | null => {
  if (view.outcome) {
    return outcomeAlert(view.outcome, view.humanColor);
  }

  if (isBrowsing || !view.checkSquare) {
    return null;
  }

  if (view.turn === view.humanColor) {
    return {
      tone: 'danger',
      icon: CircleAlert,
      titleKey: 'alert.check',
      textKey: null,
    };
  }

  return {
    tone: 'neutral',
    icon: CircleAlert,
    titleKey: 'alert.check',
    textKey: null,
  };
};
