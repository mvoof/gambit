import { makeAutoObservable, runInAction } from 'mobx';

import {
  chooseEngine,
  fetchEngineCatalog,
  fetchEngineStatus,
  installEngine,
  removeEngine,
} from '@shared/api/engine.service';
import { onEngineInstall } from '@shared/api/events.service';
import { pickFile } from '@shared/api/system.service';
import type {
  CatalogEntry,
  EngineInfo,
  EngineInstallProgress,
} from '@shared/contracts/generated/bindings';

export class EngineStore {
  engine: EngineInfo | null = null;
  catalog: CatalogEntry[] = [];
  isLoaded = false;
  installingId: string | null = null;
  progress: EngineInstallProgress | null = null;
  error: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  get isInstalling() {
    return this.installingId !== null;
  }

  start = async () => {
    await onEngineInstall((progress) => {
      runInAction(() => {
        this.progress = progress;
      });
    });
    await Promise.all([this.refresh(), this.refreshCatalog()]);
  };

  refreshCatalog = async () => {
    try {
      const catalog = await fetchEngineCatalog();

      runInAction(() => {
        this.catalog = catalog;
      });
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }
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

  private apply = async (action: () => Promise<EngineInfo>) => {
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

  install = async (id: string) => {
    this.installingId = id;
    await this.apply(() => installEngine(id));
    await this.refreshCatalog();

    runInAction(() => {
      this.installingId = null;
      this.progress = null;
    });
  };

  activate = (path: string) => this.apply(() => chooseEngine(path));

  remove = async (id: string) => {
    this.error = null;

    try {
      await removeEngine(id);
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }

    await this.refreshCatalog();
  };

  /** `true` when an engine was chosen, so the settings can reload. */
  pickOwnEngine = async () => {
    // No filter: a Linux engine has no extension to filter by.
    const path = await pickFile();

    if (!path) {
      return false;
    }

    await this.apply(() => chooseEngine(path));

    return true;
  };
}
