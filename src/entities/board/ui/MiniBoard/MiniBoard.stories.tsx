import type { Meta, StoryObj } from '@storybook/react-vite';

import { MiniBoard } from './MiniBoard';

const meta: Meta<typeof MiniBoard> = {
  title: 'entities/MiniBoard',
  component: MiniBoard,
  args: {
    images: {},
    lightSquare: '#e3d3b5',
    darkSquare: '#8a6a4a',
    fen: 'r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3',
    className: undefined,
  },
};

export default meta;

export const Default: StoryObj<typeof MiniBoard> = {};
