import { createContext, useContext } from 'react';

import type { GameStore } from './game.store';

export const GameStoreContext = createContext<GameStore | null>(null);

export const useGameStore = () => {
  const store = useContext(GameStoreContext);

  if (!store) {
    throw new Error('useGameStore needs a GameStoreContext provider');
  }

  return store;
};
