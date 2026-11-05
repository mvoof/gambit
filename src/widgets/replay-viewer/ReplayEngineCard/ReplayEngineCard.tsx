import { Cpu, Lightbulb } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';
import { useEngineStore } from '@entities/engine/engine-context';
import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import { EngineLines } from '@entities/evaluation/ui/EngineLines/EngineLines';
import { formatScore } from '@shared/lib/format-score';

import { ReplayNote } from '../ReplayNote/ReplayNote';
import styles from './ReplayEngineCard.module.scss';

export const ReplayEngineCard = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const evaluation = useEvaluationStore();
  const { engine } = useEngineStore();
  const { frame, frameMark } = archive;

  if (!frame) {
    return null;
  }

  const { latest } = evaluation;
  const measured = latest?.fen === frame.fen ? latest : null;

  return (
    <section className={styles.card}>
      <header className={styles.header}>
        <Cpu className={styles.icon} aria-hidden />
        <div className={styles.engine}>
          <span className={styles.name}>{engine?.name}</span>
          <span className={styles.depth}>
            {measured ? t('replay.depth', { depth: measured.depth }) : ' '}
          </span>
        </div>
        <span className={styles.score}>
          {measured ? formatScore(measured) : ' '}
        </span>
      </header>
      {frameMark ? (
        <p className={styles.better}>
          <Lightbulb className={styles.hint} aria-hidden />
          {t(`game.mark.${frameMark.quality}`, { move: frameMark.bestSan })}
        </p>
      ) : null}
      <EngineLines
        lines={evaluation.linesFor(frame.fen)}
        turn={evaluation.turnFor(frame.fen)}
      />
      <ReplayNote />
    </section>
  );
});
