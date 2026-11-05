import type { RootStores } from './root-stores';
import { startSync } from './sync/start-sync';

/** The settings come first: everything else reads them. */
export const startApp = async (stores: RootStores) => {
  await stores.settings.start();
  startSync(stores);
  await Promise.all([
    stores.engine.start(),
    stores.pieces.start(),
    stores.game.start(),
    stores.evaluation.start(),
  ]);
};
