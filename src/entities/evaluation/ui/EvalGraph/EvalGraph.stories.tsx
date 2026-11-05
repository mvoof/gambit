import type { Meta, StoryObj } from '@storybook/react-vite';

import type { EvalPoint } from '@shared/contracts/generated/bindings';

import { EvalGraph } from './EvalGraph';

const point = (ply: number, whiteWinPercent: number): EvalPoint => ({
  ply,
  whiteWinPercent,
  centipawns: Math.round((whiteWinPercent - 50) * 10),
  mateIn: null,
  isBest: false,
});

const POINTS = [50, 52, 55, 61, 58, 64, 70, 66, 40, 34, 28, 12].map(
  (percent, ply) => point(ply, percent)
);

const meta: Meta<typeof EvalGraph> = {
  title: 'entities/EvalGraph',
  component: EvalGraph,
  args: {
    points: POINTS,
    mistakePlies: [8, 11],
    activePly: 6,
    lastPly: 11,
    onSelect: () => undefined,
  },
  decorators: [
    (Story) => (
      <div style={{ display: 'flex', width: 360, height: 90 }}>
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof EvalGraph>;

export const Default: Story = {};
export const Measuring: Story = {
  args: { points: POINTS.slice(0, 5), mistakePlies: [] },
};
export const Pending: Story = { args: { points: [] } };
