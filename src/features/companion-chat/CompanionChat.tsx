import { observer } from 'mobx-react-lite';
import { useState } from 'react';
import type { FormEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { useCompanionStore } from '@entities/companion/companion-context';
import { ChatLog } from '@entities/companion/ui/ChatLog/ChatLog';
import { useGameStore } from '@entities/game/game-context';
import { useSettingsStore } from '@entities/settings/settings-context';
import { Button } from '@shared/ui/Button/Button';
import { Toggle } from '@shared/ui/Toggle/Toggle';

import styles from './CompanionChat.module.scss';

export const CompanionChat = observer(() => {
  const { t } = useTranslation();
  const companion = useCompanionStore();
  const game = useGameStore();
  const settingsStore = useSettingsStore();
  const [question, setQuestion] = useState('');
  const { settings } = settingsStore;

  if (!settings) {
    return null;
  }

  const submit = (event: FormEvent) => {
    event.preventDefault();

    const text = question.trim();

    if (text && !companion.isReplying) {
      setQuestion('');
      void companion.ask(text, game.browsedPly);
    }
  };

  return (
    <section className={styles.chat}>
      <Toggle
        label={t('companion.autoAdvice')}
        isChecked={settings.llm.autoAdvice}
        onChange={(autoAdvice) => {
          void settingsStore.updateLlm({ autoAdvice });
        }}
      />
      <ChatLog
        turns={companion.view.turns}
        draft={companion.view.draft}
        isLoadingModel={companion.view.loadingModel}
      />
      {companion.error ? (
        <p className={styles.error}>{companion.error}</p>
      ) : null}
      <form className={styles.form} onSubmit={submit}>
        <input
          className={styles.input}
          value={question}
          placeholder={t('companion.placeholder')}
          aria-label={t('companion.placeholder')}
          onChange={(event) => {
            setQuestion(event.target.value);
          }}
        />
        {companion.isReplying ? (
          <Button onClick={companion.stop}>{t('companion.stop')}</Button>
        ) : (
          <Button type="submit" variant="primary" disabled={!question.trim()}>
            {t('companion.send')}
          </Button>
        )}
      </form>
    </section>
  );
});
