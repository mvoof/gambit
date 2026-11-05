import { makeAutoObservable, runInAction } from 'mobx';

import { checkForUpdate, relaunchApp } from '@shared/api/update.service';
import type {
  AppUpdate,
  UpdateDownloadEvent,
} from '@shared/api/update.service';

const PERCENT = 100;

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'upToDate'
  | 'available'
  | 'installing'
  | 'failed';

export class UpdateStore {
  status: UpdateStatus = 'idle';
  version: string | null = null;
  percentDone = 0;
  error: string | null = null;

  private update: AppUpdate | null = null;
  private totalBytes = 0;
  private downloadedBytes = 0;

  constructor() {
    makeAutoObservable(this);
  }

  /** The background check at startup: being offline is not worth a message. */
  checkQuietly = async () => {
    try {
      await this.lookUp();
    } catch {
      runInAction(() => {
        this.status = 'idle';
      });
    }
  };

  /** The check the player asked for: a failure is shown. */
  check = async () => {
    try {
      await this.lookUp();
    } catch (error) {
      this.fail(error);
    }
  };

  private lookUp = async () => {
    this.status = 'checking';
    this.error = null;

    const update = await checkForUpdate();

    runInAction(() => {
      this.update = update;
      this.version = update?.version ?? null;
      this.status = update ? 'available' : 'upToDate';
    });
  };

  private fail = (error: unknown) => {
    runInAction(() => {
      this.status = 'failed';
      this.error = String(error);
    });
  };

  private track = (event: UpdateDownloadEvent) => {
    runInAction(() => {
      if (event.event === 'Started') {
        this.totalBytes = event.data.contentLength ?? 0;
      } else if (event.event === 'Progress') {
        this.downloadedBytes += event.data.chunkLength;

        if (this.totalBytes > 0) {
          this.percentDone = Math.round(
            (this.downloadedBytes / this.totalBytes) * PERCENT
          );
        }
      }
    });
  };

  install = async () => {
    if (!this.update) {
      return;
    }

    this.status = 'installing';
    this.percentDone = 0;
    this.downloadedBytes = 0;

    try {
      await this.update.downloadAndInstall(this.track);
      await relaunchApp();
    } catch (error) {
      this.fail(error);
    }
  };
}
