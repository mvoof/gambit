import { useTranslation } from 'react-i18next';

import type { EvaluationLine } from '@shared/contracts/generated/bindings';
import { formatScore } from '@shared/lib/format-score';

import styles from './EngineLines.module.scss';

interface EngineLinesProps {
  lines: EvaluationLine[];
}

export const EngineLines = ({ lines }: EngineLinesProps) => {
  const { t } = useTranslation();

  return (
    <ol className={styles.lines} aria-label={t('analysis.lines')}>
      {lines.length === 0 ? (
        <li className={styles.waiting}>{t('analysis.thinking')}</li>
      ) : null}
      {lines.map((line) => (
        <li key={line.moves[0]} className={styles.line}>
          <span className={styles.score}>{formatScore(line)}</span>
          <span className={styles.moves}>{line.moves.join(' ')}</span>
        </li>
      ))}
    </ol>
  );
};
