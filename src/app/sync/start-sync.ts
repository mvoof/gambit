import { reaction } from 'mobx';

import i18n from '@/i18n';
import { playSound } from '@shared/lib/sound-synth';
import type { GameSound } from '@shared/lib/sound-synth';
import type { Theme } from '@shared/contracts/generated/bindings';

import type { RootStores } from '../root-stores';

const DARK_SCHEME_QUERY = '(prefers-color-scheme: dark)';

const volumeOf = ({ settings }: RootStores) =>
  settings.settings?.soundVolume ?? 0;

const syncLanguage = (stores: RootStores) =>
  reaction(
    () => stores.settings.settings?.language,
    (language) => {
      if (language) {
        void i18n.changeLanguage(language);
      }
    },
    { fireImmediately: true }
  );

const applyTheme = (theme: Theme) => {
  const systemTheme = window.matchMedia(DARK_SCHEME_QUERY).matches
    ? 'dark'
    : 'light';

  document.documentElement.dataset.theme =
    theme === 'system' ? systemTheme : theme;
};

const syncTheme = (stores: RootStores) => {
  const themeOf = () => stores.settings.settings?.theme ?? 'dark';

  window.matchMedia(DARK_SCHEME_QUERY).addEventListener('change', () => {
    applyTheme(themeOf());
  });

  return reaction(themeOf, applyTheme, { fireImmediately: true });
};

/** Leaving a game with moves on the board stops it, so it asks first. */
const syncLeaveGuard = (stores: RootStores) =>
  reaction(
    () => stores.game.canBeLost,
    (canBeLost) => {
      stores.navigation.setLeaveGuarded(canBeLost);
    },
    { fireImmediately: true }
  );

/** One position at a time reaches the analysis engine: a replay, or a training
 * game. An ordinary game gets no help, so nothing is analysed during it. */
const syncAnalysis = (stores: RootStores) =>
  reaction(
    () => {
      const { navigation, archive, game } = stores;

      if (navigation.page === 'archive') {
        return archive.frame?.fen ?? null;
      }

      const isTraining = navigation.page === 'play' && game.view?.isTraining;

      return isTraining ? game.displayedFen : null;
    },
    (fen) => {
      void stores.evaluation.setTarget(fen);
    },
    { fireImmediately: true }
  );

/** A saved game is measured only while its replay is on screen. */
const syncReview = (stores: RootStores) =>
  reaction(
    () =>
      stores.navigation.page === 'archive' ? stores.archive.replayId : null,
    (id) => {
      void stores.archive.setReviewTarget(id);
    },
    { fireImmediately: true }
  );

const syncGameSounds = (stores: RootStores) =>
  reaction(
    () => stores.game.view,
    (view, previous) => {
      const isNewMove =
        view !== null &&
        previous !== null &&
        view.moves.length > previous.moves.length;
      const sound: GameSound | undefined = view?.moves.at(-1)?.sound;

      if (isNewMove && sound) {
        playSound(sound, volumeOf(stores));
      }

      if (view?.outcome && !previous?.outcome) {
        playSound('gameEnd', volumeOf(stores));
      }
    }
  );

const syncReplaySounds = (stores: RootStores) =>
  reaction(
    () => stores.archive.frameIndex,
    (index, previous) => {
      const sound = stores.archive.frame?.sound;

      if (index === previous + 1 && sound) {
        playSound(sound, volumeOf(stores));
      }
    }
  );

/** Wires the stores to each other: entities never import their siblings. */
export const startSync = (stores: RootStores) => {
  syncLanguage(stores);
  syncTheme(stores);
  syncLeaveGuard(stores);
  syncAnalysis(stores);
  syncReview(stores);
  syncGameSounds(stores);
  syncReplaySounds(stores);
};
