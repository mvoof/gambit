import { reaction } from 'mobx';

import i18n from '@/i18n';
import { playSound } from '@shared/lib/sound-synth';
import type { GameSound } from '@shared/lib/sound-synth';

import type { RootStores } from '../root-stores';

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

/** One position at a time reaches the analysis engine. */
const syncAnalysis = (stores: RootStores) =>
  reaction(
    () => {
      const { navigation, archive, game, settings } = stores;

      if (!settings.settings?.evalBarEnabled) {
        return null;
      }

      if (navigation.page === 'archive') {
        return archive.frame?.fen ?? null;
      }

      return navigation.page === 'play' ? game.displayedFen : null;
    },
    (fen) => {
      void stores.evaluation.setTarget(fen);
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

/** Rust decides whether advice follows; this only says that the moment came. */
const syncAutoAdvice = (stores: RootStores) =>
  reaction(
    () => stores.game.view,
    (view, previous) => {
      const isNewMove =
        view !== null &&
        previous !== null &&
        view.moves.length > previous.moves.length;

      if (isNewMove && (view.outcome || view.turn === view.humanColor)) {
        void stores.companion.advise();
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
  syncAnalysis(stores);
  syncGameSounds(stores);
  syncAutoAdvice(stores);
  syncReplaySounds(stores);
};
