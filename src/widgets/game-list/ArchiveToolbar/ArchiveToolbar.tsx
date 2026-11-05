import { Search } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import type { ArchiveFilter } from '@entities/archive/select-games';
import { Select } from '@shared/ui/Select/Select';

import styles from './ArchiveToolbar.module.scss';

export const ArchiveToolbar = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const { filter } = archive;

  return (
    <div className={styles.toolbar}>
      <label className={styles.search}>
        <Search className={styles.icon} aria-hidden />
        <input
          className={styles.input}
          type="search"
          spellCheck={false}
          placeholder={t('archive.search')}
          value={filter.query}
          onChange={(event) => {
            archive.setFilter({ query: event.target.value });
          }}
        />
      </label>
      <div className={styles.select}>
        <Select<ArchiveFilter['color']>
          label={t('archive.colorFilter')}
          value={filter.color}
          onChange={(color) => {
            archive.setFilter({ color });
          }}
          options={[
            { value: 'any', label: t('archive.colorAny') },
            { value: 'white', label: t('archive.colorWhite') },
            { value: 'black', label: t('archive.colorBlack') },
          ]}
        />
      </div>
      <div className={styles.select}>
        <Select<ArchiveFilter['sort']>
          label={t('archive.sortFilter')}
          value={filter.sort}
          onChange={(sort) => {
            archive.setFilter({ sort });
          }}
          options={[
            { value: 'newest', label: t('archive.sortNewest') },
            { value: 'oldest', label: t('archive.sortOldest') },
            { value: 'longest', label: t('archive.sortLongest') },
          ]}
        />
      </div>
    </div>
  );
});
