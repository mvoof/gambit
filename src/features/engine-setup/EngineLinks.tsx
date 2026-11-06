import { useTranslation } from 'react-i18next';

import { useEngineLinks } from './use-engine-links';
import styles from './EngineLinks.module.scss';

const OTHER_ENGINES = [
  { name: 'Lc0', url: 'https://lczero.org/play/download/' },
  { name: 'Obsidian', url: 'https://github.com/gab8192/Obsidian/releases' },
  { name: 'CCRL', url: 'https://computerchess.org.uk/ccrl/4040/' },
];

export const EngineLinks = () => {
  const { t } = useTranslation();
  const { openLink } = useEngineLinks();

  return (
    <div className={styles.links}>
      <span>{t('engine.others')}</span>
      {OTHER_ENGINES.map((engine) => (
        <button
          key={engine.name}
          type="button"
          className={styles.link}
          onClick={() => {
            openLink(engine.url);
          }}
        >
          {engine.name}
        </button>
      ))}
    </div>
  );
};
