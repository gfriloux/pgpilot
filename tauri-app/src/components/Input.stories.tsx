import type { Meta, StoryObj } from '@storybook/react';
import { Input } from './Input';

const meta: Meta<typeof Input> = {
  title: 'Components/Input',
  component: Input,
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <div style={{ width: '320px', background: 'var(--card-bg)', padding: '24px', borderRadius: '10px' }}>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof Input>;

export const Default: Story = {
  args: {
    label: 'Name',
    placeholder: 'Alice Liddell',
  },
};

export const WithHint: Story = {
  args: {
    label: 'Email',
    type: 'email',
    placeholder: 'alice@example.com',
    hint: 'Used as the key UID email.',
  },
};

export const WithError: Story = {
  args: {
    label: 'Fingerprint',
    value: 'DEAD',
    error: 'Must be exactly 40 hexadecimal characters.',
    onChange: () => undefined,
  },
};

export const Password: Story = {
  args: {
    label: 'Passphrase',
    type: 'password',
    placeholder: '••••••••',
  },
};

export const Disabled: Story = {
  args: {
    label: 'Key ID',
    value: 'A1B2C3D4E5F6',
    disabled: true,
    onChange: () => undefined,
  },
};

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', width: '320px', background: 'var(--card-bg)', padding: '24px', borderRadius: '10px' }}>
      <Input label="Default" placeholder="Placeholder text" />
      <Input label="With hint" placeholder="alice@example.com" hint="Used as the key UID email." />
      <Input label="With error" value="DEAD" error="Must be 40 hex chars." onChange={() => undefined} />
      <Input label="Password" type="password" placeholder="••••••••" />
      <Input label="Disabled" value="Fixed value" disabled onChange={() => undefined} />
    </div>
  ),
};
