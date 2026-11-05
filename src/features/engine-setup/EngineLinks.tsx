import { useTranslation } from 'react-i18next';

import { useEngineLinks } from './use-engine-links';
import styles from './EngineLinks.module.scss';

const OTHER_ENGINES = [
  { name: 'Lc0', url: 'https://lczero.org/play/download/' },
  { name: 'Berserk', url: 'https://github.com/jhonnold/berserk/releases' },
  { name: 'Ethereal', url: 'https://github.com/AndyGrant/Ethereal/releases' },
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
