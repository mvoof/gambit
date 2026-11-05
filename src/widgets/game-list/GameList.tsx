import { FolderOpen, Plus, Upload } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { GAMES_PER_PAGE } from '@entities/archive/archive.store';
import { selectGames } from '@entities/archive/select-games';
import { NewGameDialog } from '@features/new-game/NewGameDialog';
import { formatPlayedAt } from '@shared/lib/format-date';
import { Button } from '@shared/ui/Button/Button';
import { PageHeader } from '@shared/ui/PageHeader/PageHeader';

import { ArchiveSidebar } from './ArchiveSidebar/ArchiveSidebar';
import { ArchiveToolbar } from './ArchiveToolbar/ArchiveToolbar';
import { GameRow } from './GameRow/GameRow';
import { GamesPagination } from './GamesPagination/GamesPagination';
import styles from './GameList.module.scss';

export const GameList = observer(() => {
  const { t, i18n } = useTranslation();
  const archive = useArchiveStore();
  const [isNewGameOpen, setNewGameOpen] = useState(false);
  const selected = selectGames(archive.games, archive.filter, (game) =>
    formatPlayedAt(game.playedAt, i18n.language)
  );
  const pageCount = Math.max(1, Math.ceil(selected.length / GAMES_PER_PAGE));
  const pageIndex = Math.min(archive.pageIndex, pageCount - 1);
  const pageStart = pageIndex * GAMES_PER_PAGE;
  const games = selected.slice(pageStart, pageStart + GAMES_PER_PAGE);

  return (
    <main className={styles.page}>
      <PageHeader
        icon={FolderOpen}
        title={t('archive.title')}
        subtitle={t('archive.subtitle')}
      >
        <Button onClick={archive.importFromFile}>
          <Upload aria-hidden />
          {t('archive.import')}
        </Button>
        <Button
          variant="primary"
          onClick={() => {
            setNewGameOpen(true);
          }}
        >
          <Plus aria-hidden />
          {t('game.newGame')}
        </Button>
      </PageHeader>
      <div className={styles.body}>
        <ArchiveSidebar />
        <section className={styles.content}>
          <ArchiveToolbar />
          {archive.error ? (
            <p className={styles.error}>{archive.error}</p>
          ) : null}
          <div className={styles.table}>
            <div className={styles.heading} aria-hidden>
              <span>{t('archive.column.opponent')}</span>
              <span>{t('archive.column.result')}</span>
              <span>{t('archive.column.moves')}</span>
              <span>{t('archive.column.date')}</span>
              <span>{t('archive.column.actions')}</span>
            </div>
            {games.length === 0 ? (
              <p className={styles.empty}>
                {archive.games.length === 0
                  ? t('archive.empty')
                  : t('archive.nothingFound')}
              </p>
            ) : null}
            <ul className={styles.rows}>
              {games.map((game) => (
                <GameRow key={game.id} game={game} />
              ))}
            </ul>
            <GamesPagination pageIndex={pageIndex} pageCount={pageCount} />
          </div>
        </section>
      </div>
      <NewGameDialog
        isOpen={isNewGameOpen}
        onClose={() => {
          setNewGameOpen(false);
        }}
      />
    </main>
  );
});
