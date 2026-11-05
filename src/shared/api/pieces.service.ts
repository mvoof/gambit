import { invoke } from '@tauri-apps/api/core';

import type {
  PieceSetView,
  SheetColumnOrder,
} from '@shared/contracts/generated/bindings';

type PieceImages = PieceSetView['images'];

export const fetchPieceSets = () => invoke<PieceSetView[]>('list_piece_sets');

export const previewPieceSheet = (path: string, order: SheetColumnOrder) =>
  invoke<PieceImages>('preview_piece_sheet', { path, order });

export const importPieceSheet = (
  name: string,
  path: string,
  order: SheetColumnOrder
) => invoke<PieceSetView>('import_piece_sheet', { name, path, order });

export const importPieceFolder = (name: string, path: string) =>
  invoke<PieceSetView>('import_piece_folder', { name, path });

export const deletePieceSet = (id: string) =>
  invoke<void>('delete_piece_set', { id });
