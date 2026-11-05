import { makeAutoObservable, runInAction } from 'mobx';

import {
  chooseEngine,
  downloadStockfish,
  fetchEngineStatus,
} from '@shared/api/engine.service';
import { onEngineInstall } from '@shared/api/events.service';
import { pickFile } from '@shared/api/system.service';
import type {
  EngineInfo,
  EngineInstallProgress,
} from '@shared/contracts/generated/bindings';

const ENGINE_EXTENSIONS = ['exe'];

export class EngineStore {
  engine: EngineInfo | null = null;
  isLoaded = false;
  isInstalling = false;
  progress: EngineInstallProgress | null = null;
  error: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  start = async () => {
    await onEngineInstall((progress) => {
      runInAction(() => {
        this.progress = progress;
      });
    });
    await this.refresh();
  };

  refresh = async () => {
    try {
      const engine = await fetchEngineStatus();

      runInAction(() => {
        this.engine = engine;
        this.isLoaded = true;
        this.error = null;
      });
    } catch (error) {
      runInAction(() => {
        this.engine = null;
        this.isLoaded = true;
        this.error = String(error);
      });
    }
  };

  private install = async (action: () => Promise<EngineInfo>) => {
    this.error = null;

    try {
      const engine = await action();

      runInAction(() => {
        this.engine = engine;
      });
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }
  };

  download = async () => {
    this.isInstalling = true;
    await this.install(downloadStockfish);

    runInAction(() => {
      this.isInstalling = false;
      this.progress = null;
    });
  };

  /** `true` when an engine was chosen, so the settings can reload. */
  pickOwnEngine = async () => {
    const path = await pickFile(ENGINE_EXTENSIONS);

    if (!path) {
      return false;
    }

    await this.install(() => chooseEngine(path));

    return true;
  };
}
