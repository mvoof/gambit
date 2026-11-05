import type { Meta, StoryObj } from '@storybook/react-vite';

import type { MoveRecord } from '@shared/contracts/generated/bindings';

import { MoveList } from './MoveList';

const move = (
  ply: number,
  san: string,
  color: MoveRecord['color']
): MoveRecord => ({
  ply,
  san,
  uci: '',
  from: '',
  to: '',
  color,
  sound: 'move',
});

const meta: Meta<typeof MoveList> = {
  title: 'entities/MoveList',
  component: MoveList,
  args: {
    activePly: 3,
    onSelect: () => undefined,
    moves: [
      move(1, 'e4', 'white'),
      move(2, 'e5', 'black'),
      move(3, 'Nf3', 'white'),
      move(4, 'Nc6', 'black'),
    ],
  },
};

export default meta;

type Story = StoryObj<typeof MoveList>;

export const WithMarks: Story = {
  args: {
    marks: [{ ply: 3, quality: 'blunder', bestSan: 'd4' }],
  },
};

export const FromTheStart: Story = {};

export const BlackMovesFirst: Story = {
  args: { moves: [move(1, 'Kd7', 'black'), move(2, 'e4', 'white')] },
};
