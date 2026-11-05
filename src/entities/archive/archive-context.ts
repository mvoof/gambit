import { createContext, useContext } from 'react';

import type { ArchiveStore } from './archive.store';

export const ArchiveStoreContext = createContext<ArchiveStore | null>(null);

export const useArchiveStore = () => {
  const store = useContext(ArchiveStoreContext);

  if (!store) {
    throw new Error('useArchiveStore needs an ArchiveStoreContext provider');
  }

  return store;
};
