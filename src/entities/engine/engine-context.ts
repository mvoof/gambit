import { createContext, useContext } from 'react';

import type { EngineStore } from './engine.store';

export const EngineStoreContext = createContext<EngineStore | null>(null);

export const useEngineStore = () => {
  const store = useContext(EngineStoreContext);

  if (!store) {
    throw new Error('useEngineStore needs an EngineStoreContext provider');
  }

  return store;
};
