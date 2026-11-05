import type { Meta, StoryObj } from '@storybook/react-vite';

import { ResultBadge } from './ResultBadge';

const meta: Meta<typeof ResultBadge> = {
  title: 'entities/ResultBadge',
  component: ResultBadge,
  args: { score: '1-0' },
};

export default meta;

type Story = StoryObj<typeof ResultBadge>;

export const Win: Story = { args: { result: 'win' } };
export const Loss: Story = { args: { result: 'loss' } };
export const Draw: Story = { args: { result: 'draw' } };
export const Unfinished: Story = { args: { result: 'unfinished' } };
export const Imported: Story = { args: { result: 'unknown' } };
