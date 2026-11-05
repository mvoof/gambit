import { createContext, useContext } from 'react';

import type { PiecesStore } from './pieces.store';

export const PiecesStoreContext = createContext<PiecesStore | null>(null);

export const usePiecesStore = () => {
  const store = useContext(PiecesStoreContext);

  if (!store) {
    throw new Error('usePiecesStore needs a PiecesStoreContext provider');
  }

  return store;
};
