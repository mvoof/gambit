import { figurineSan } from '@shared/lib/figurine-san';

import { useSettingsStore } from './settings-context';

const asLetters = (san: string) => san;

/** Read inside an `observer`, so a change of the setting redraws the moves. */
export const useMoveFormat = () => {
  const { settings } = useSettingsStore();

  return settings?.moveNotation === 'letters' ? asLetters : figurineSan;
};
