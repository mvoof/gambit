import { useTranslation } from 'react-i18next';

import type {
  Color,
  EvaluationLine,
} from '@shared/contracts/generated/bindings';
import { formatScore } from '@shared/lib/format-score';

import styles from './EngineLines.module.scss';

const TURN_KEYS: Record<Color, string> = {
  white: 'analysis.whiteToMove',
  black: 'analysis.blackToMove',
};

interface EngineLinesProps {
  lines: EvaluationLine[];
  /** Who plays the first move of every line. */
  turn: Color | null;
  /** Writes a move the way the player wants to read it. */
  formatMove?: (san: string) => string;
}

const asWritten = (san: string) => san;

export const EngineLines = ({
  lines,
  turn,
  formatMove = asWritten,
}: EngineLinesProps) => {
  const { t } = useTranslation();

  return (
    <section className={styles.panel} aria-label={t('analysis.lines')}>
      <header className={styles.header}>
        <span className={styles.turn}>{turn ? t(TURN_KEYS[turn]) : ' '}</span>
        <span>{t('analysis.scoreSide')}</span>
      </header>
      <ol className={styles.lines}>
        {lines.length === 0 ? (
          <li className={styles.waiting}>{t('analysis.thinking')}</li>
        ) : null}
        {lines.map((line) => (
          <li key={line.moves.join(' ')} className={styles.line}>
            <span className={styles.score}>{formatScore(line)}</span>
            <span className={styles.moves}>
              <strong className={styles.firstMove}>
                {formatMove(line.moves[0])}
              </strong>{' '}
              {line.moves.slice(1).map(formatMove).join(' ')}
            </span>
          </li>
        ))}
      </ol>
    </section>
  );
};
