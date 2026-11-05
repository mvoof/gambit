import { RotateCcw, Wrench } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import heroImage from '@assets/brand/home-hero.jpg';
import { useArchiveStore } from '@entities/archive/archive-context';
import { useEngineStore } from '@entities/engine/engine-context';
import { useGameStore } from '@entities/game/game-context';
import { useNavigationStore } from '@entities/navigation/navigation-context';
import { Button } from '@shared/ui/Button/Button';

import styles from './HomeHero.module.scss';

/** A greeting: a game starts from the card next to it. */
export const HomeHero = observer(() => {
  const { t } = useTranslation();
  const archive = useArchiveStore();
  const game = useGameStore();
  const navigation = useNavigationStore();
  const { engine } = useEngineStore();
  const unfinished = archive.games.find(
    (entry) => entry.playerResult === 'unfinished'
  );

  const resumeLast = async (id: number) => {
    navigation.request('play');
    await game.resume(id);
  };

  return (
    <section className={styles.hero}>
      <img className={styles.image} src={heroImage} alt="" />
      <div className={styles.text}>
        <h1 className={styles.title}>{t('home.welcome')}</h1>
        <p className={styles.tagline}>{t('home.tagline')}</p>
        {engine && unfinished ? (
          <div className={styles.actions}>
            <Button
              size="large"
              onClick={() => {
                void resumeLast(unfinished.id);
              }}
            >
              <RotateCcw aria-hidden />
              {t('home.resume')}
            </Button>
          </div>
        ) : null}
        {engine ? null : (
          <div className={styles.actions}>
            <Button
              variant="primary"
              size="large"
              onClick={() => {
                navigation.request('settings');
              }}
            >
              <Wrench aria-hidden />
              {t('engine.setUp')}
            </Button>
            <p className={styles.hint}>{t('engine.needed')}</p>
          </div>
        )}
      </div>
    </section>
  );
});
