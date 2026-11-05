import type { Meta, StoryObj } from '@storybook/react-vite';

import { ChatLog } from './ChatLog';

const meta: Meta<typeof ChatLog> = {
  title: 'entities/ChatLog',
  component: ChatLog,
  args: {
    draft: null,
    isLoadingModel: false,
    turns: [
      { speaker: 'player', text: 'What should I think about here?' },
      {
        speaker: 'companion',
        text: 'The engine likes Nf3: it develops a piece and eyes the centre.',
      },
    ],
  },
};

export default meta;

type Story = StoryObj<typeof ChatLog>;

export const Conversation: Story = {};

export const Empty: Story = { args: { turns: [] } };

export const Waiting: Story = { args: { draft: '' } };

export const LoadingModel: Story = {
  args: { draft: '', isLoadingModel: true },
};

export const Streaming: Story = { args: { draft: 'The engine likes' } };
