import { X } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import type { ReplayTab } from '@entities/archive/archive.store';
import { MoveList } from '@entities/game/ui/MoveList/MoveList';
import type { Color } from '@shared/contracts/generated/bindings';
import { Button } from '@shared/ui/Button/Button';
import { Segmented } from '@shared/ui/Segmented/Segmented';

import { ReplayControls } from '../ReplayControls/ReplayControls';
import { ReplayEngineCard } from '../ReplayEngineCard/ReplayEngineCard';
import styles from './ReplayPanel.module.scss';

export const ReplayPanel = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const { replay, replayTab, frameIndex } = archive;

  if (!replay) {
    return null;
  }

  const nameOf = (color: Color) => {
    if (color === replay.humanColor) {
      return t('game.you');
    }

    return color === 'white' ? replay.white : replay.black;
  };

  return (
    <aside className={styles.panel}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>{t('replay.title')}</h1>
          <p className={styles.result}>
            {t('replay.players', {
              white: nameOf('white'),
              black: nameOf('black'),
              result: replay.result,
            })}
          </p>
        </div>
        <Button
          variant="ghost"
          aria-label={t('common.close')}
          onClick={archive.close}
        >
          <X aria-hidden />
        </Button>
      </header>
      <Segmented<ReplayTab>
        value={replayTab}
        onChange={archive.setReplayTab}
        options={[
          { value: 'all', label: t('replay.tabAll') },
          {
            value: 'mistakes',
            label: t('replay.tabMistakes', {
              count: archive.playerMistakePlies.length,
            }),
          },
          {
            value: 'best',
            label: t('replay.tabBest', {
              count: archive.playerBestPlies.length,
            }),
          },
        ]}
      />
      <MoveList
        moves={replay.moves}
        marks={archive.marks}
        activePly={frameIndex}
        highlightedPlies={archive.highlightedPlies}
        bestPlies={replayTab === 'best' ? archive.playerBestPlies : undefined}
        onSelect={archive.goTo}
      />
      <ReplayEngineCard />
      <ReplayControls />
    </aside>
  );
});
