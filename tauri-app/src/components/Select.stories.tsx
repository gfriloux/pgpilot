import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { Select } from './Select';

const KEYSERVER_OPTIONS = [
  { value: 'keys.openpgp.org', label: 'keys.openpgp.org' },
  { value: 'keyserver.ubuntu.com', label: 'keyserver.ubuntu.com' },
  { value: 'pgp.mit.edu', label: 'pgp.mit.edu' },
];

const meta: Meta<typeof Select> = {
  title: 'Components/Select',
  component: Select,
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <div style={{ width: '280px', background: 'var(--card-bg)', padding: '24px', borderRadius: '10px' }}>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof Select>;

function Controlled() {
  const [val, setVal] = useState('keys.openpgp.org');
  return (
    <Select
      label="Keyserver"
      options={KEYSERVER_OPTIONS}
      value={val}
      onChange={setVal}
    />
  );
}

export const Default: Story = {
  render: () => <Controlled />,
};

export const Disabled: Story = {
  render: () => (
    <Select
      label="Keyserver"
      options={KEYSERVER_OPTIONS}
      value="keys.openpgp.org"
      onChange={() => undefined}
      disabled
    />
  ),
};

export const AllStates: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', width: '280px', background: 'var(--card-bg)', padding: '24px', borderRadius: '10px' }}>
      <Controlled />
      <Select
        label="Disabled"
        options={KEYSERVER_OPTIONS}
        value="keys.openpgp.org"
        onChange={() => undefined}
        disabled
      />
    </div>
  ),
};
