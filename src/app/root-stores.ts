import { ArchiveStore } from '@entities/archive/archive.store';
import { EngineStore } from '@entities/engine/engine.store';
import { EvaluationStore } from '@entities/evaluation/evaluation.store';
import { GameStore } from '@entities/game/game.store';
import { NavigationStore } from '@entities/navigation/navigation.store';
import { PiecesStore } from '@entities/pieces/pieces.store';
import { SettingsStore } from '@entities/settings/settings.store';
import { UpdateStore } from '@entities/update/update.store';

export const createRootStores = () => ({
  archive: new ArchiveStore(),
  engine: new EngineStore(),
  evaluation: new EvaluationStore(),
  game: new GameStore(),
  navigation: new NavigationStore(),
  pieces: new PiecesStore(),
  settings: new SettingsStore(),
  update: new UpdateStore(),
});

export type RootStores = ReturnType<typeof createRootStores>;
