import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';

import { Select } from './Select';

const OPTIONS = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
];

const SelectDemo = () => {
  const [value, setValue] = useState('newest');

  return (
    <Select label="Sort" options={OPTIONS} value={value} onChange={setValue} />
  );
};

const meta: Meta<typeof SelectDemo> = {
  title: 'shared/Select',
  component: SelectDemo,
};

export default meta;

export const Default: StoryObj<typeof SelectDemo> = {};
