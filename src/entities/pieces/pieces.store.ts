import { makeAutoObservable, runInAction } from 'mobx';

import { pickFile, pickFolder } from '@shared/api/system.service';
import {
  deletePieceSet,
  fetchPieceSets,
  importPieceFolder,
  importPieceSheet,
  previewPieceSheet,
} from '@shared/api/pieces.service';
import type {
  PieceSetView,
  SheetColumnOrder,
} from '@shared/contracts/generated/bindings';

import { BUNDLED_SETS, DEFAULT_SET_ID } from './bundled-sets';

type PieceImages = PieceSetView['images'];

const SHEET_EXTENSIONS = ['png', 'webp', 'jpg', 'jpeg'];

export class PiecesStore {
  customSets: PieceSetView[] = [];
  preview: PieceImages | null = null;
  error: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  private attempt = async (action: () => Promise<void>) => {
    try {
      await action();
      runInAction(() => {
        this.error = null;
      });
    } catch (error) {
      runInAction(() => {
        this.error = String(error);
      });
    }
  };

  start = () =>
    this.attempt(async () => {
      const sets = await fetchPieceSets();

      runInAction(() => {
        this.customSets = sets;
      });
    });

  get allSets() {
    return [...BUNDLED_SETS, ...this.customSets];
  }

  /** A deleted custom set falls back to the default, never to blank pieces. */
  imagesFor = (setId: string): PieceImages => {
    const chosen = this.allSets.find((set) => set.id === setId);
    const fallback = this.allSets.find((set) => set.id === DEFAULT_SET_ID);

    return chosen?.images ?? fallback?.images ?? {};
  };

  pickSheet = () => pickFile(SHEET_EXTENSIONS);

  pickFolder = () => pickFolder();

  previewSheet = (path: string, order: SheetColumnOrder) =>
    this.attempt(async () => {
      const preview = await previewPieceSheet(path, order);

      runInAction(() => {
        this.preview = preview;
      });
    });

  clearPreview = () => {
    this.preview = null;
  };

  private add = (set: PieceSetView) => {
    this.customSets.push(set);
    this.preview = null;
  };

  importSheet = (name: string, path: string, order: SheetColumnOrder) =>
    this.attempt(async () => {
      const set = await importPieceSheet(name, path, order);

      runInAction(() => {
        this.add(set);
      });
    });

  importFolder = (name: string, path: string) =>
    this.attempt(async () => {
      const set = await importPieceFolder(name, path);

      runInAction(() => {
        this.add(set);
      });
    });

  remove = (id: string) =>
    this.attempt(async () => {
      await deletePieceSet(id);

      runInAction(() => {
        this.customSets = this.customSets.filter((set) => set.id !== id);
      });
    });
}
