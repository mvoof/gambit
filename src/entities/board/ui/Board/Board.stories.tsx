import type { Meta, StoryObj } from '@storybook/react-vite';

import type {
  BoardPiece,
  BoardSettings,
  MoveOption,
} from '@shared/contracts/generated/bindings';

import { Board } from './Board';

const IMAGE_MODULES = import.meta.glob<string>(
  '/src/assets/pieces/cburnett/*.svg',
  { eager: true, query: '?url', import: 'default' }
);

const images = Object.fromEntries(
  Object.entries(IMAGE_MODULES).map(([path, url]) => [
    path.slice(path.lastIndexOf('/') + 1, path.lastIndexOf('.')),
    url,
  ])
);

const settings: BoardSettings = {
  lightSquare: '#e3d3b5',
  darkSquare: '#8a6a4a',
  lastMoveColor: '#f2c94c',
  showCoordinates: true,
  showLegalMoves: true,
  showLastMove: true,
  showCheck: true,
};

const pieces: BoardPiece[] = [
  { id: 1, color: 'white', kind: 'king', square: 'e1' },
  { id: 2, color: 'white', kind: 'queen', square: 'h5' },
  { id: 3, color: 'white', kind: 'pawn', square: 'e7' },
  { id: 4, color: 'black', kind: 'king', square: 'g8' },
  { id: 5, color: 'black', kind: 'rook', square: 'a8' },
];

const queenMove: MoveOption = {
  from: 'h5',
  to: 'h7',
  uci: 'h5h7',
  promotion: null,
};

const meta: Meta<typeof Board> = {
  title: 'entities/Board',
  component: Board,
  decorators: [
    (Story) => (
      <div style={{ width: 480 }}>
        <Story />
      </div>
    ),
  ],
  args: { pieces, images, orientation: 'white', settings },
};

export default meta;

type Story = StoryObj<typeof Board>;

export const Plain: Story = {};

export const Flipped: Story = { args: { orientation: 'black' } };

export const WithLastMoveAndCheck: Story = {
  args: {
    lastMove: { from: 'h1', to: 'h5' },
    checkSquare: 'g8',
  },
};

export const WithHint: Story = {
  args: { hint: { from: 'e2', to: 'e4' } },
};

export const WithMoves: Story = { args: { moveOptions: [queenMove] } };

export const WithPremove: Story = {
  args: { premove: queenMove, premoveOptions: [queenMove] },
};

export const NoCoordinates: Story = {
  args: { settings: { ...settings, showCoordinates: false } },
};
