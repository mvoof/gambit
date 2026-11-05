import { Lightbulb } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useEvaluationStore } from '@entities/evaluation/evaluation-context';
import { EngineLines } from '@entities/evaluation/ui/EngineLines/EngineLines';
import { useGameStore } from '@entities/game/game-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { formatScore } from '@shared/lib/format-score';

import { verdictOf } from './position-verdict';
import styles from './GameEvaluation.module.scss';

/** What a training game shows: every word here is the engine's number. */
export const GameEvaluation = observer(() => {
  const { t } = useTranslation();
  const game = useGameStore();
  const evaluation = useEvaluationStore();
  const { settings } = useSettingsStore();
  const fen = game.displayedFen ?? '';
  const lines = evaluation.linesFor(fen);
  const best = lines.at(0)?.moves.at(0);
  const { latest, whiteWinPercent } = evaluation;

  return (
    <>
      {settings?.evalBarEnabled && latest ? (
        <div className={styles.meter}>
          <p className={styles.score}>
            {t('analysis.score', {
              score: formatScore(latest),
              verdict: t(`analysis.verdict.${verdictOf(latest)}`),
            })}
          </p>
          <div className={styles.track}>
            <div
              className={styles.fill}
              style={{ width: `${whiteWinPercent}%` }}
            />
          </div>
        </div>
      ) : null}
      {best ? (
        <p className={styles.best}>
          <Lightbulb className={styles.icon} aria-hidden />
          {t('analysis.bestMove')}: <strong>{best}</strong>
        </p>
      ) : null}
      <EngineLines lines={lines} turn={evaluation.turnFor(fen)} />
    </>
  );
});
