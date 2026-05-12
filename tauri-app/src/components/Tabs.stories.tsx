import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { Tabs } from './Tabs';

const meta: Meta<typeof Tabs> = {
  title: 'Components/Tabs',
  component: Tabs,
  tags: ['autodocs'],
  parameters: { layout: 'padded' },
};

export default meta;
type Story = StoryObj<typeof Tabs>;

const KEY_TABS = [
  { id: 'info', label: 'Key info' },
  { id: 'subkeys', label: 'Subkeys' },
  { id: 'trust', label: 'Trust' },
];

function Controlled() {
  const [active, setActive] = useState('info');
  return (
    <div style={{ background: 'var(--card-bg)', borderRadius: '10px', padding: '0 0 24px' }}>
      <Tabs tabs={KEY_TABS} activeTab={active} onChange={setActive} />
      <div style={{ padding: '20px 16px', color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
        Active panel: <strong style={{ color: 'var(--text-strong)' }}>{active}</strong>
      </div>
    </div>
  );
}

export const Default: Story = {
  render: () => <Controlled />,
};

export const TwoTabs: Story = {
  render: () => {
    const [active, setActive] = useState('encrypt');
    return (
      <div style={{ background: 'var(--card-bg)', borderRadius: '10px', padding: '0 0 24px' }}>
        <Tabs
          tabs={[{ id: 'encrypt', label: 'Encrypt' }, { id: 'decrypt', label: 'Decrypt' }]}
          activeTab={active}
          onChange={setActive}
        />
        <div style={{ padding: '20px 16px', color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
          Active: <strong style={{ color: 'var(--text-strong)' }}>{active}</strong>
        </div>
      </div>
    );
  },
};
