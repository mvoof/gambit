import { createContext, useContext } from 'react';

import type { NavigationStore } from './navigation.store';

export const NavigationStoreContext = createContext<NavigationStore | null>(
  null
);

export const useNavigationStore = () => {
  const store = useContext(NavigationStoreContext);

  if (!store) {
    throw new Error(
      'useNavigationStore needs a NavigationStoreContext provider'
    );
  }

  return store;
};
