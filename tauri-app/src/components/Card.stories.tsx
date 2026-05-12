import type { Meta, StoryObj } from '@storybook/react';
import { Card } from './Card';

const meta: Meta<typeof Card> = {
  title: 'Components/Card',
  component: Card,
  tags: ['autodocs'],
  parameters: { layout: 'padded' },
};

export default meta;
type Story = StoryObj<typeof Card>;

const SampleContent = () => (
  <>
    <h2 style={{ fontFamily: 'var(--font-heading)', color: 'var(--text-header)', marginBottom: '8px' }}>
      Alice Liddell
    </h2>
    <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
      alice@example.com — Expires 2026-01-01
    </p>
  </>
);

export const Default: Story = {
  args: {
    variant: 'default',
    children: <SampleContent />,
  },
};

export const Elevated: Story = {
  args: {
    variant: 'elevated',
    children: <SampleContent />,
  },
};

export const BothVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '24px', flexWrap: 'wrap', padding: '32px' }}>
      <Card variant="default" style={{ minWidth: '240px' }}>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.7rem', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '8px' }}>Default</p>
        <SampleContent />
      </Card>
      <Card variant="elevated" style={{ minWidth: '240px' }}>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.7rem', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '8px' }}>Elevated</p>
        <SampleContent />
      </Card>
    </div>
  ),
};
