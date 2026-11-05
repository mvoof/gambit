import { observer } from 'mobx-react-lite';
import { useEffect, useState } from 'react';

import { useNavigationStore } from '@entities/navigation/navigation-context';
import type { AppPage } from '@entities/navigation/navigation.store';
import { LeaveGameDialog } from '@features/leave-game/LeaveGameDialog';
import { ArchivePage } from '@pages/archive/ArchivePage';
import { HomePage } from '@pages/home/HomePage';
import { PlayPage } from '@pages/play/PlayPage';
import { SettingsPage } from '@pages/settings/SettingsPage';

import { AppHeader } from './AppHeader/AppHeader';
import { StoreProviders } from './StoreProviders';
import { createRootStores } from './root-stores';
import { startApp } from './start-app';

const PAGES: Record<AppPage, () => React.ReactNode> = {
  home: HomePage,
  play: PlayPage,
  archive: ArchivePage,
  settings: SettingsPage,
};

const PageOutlet = observer(() => {
  const { page } = useNavigationStore();
  const Page = PAGES[page];

  return <Page />;
});

export const App = () => {
  const [stores] = useState(createRootStores);
  const [isReady, setReady] = useState(false);
  const [startupError, setStartupError] = useState<string | null>(null);

  useEffect(() => {
    startApp(stores)
      .then(() => {
        setReady(true);
      })
      .catch((error: unknown) => {
        setStartupError(String(error));
      });
  }, [stores]);

  if (startupError) {
    return <p>{startupError}</p>;
  }

  if (!isReady) {
    return null;
  }

  return (
    <StoreProviders stores={stores}>
      <AppHeader />
      <PageOutlet />
      <LeaveGameDialog />
    </StoreProviders>
  );
};
