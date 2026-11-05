import { createContext, useContext } from 'react';

import type { UpdateStore } from './update.store';

export const UpdateStoreContext = createContext<UpdateStore | null>(null);

export const useUpdateStore = () => {
  const store = useContext(UpdateStoreContext);

  if (!store) {
    throw new Error('useUpdateStore needs an UpdateStoreContext provider');
  }

  return store;
};
