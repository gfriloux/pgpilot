import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { createKey } from '../ipc/keys';
import { useKeys } from '../hooks/useKeys';
import { useUiStore } from '../store/ui';
import { Input } from '../components/Input';
import { Select } from '../components/Select';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import styles from './CreateKey.module.css';

const EXPIRY_OPTIONS = [
  { value: '365',  label: '1 year' },
  { value: '730',  label: '2 years (recommended)' },
  { value: '1825', label: '5 years' },
];

function isValidEmail(value: string): boolean {
  return value.includes('@') && value.includes('.');
}

export default function CreateKey() {
  const navigate = useNavigate();
  const { reload } = useKeys();
  const setStatus = useUiStore((s) => s.setStatus);

  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [expiryDays, setExpiryDays] = useState('730');
  const [nameError, setNameError] = useState<string | undefined>(undefined);
  const [emailError, setEmailError] = useState<string | undefined>(undefined);
  const [createError, setCreateError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  function validate(): boolean {
    let ok = true;
    if (name.trim().length === 0) {
      setNameError('Full name is required.');
      ok = false;
    } else {
      setNameError(undefined);
    }
    if (email.trim().length === 0) {
      setEmailError('Email address is required.');
      ok = false;
    } else if (!isValidEmail(email.trim())) {
      setEmailError('Enter a valid email address.');
      ok = false;
    } else {
      setEmailError(undefined);
    }
    return ok;
  }

  function handleSubmit(e: React.FormEvent): void {
    e.preventDefault();
    if (!validate()) return;

    setCreateError(null);
    setLoading(true);

    createKey(name.trim(), email.trim(), parseInt(expiryDays, 10))
      .then(() => {
        reload();
        setStatus('success', `Key created for ${name.trim()}.`);
        void navigate('/');
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        setCreateError(msg);
      })
      .finally(() => {
        setLoading(false);
      });
  }

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <h1 className={styles.title}>Create Key</h1>
        <p className={styles.subtitle}>
          Generates a master certification key with Signature, Encryption, and Auth SSH subkeys.
          Key creation may take a few seconds.
        </p>

        {createError !== null && (
          <Alert variant="error" message={createError} dismissible />
        )}

        <form onSubmit={handleSubmit} noValidate className={styles.form}>
          <Input
            label="Full name"
            type="text"
            value={name}
            onChange={(e) => { setName(e.currentTarget.value); }}
            {...(nameError !== undefined ? { error: nameError } : {})}
            placeholder="Alice Dupont"
            disabled={loading}
            autoFocus
          />
          <Input
            label="Email address"
            type="email"
            value={email}
            onChange={(e) => { setEmail(e.currentTarget.value); }}
            {...(emailError !== undefined ? { error: emailError } : {})}
            placeholder="alice@example.com"
            disabled={loading}
          />
          <Select
            label="Subkey expiry"
            options={EXPIRY_OPTIONS}
            value={expiryDays}
            onChange={setExpiryDays}
            disabled={loading}
          />

          <div className={styles.actions}>
            <Button
              variant="ghost"
              size="md"
              type="button"
              disabled={loading}
              onClick={() => { void navigate('/'); }}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="md"
              type="submit"
              loading={loading}
            >
              {loading ? 'Creating…' : 'Create key'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
