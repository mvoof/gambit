import { openExternal } from '@shared/api/system.service';

export const useEngineLinks = () => ({
  openLink: (url: string) => {
    void openExternal(url);
  },
});
