import { createContext, useContext } from 'react';

import type { SettingsStore } from './settings.store';

export const SettingsStoreContext = createContext<SettingsStore | null>(null);

export const useSettingsStore = () => {
  const store = useContext(SettingsStoreContext);

  if (!store) {
    throw new Error('useSettingsStore needs a SettingsStoreContext provider');
  }

  return store;
};
