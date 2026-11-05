import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { EvalGraph } from '@entities/evaluation/ui/EvalGraph/EvalGraph';
import { usePiecesStore } from '@entities/pieces/pieces-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import type { Color } from '@shared/contracts/generated/bindings';
import { pieceCode } from '@shared/lib/piece-code';

import styles from './ReplaySummary.module.scss';

const COLORS: Color[] = ['white', 'black'];

/** What was taken so far, and the course of the game. */
export const ReplaySummary = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const pieceSets = usePiecesStore();
  const { settings } = useSettingsStore();
  const { frame } = archive;

  if (!frame || !settings) {
    return null;
  }

  const images = pieceSets.imagesFor(settings.pieceSet);

  return (
    <div className={styles.summary}>
      <section className={styles.block}>
        <h2 className={styles.title}>{t('replay.captured')}</h2>
        {COLORS.map((color) => (
          <div key={color} className={styles.captured}>
            {frame.captured[color].map((kind, index) => (
              <img
                key={`${kind}${index}`}
                className={styles.piece}
                src={
                  images[pieceCode(color === 'white' ? 'black' : 'white', kind)]
                }
                alt=""
              />
            ))}
          </div>
        ))}
      </section>
      <section className={styles.block}>
        <h2 className={styles.title}>{t('replay.graph')}</h2>
        <EvalGraph
          points={archive.points}
          mistakePlies={archive.playerMistakePlies}
          activePly={archive.frameIndex}
          lastPly={archive.lastFrameIndex}
          onSelect={archive.goTo}
        />
      </section>
    </div>
  );
});
