import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';

import { Button } from '@shared/ui/Button/Button';

import styles from './Chat.module.scss';

export interface ChatMessage {
  id: string;
  isOwn: boolean;
  text: string;
}

interface ChatProps {
  messages: ChatMessage[];
  emptyText: string;
  placeholder: string;
  sendLabel: string;
  onSend: (text: string) => void;
}

/** Nothing uses it yet: it waits for games against a friend. */
export const Chat = ({
  messages,
  emptyText,
  placeholder,
  sendLabel,
  onSend,
}: ChatProps) => {
  const [text, setText] = useState('');
  const listRef = useRef<HTMLOListElement>(null);

  useEffect(() => {
    const list = listRef.current;

    if (list) {
      list.scrollTop = list.scrollHeight;
    }
  }, [messages]);

  const submit = (event: FormEvent) => {
    event.preventDefault();

    const trimmed = text.trim();

    if (trimmed) {
      setText('');
      onSend(trimmed);
    }
  };

  return (
    <section className={styles.chat}>
      {messages.length === 0 ? (
        <p className={styles.empty}>{emptyText}</p>
      ) : (
        <ol ref={listRef} className={styles.log}>
          {messages.map((message) => (
            <li
              key={message.id}
              className={
                message.isOwn
                  ? `${styles.message} ${styles.own}`
                  : styles.message
              }
            >
              {message.text}
            </li>
          ))}
        </ol>
      )}
      <form className={styles.form} onSubmit={submit}>
        <input
          className={styles.input}
          value={text}
          placeholder={placeholder}
          aria-label={placeholder}
          onChange={(event) => {
            setText(event.target.value);
          }}
        />
        <Button type="submit" variant="primary" disabled={!text.trim()}>
          {sendLabel}
        </Button>
      </form>
    </section>
  );
};
