import type { PieceSetView } from '@shared/contracts/generated/bindings';

export const DEFAULT_SET_ID = 'cburnett';

const SET_NAMES: Record<string, string> = {
  cburnett: 'CBurnett',
  merida: 'Merida',
  chessnut: 'Chessnut',
  fantasy: 'Fantasy',
};

const IMAGE_MODULES = import.meta.glob<string>('/src/assets/pieces/*/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
});

const SET_FOLDER_FROM_END = 2;

const setOfPath = (path: string) =>
  path.split('/').at(-SET_FOLDER_FROM_END) ?? '';

const codeOfPath = (path: string) =>
  path.slice(path.lastIndexOf('/') + 1, path.lastIndexOf('.'));

const imagesOfSet = (setId: string) =>
  Object.fromEntries(
    Object.entries(IMAGE_MODULES)
      .filter(([path]) => setOfPath(path) === setId)
      .map(([path, url]) => [codeOfPath(path), url])
  );

export const BUNDLED_SETS: PieceSetView[] = Object.entries(SET_NAMES).map(
  ([id, name]) => ({ id, name, images: imagesOfSet(id) })
);
