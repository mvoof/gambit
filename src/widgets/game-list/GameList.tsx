import { Download, Trash2 } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { Button } from '@shared/ui/Button/Button';

import styles from './GameList.module.scss';

const MS_PER_SECOND = 1000;

export const GameList = observer(() => {
  const { t, i18n } = useTranslation();
  const archive = useArchiveStore();
  const dateFormat = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: 'medium',
    timeStyle: 'short',
  });

  return (
    <div className={styles.list}>
      <header className={styles.header}>
        <h1 className={styles.title}>{t('archive.title')}</h1>
        <Button onClick={archive.importFromFile}>{t('archive.import')}</Button>
      </header>
      {archive.error ? <p className={styles.error}>{archive.error}</p> : null}
      {archive.games.length === 0 ? (
        <p className={styles.empty}>{t('archive.empty')}</p>
      ) : null}
      {archive.games.map((game) => (
        <article key={game.id} className={styles.game}>
          <button
            type="button"
            className={styles.open}
            onClick={() => {
              void archive.open(game.id);
            }}
          >
            <span className={styles.players}>
              {game.white} — {game.black}
            </span>
            <span className={styles.details}>
              {game.result} · {game.timeControl} ·{' '}
              {t('archive.plies', { count: game.plyCount })} ·{' '}
              {dateFormat.format(game.playedAt * MS_PER_SECOND)}
            </span>
          </button>
          <Button
            variant="ghost"
            aria-label={t('archive.export')}
            title={t('archive.export')}
            onClick={() => {
              void archive.exportToFile(game.id);
            }}
          >
            <Download className={styles.icon} />
          </Button>
          <Button
            variant="ghost"
            aria-label={t('archive.delete')}
            title={t('archive.delete')}
            onClick={() => {
              void archive.remove(game.id);
            }}
          >
            <Trash2 className={styles.icon} />
          </Button>
        </article>
      ))}
    </div>
  );
});
