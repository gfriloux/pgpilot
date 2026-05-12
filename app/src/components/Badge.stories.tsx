import type { Meta, StoryObj } from '@storybook/react';
import { Badge } from './Badge';

const meta: Meta<typeof Badge> = {
  title: 'Components/Badge',
  component: Badge,
  tags: ['autodocs'],
  argTypes: {
    color: {
      control: 'select',
      options: ['success', 'error', 'warning', 'info', 'neutral'],
    },
    size: { control: 'select', options: ['sm', 'md'] },
  },
};

export default meta;
type Story = StoryObj<typeof Badge>;

export const Success: Story = { args: { color: 'success', children: 'Published' } };
export const Error: Story = { args: { color: 'error', children: 'Revoked' } };
export const Warning: Story = { args: { color: 'warning', children: 'Expiring soon' } };
export const Info: Story = { args: { color: 'info', children: 'YubiKey' } };
export const Neutral: Story = { args: { color: 'neutral', children: 'Unknown' } };
export const Small: Story = { args: { color: 'success', size: 'sm', children: 'OK' } };

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' }}>
      <Badge color="success">Published</Badge>
      <Badge color="error">Revoked</Badge>
      <Badge color="warning">Expiring soon</Badge>
      <Badge color="info">YubiKey</Badge>
      <Badge color="neutral">Unknown</Badge>
      <Badge color="success" size="sm">OK</Badge>
      <Badge color="error" size="sm">Error</Badge>
    </div>
  ),
};
