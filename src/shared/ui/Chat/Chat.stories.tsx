import type { Meta, StoryObj } from '@storybook/react-vite';

import { Chat } from './Chat';

const meta: Meta<typeof Chat> = {
  title: 'shared/Chat',
  component: Chat,
  args: {
    emptyText: 'No messages yet.',
    placeholder: 'Write a message',
    sendLabel: 'Send',
    onSend: () => undefined,
    messages: [
      { id: '1', isOwn: true, text: 'Good game!' },
      { id: '2', isOwn: false, text: 'Thanks, one more?' },
    ],
  },
};

export default meta;

type Story = StoryObj<typeof Chat>;

export const Conversation: Story = {};

export const Empty: Story = { args: { messages: [] } };
