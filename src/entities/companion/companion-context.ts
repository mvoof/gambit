import { createContext, useContext } from 'react';

import type { CompanionStore } from './companion.store';

export const CompanionStoreContext = createContext<CompanionStore | null>(null);

export const useCompanionStore = () => {
  const store = useContext(CompanionStoreContext);

  if (!store) {
    throw new Error('useCompanionStore needs a CompanionStoreContext provider');
  }

  return store;
};
