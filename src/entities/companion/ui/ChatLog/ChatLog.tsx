import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import type { ChatTurn } from '@shared/contracts/generated/bindings';

import styles from './ChatLog.module.scss';

interface ChatLogProps {
  turns: ChatTurn[];
  /** The reply so far; empty while the model has not said anything yet. */
  draft: string | null;
  isLoadingModel: boolean;
}

export const ChatLog = ({ turns, draft, isLoadingModel }: ChatLogProps) => {
  const { t } = useTranslation();
  const listRef = useRef<HTMLOListElement>(null);
  const waitingText = isLoadingModel
    ? t('companion.loadingModel')
    : t('companion.thinking');

  useEffect(() => {
    const list = listRef.current;

    if (list) {
      list.scrollTop = list.scrollHeight;
    }
  }, [turns, draft]);

  if (turns.length === 0 && draft === null) {
    return <p className={styles.empty}>{t('companion.empty')}</p>;
  }

  return (
    <ol ref={listRef} className={styles.log}>
      {turns.map((turn, index) => (
        <li
          key={index}
          className={
            turn.speaker === 'player'
              ? `${styles.turn} ${styles.player}`
              : styles.turn
          }
        >
          {turn.text}
        </li>
      ))}
      {draft === null ? null : (
        <li
          className={draft ? styles.turn : `${styles.turn} ${styles.waiting}`}
        >
          {draft || waitingText}
        </li>
      )}
    </ol>
  );
};
