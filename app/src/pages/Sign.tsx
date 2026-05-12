import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useKeysStore } from '../store/keys';
import { useConfigStore } from '../store/config';
import { signFile } from '../ipc/keys';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import { UssrBanner } from '../components/UssrBanner';
import styles from './Sign.module.css';

function basename(p: string): string {
  return p.split('/').pop() ?? p;
}

export default function Sign() {
  const allKeys = useKeysStore((s) => s.keys);
  const theme = useConfigStore((s) => s.theme);

  // Only keys with a secret part that have a signing subkey
  const signingKeys = allKeys.filter(
    (k) =>
      k.has_secret &&
      k.subkeys.some((sk) => sk.usage.includes('S')),
  );

  const [file, setFile] = useState<string | null>(null);
  const [signerFp, setSignerFp] = useState<string>(
    signingKeys[0]?.fingerprint ?? '',
  );
  const [loading, setLoading] = useState(false);
  const [sigPath, setSigPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const title = theme === 'ussr' ? 'Sign for the Party' : 'Sign File';

  async function handlePickFile(): Promise<void> {
    const picked = await open({ multiple: false });
    if (picked === null) return;
    setFile(Array.isArray(picked) ? picked[0] ?? null : picked);
    setSigPath(null);
    setError(null);
  }

  function handleSign(): void {
    setError(null);
    setSigPath(null);

    if (file === null) {
      setError('Select a file to sign first.');
      return;
    }
    if (signerFp === '') {
      setError('No signing-capable key available.');
      return;
    }

    setLoading(true);
    signFile(file, signerFp)
      .then((path) => {
        setSigPath(path);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <div className={styles.cardContent}>
        <h1 className={styles.title}>{title}</h1>

        {error !== null && (
          <Alert variant="error" message={error} dismissible />
        )}

        {sigPath !== null && (
          <Alert
            variant="success"
            title="Signature created"
            message={sigPath}
          />
        )}

        {/* File selection */}
        <section>
          <p className={styles.sectionLabel}>File to sign</p>
          <div className={styles.fileRow}>
            <Button
              variant="ghost"
              size="md"
              onClick={() => { void handlePickFile(); }}
              disabled={loading}
            >
              Select file
            </Button>
            {file !== null && (
              <span className={styles.fileName}>{basename(file)}</span>
            )}
          </div>
        </section>

        {/* Signing key */}
        <section>
          <p className={styles.sectionLabel}>Signing key</p>
          {signingKeys.length === 0 ? (
            <Alert
              variant="warning"
              message="No signing-capable private key found in your keyring."
            />
          ) : (
            <div className={styles.selectWrapper}>
              <select
                className={styles.select}
                value={signerFp}
                onChange={(e) => setSignerFp(e.currentTarget.value)}
                disabled={loading}
                aria-label="Signing key"
              >
                {signingKeys.map((k) => (
                  <option key={k.fingerprint} value={k.fingerprint}>
                    {k.name} &lt;{k.email}&gt;
                  </option>
                ))}
              </select>
            </div>
          )}
        </section>

        {/* Actions */}
        <div className={styles.actions}>
          <Button
            variant="primary"
            size="md"
            loading={loading}
            disabled={signingKeys.length === 0}
            onClick={handleSign}
          >
            {loading ? 'Signing…' : 'Sign'}
          </Button>
        </div>

        </div>
        <UssrBanner n={20} variant="fill" />
      </div>
    </div>
  );
}
