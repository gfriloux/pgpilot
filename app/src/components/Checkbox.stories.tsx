import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { Checkbox } from './Checkbox';

const meta: Meta<typeof Checkbox> = {
  title: 'Components/Checkbox',
  component: Checkbox,
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof Checkbox>;

function Controlled(props: { label: string; indeterminate?: boolean; disabled?: boolean }) {
  const [checked, setChecked] = useState(false);
  return (
    <Checkbox
      label={props.label}
      checked={checked}
      onChange={setChecked}
      indeterminate={props.indeterminate ?? false}
      disabled={props.disabled ?? false}
    />
  );
}

export const Unchecked: Story = {
  render: () => <Controlled label="Armor output (.asc)" />,
};

export const Checked: Story = {
  render: () => {
    const [c, setC] = useState(true);
    return <Checkbox label="Force trust model" checked={c} onChange={setC} />;
  },
};

export const Indeterminate: Story = {
  render: () => {
    const [c, setC] = useState(false);
    return (
      <Checkbox label="Select all recipients" checked={c} onChange={setC} indeterminate />
    );
  },
};

export const Disabled: Story = {
  render: () => <Controlled label="Hardware key (unavailable)" disabled />,
};

export const AllStates: Story = {
  render: () => {
    const [a, setA] = useState(false);
    const [b, setB] = useState(true);
    const [d, setD] = useState(false);
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        <Checkbox label="Unchecked" checked={a} onChange={setA} />
        <Checkbox label="Checked" checked={b} onChange={setB} />
        <Checkbox label="Indeterminate" checked={d} onChange={setD} indeterminate />
        <Checkbox label="Disabled" checked={false} onChange={() => undefined} disabled />
      </div>
    );
  },
};
