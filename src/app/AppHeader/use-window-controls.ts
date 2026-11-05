import { useEffect, useState } from 'react';

import {
  closeWindow,
  isWindowMaximized,
  minimizeWindow,
  onWindowResized,
  toggleMaximizeWindow,
} from '@shared/api/window.service';

export const useWindowControls = () => {
  const [isMaximized, setMaximized] = useState(false);

  useEffect(() => {
    const refresh = () => {
      void isWindowMaximized().then(setMaximized);
    };
    const subscription = onWindowResized(refresh);

    refresh();

    return () => {
      void subscription.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  return {
    isMaximized,
    minimize: minimizeWindow,
    toggleMaximize: toggleMaximizeWindow,
    close: closeWindow,
  };
};
