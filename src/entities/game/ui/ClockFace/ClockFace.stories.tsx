import type { Meta, StoryObj } from '@storybook/react-vite';

import { ClockFace } from './ClockFace';

const meta: Meta<typeof ClockFace> = {
  title: 'entities/ClockFace',
  component: ClockFace,
  args: { color: 'white' },
};

export default meta;

type Story = StoryObj<typeof ClockFace>;

export const Waiting: Story = {
  args: {
    clock: { whiteMs: 300_000, blackMs: 300_000, running: 'black', syncMs: 1 },
  },
};

export const Running: Story = {
  args: {
    clock: { whiteMs: 300_000, blackMs: 300_000, running: 'white', syncMs: 1 },
  },
};

export const LowOnTime: Story = {
  args: {
    clock: { whiteMs: 12_000, blackMs: 300_000, running: 'white', syncMs: 1 },
  },
};
