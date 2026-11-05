import { makeAutoObservable, runInAction } from 'mobx';

import {
  askCompanion,
  fetchCompanionView,
  fetchLlmModels,
  requestAdvice,
  stopCompanion,
} from '@shared/api/companion.service';
import { onCompanionUpdated } from '@shared/api/events.service';
import type { CompanionView } from '@shared/contracts/generated/bindings';

export class CompanionStore {
  view: CompanionView = {
    turns: [],
    draft: null,
    loadingModel: false,
    error: null,
  };
  models: string[] = [];
  isChecking = false;
  connectionError: string | null = null;
  askError: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  start = async () => {
    await onCompanionUpdated(this.receive);
    this.receive(await fetchCompanionView());
  };

  private receive = (view: CompanionView) => {
    this.view = view;
  };

  get isReplying() {
    return this.view.draft !== null;
  }

  get error() {
    return this.askError ?? this.view.error;
  }

  checkConnection = async (baseUrl: string) => {
    this.isChecking = true;
    this.connectionError = null;

    try {
      const models = await fetchLlmModels(baseUrl);

      runInAction(() => {
        this.models = models;
      });
    } catch (error) {
      runInAction(() => {
        this.models = [];
        this.connectionError = String(error);
      });
    }

    runInAction(() => {
      this.isChecking = false;
    });
  };

  ask = async (text: string, ply: number | null) => {
    this.askError = null;

    try {
      await askCompanion(text, ply);
    } catch (error) {
      runInAction(() => {
        this.askError = String(error);
      });
    }
  };

  advise = async () => {
    try {
      await requestAdvice();
    } catch (error) {
      runInAction(() => {
        this.askError = String(error);
      });
    }
  };

  stop = () => stopCompanion();
}
