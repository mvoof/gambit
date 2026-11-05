import { makeAutoObservable, runInAction } from 'mobx';

import {
  fetchSettings,
  resetSettings,
  saveSettings,
} from '@shared/api/settings.service';
import type {
  AppSettings,
  BoardSettings,
} from '@shared/contracts/generated/bindings';

export class SettingsStore {
  settings: AppSettings | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  start = async () => {
    const settings = await fetchSettings();

    runInAction(() => {
      this.settings = settings;
    });
  };

  update = async (patch: Partial<AppSettings>) => {
    if (!this.settings) {
      return;
    }

    const next = { ...this.settings, ...patch };

    // Applied first: a slider must not wait for the disk.
    this.settings = next;

    const saved = await saveSettings(next);

    runInAction(() => {
      this.settings = saved;
    });
  };

  updateBoard = (patch: Partial<BoardSettings>) => {
    if (!this.settings) {
      return Promise.resolve();
    }

    return this.update({ board: { ...this.settings.board, ...patch } });
  };

  reset = async () => {
    const settings = await resetSettings();

    runInAction(() => {
      this.settings = settings;
    });
  };

  /** The engine path is written by the backend when an engine is chosen. */
  reload = () => this.start();
}
