import type { Meta, StoryObj } from '@storybook/react-vite';
import { CircleAlert, Crown, Handshake } from 'lucide-react';

import { GameBanner } from './GameBanner';

const meta: Meta<typeof GameBanner> = {
  title: 'Game/GameBanner',
  component: GameBanner,
};

export default meta;

type Story = StoryObj<typeof GameBanner>;

export const Check: Story = {
  args: {
    tone: 'danger',
    icon: CircleAlert,
    title: 'Check!',
  },
};

export const Mate: Story = {
  args: {
    tone: 'success',
    icon: Crown,
    title: 'Checkmate!',
    text: 'White wins',
  },
};

export const Draw: Story = {
  args: {
    tone: 'neutral',
    icon: Handshake,
    title: 'Draw',
    text: 'stalemate',
  },
};
