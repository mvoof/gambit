import type { Meta, StoryObj } from '@storybook/react-vite';

import { EngineLines } from './EngineLines';

const meta: Meta<typeof EngineLines> = {
  title: 'entities/EngineLines',
  component: EngineLines,
  args: {
    lines: [
      { moves: ['e4', 'e5', 'Nf3'], centipawns: 35, mateIn: null },
      { moves: ['d4', 'Nf6', 'c4'], centipawns: 28, mateIn: null },
      { moves: ['Qh5'], centipawns: null, mateIn: 3 },
    ],
  },
};

export default meta;

type Story = StoryObj<typeof EngineLines>;

export const Lines: Story = {};

export const Waiting: Story = { args: { lines: [] } };
