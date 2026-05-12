import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { Modal } from './Modal';
import { Button } from './Button';

const meta: Meta<typeof Modal> = {
  title: 'Components/Modal',
  component: Modal,
  tags: ['autodocs'],
  parameters: { layout: 'fullscreen' },
};

export default meta;
type Story = StoryObj<typeof Modal>;

function ModalDemo({ title, children }: { title: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <div style={{ padding: '32px', minHeight: '200px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <Button onClick={() => setOpen(true)}>Open modal</Button>
      {open && (
        <Modal title={title} onClose={() => setOpen(false)}>
          {children}
        </Modal>
      )}
    </div>
  );
}

export const Default: Story = {
  render: () => (
    <ModalDemo title="Delete key">
      <p>Are you sure you want to delete the key for <strong>Alice Liddell</strong>? This action cannot be undone.</p>
      <div style={{ display: 'flex', gap: '8px', marginTop: '20px', justifyContent: 'flex-end' }}>
        <Button variant="ghost">Cancel</Button>
        <Button variant="danger">Delete</Button>
      </div>
    </ModalDemo>
  ),
};

export const WithForm: Story = {
  render: () => (
    <ModalDemo title="Publish to keyserver">
      <p style={{ marginBottom: '16px' }}>Choose a keyserver to publish to:</p>
      <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end', marginTop: '8px' }}>
        <Button variant="ghost">Cancel</Button>
        <Button variant="primary">Publish</Button>
      </div>
    </ModalDemo>
  ),
};
