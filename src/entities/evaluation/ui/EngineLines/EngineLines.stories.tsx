import type { Meta, StoryObj } from '@storybook/react-vite';

import { EngineLines } from './EngineLines';

const meta: Meta<typeof EngineLines> = {
  title: 'entities/EngineLines',
  component: EngineLines,
  args: {
    turn: 'white',
    lines: [
      { moves: ['1. e4', 'e5', '2. Nf3'], centipawns: 35, mateIn: null },
      { moves: ['1. d4', 'Nf6', '2. c4'], centipawns: 28, mateIn: null },
      { moves: ['1. Qh5'], centipawns: null, mateIn: 3 },
    ],
  },
};

export default meta;

type Story = StoryObj<typeof EngineLines>;

export const Lines: Story = {};

export const BlackToMove: Story = {
  args: {
    turn: 'black',
    lines: [{ moves: ['12... Nc6', '13. d4'], centipawns: -40, mateIn: null }],
  },
};

export const Waiting: Story = { args: { lines: [], turn: null } };
