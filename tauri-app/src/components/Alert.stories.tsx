import type { Meta, StoryObj } from '@storybook/react';
import { Alert } from './Alert';

const meta: Meta<typeof Alert> = {
  title: 'Components/Alert',
  component: Alert,
  tags: ['autodocs'],
  parameters: { layout: 'padded' },
  argTypes: {
    variant: {
      control: 'select',
      options: ['success', 'error', 'warning', 'info'],
    },
    dismissible: { control: 'boolean' },
  },
  decorators: [
    (Story) => (
      <div style={{ maxWidth: '480px' }}>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof Alert>;

export const Success: Story = {
  args: {
    variant: 'success',
    title: 'Key published',
    message: 'Your public key has been uploaded to keys.openpgp.org.',
  },
};

export const Error: Story = {
  args: {
    variant: 'error',
    title: 'Export failed',
    message: 'Could not write to the selected directory. Check permissions.',
  },
};

export const Warning: Story = {
  args: {
    variant: 'warning',
    title: 'Expiring soon',
    message: 'Signing subkey expires in 14 days. Consider renewing.',
  },
};

export const Info: Story = {
  args: {
    variant: 'info',
    message: 'Importing from URL requires an internet connection.',
  },
};

export const Dismissible: Story = {
  args: {
    variant: 'success',
    message: 'Key created successfully. You can dismiss this.',
    dismissible: true,
  },
};

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', maxWidth: '480px' }}>
      <Alert variant="success" title="Published" message="Key is live on keys.openpgp.org." dismissible />
      <Alert variant="error" title="Error" message="gpg: key not found in keyring." />
      <Alert variant="warning" message="Subkey expires in 14 days." dismissible />
      <Alert variant="info" title="Note" message="Drag and drop files to encrypt them." />
    </div>
  ),
};

export const BothThemes: Story = {
  render: () => (
    <div style={{ padding: '24px', borderRadius: '10px' }}>
      <p style={{ color: 'var(--text-muted)', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '16px' }}>
        Active theme
      </p>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', maxWidth: '440px' }}>
        <Alert variant="success" title="Key published" message="Uploaded to keys.openpgp.org." />
        <Alert variant="error" title="Export failed" message="Check directory permissions." />
        <Alert variant="warning" message="Signing subkey expires in 14 days." />
        <Alert variant="info" title="Tip" message="Drag files onto Encrypt view." />
      </div>
    </div>
  ),
};
