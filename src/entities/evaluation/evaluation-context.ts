import { createContext, useContext } from 'react';

import type { EvaluationStore } from './evaluation.store';

export const EvaluationStoreContext = createContext<EvaluationStore | null>(
  null
);

export const useEvaluationStore = () => {
  const store = useContext(EvaluationStoreContext);

  if (!store) {
    throw new Error(
      'useEvaluationStore needs an EvaluationStoreContext provider'
    );
  }

  return store;
};
