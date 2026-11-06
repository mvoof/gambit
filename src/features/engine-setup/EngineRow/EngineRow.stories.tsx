import type { Meta, StoryObj } from '@storybook/react-vite';

import { EngineRow } from './EngineRow';

const INSTALLED_PATH = '/engines/reckless/reckless';

const ENTRY = {
  id: 'reckless',
  name: 'Reckless',
  version: '0.9.0',
  licence: 'AGPL-3.0',
  homepage: 'https://github.com/codedeliveryservice/Reckless',
  sizeBytes: 64_664_576,
  path: null,
};

const meta: Meta<typeof EngineRow> = {
  title: 'Features/EngineRow',
  component: EngineRow,
  args: {
    entry: ENTRY,
    isActive: false,
    isBusy: false,
    installLabel: null,
  },
};

export default meta;

type Story = StoryObj<typeof EngineRow>;

export const NotInstalled: Story = {};

export const Installing: Story = {
  args: { isBusy: true, installLabel: 'Downloading… 42%' },
};

export const Installed: Story = {
  args: {
    entry: { ...ENTRY, path: INSTALLED_PATH },
  },
};

export const InUse: Story = {
  args: {
    entry: { ...ENTRY, path: INSTALLED_PATH },
    isActive: true,
  },
};
