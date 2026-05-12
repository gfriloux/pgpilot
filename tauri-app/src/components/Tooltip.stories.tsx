import type { Meta, StoryObj } from '@storybook/react';
import { Tooltip } from './Tooltip';
import { Button } from './Button';

const meta: Meta<typeof Tooltip> = {
  title: 'Components/Tooltip',
  component: Tooltip,
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
  decorators: [
    (Story) => (
      <div style={{ padding: '80px 120px' }}>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof Tooltip>;

export const Top: Story = {
  args: {
    content: 'Copy fingerprint',
    placement: 'top',
    children: <Button variant="ghost">Hover me</Button>,
  },
};

export const Bottom: Story = {
  args: {
    content: 'Publish to keyserver',
    placement: 'bottom',
    children: <Button variant="primary">Publish</Button>,
  },
};

export const Left: Story = {
  args: {
    content: 'Dangerous action',
    placement: 'left',
    children: <Button variant="destructive">Delete</Button>,
  },
};

export const Right: Story = {
  args: {
    content: 'YubiKey connected',
    placement: 'right',
    children: <Button variant="ghost">Migrate</Button>,
  },
};

export const AllPlacements: Story = {
  render: () => (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '48px', padding: '40px' }}>
      <Tooltip content="Top tooltip" placement="top">
        <Button variant="ghost">Top</Button>
      </Tooltip>
      <Tooltip content="Bottom tooltip" placement="bottom">
        <Button variant="ghost">Bottom</Button>
      </Tooltip>
      <Tooltip content="Left tooltip" placement="left">
        <Button variant="ghost">Left</Button>
      </Tooltip>
      <Tooltip content="Right tooltip" placement="right">
        <Button variant="ghost">Right</Button>
      </Tooltip>
    </div>
  ),
};
