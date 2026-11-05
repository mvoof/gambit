import type { ReactNode } from 'react';

import { ArchiveStoreContext } from '@entities/archive/archive-context';
import { EngineStoreContext } from '@entities/engine/engine-context';
import { EvaluationStoreContext } from '@entities/evaluation/evaluation-context';
import { GameStoreContext } from '@entities/game/game-context';
import { NavigationStoreContext } from '@entities/navigation/navigation-context';
import { PiecesStoreContext } from '@entities/pieces/pieces-context';
import { SettingsStoreContext } from '@entities/settings/settings-context';

import type { RootStores } from './root-stores';

interface StoreProvidersProps {
  stores: RootStores;
  children: ReactNode;
}

export const StoreProviders = ({ stores, children }: StoreProvidersProps) => (
  <SettingsStoreContext.Provider value={stores.settings}>
    <NavigationStoreContext.Provider value={stores.navigation}>
      <EngineStoreContext.Provider value={stores.engine}>
        <PiecesStoreContext.Provider value={stores.pieces}>
          <GameStoreContext.Provider value={stores.game}>
            <EvaluationStoreContext.Provider value={stores.evaluation}>
              <ArchiveStoreContext.Provider value={stores.archive}>
                {children}
              </ArchiveStoreContext.Provider>
            </EvaluationStoreContext.Provider>
          </GameStoreContext.Provider>
        </PiecesStoreContext.Provider>
      </EngineStoreContext.Provider>
    </NavigationStoreContext.Provider>
  </SettingsStoreContext.Provider>
);
