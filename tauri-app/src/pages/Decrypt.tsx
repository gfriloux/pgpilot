import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { decryptFiles } from '../ipc/keys';
import { useUiStore } from '../store/ui';
import { UssrBanner } from '../components/UssrBanner';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import styles from './Decrypt.module.css';

export default function Decrypt() {
  const [files, setFiles] = useState<string[]>([]);
  const [results, setResults] = useState<string[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const setStatus = useUiStore((s) => s.setStatus);

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  async function handlePickFiles() {
    const selected = await open({ multiple: true });
    if (selected === null) return;
    setFiles(Array.isArray(selected) ? selected : [selected]);
    setResults(null);
    setError(null);
  }

  async function handleDecrypt() {
    if (files.length === 0) return;
    setLoading(true);
    setError(null);
    setResults(null);
    try {
      const msgs = await decryptFiles(files);
      setResults(msgs);
      setStatus('success', `${files.length} file(s) decrypted.`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      setStatus('error', `Decryption failed: ${msg}`);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <div className={styles.cardContent}>
        <h1 className={styles.title}>Decrypt</h1>
        <p className={styles.subtitle}>
          Decrypt files encrypted for your PGP key.
        </p>

        <div className={styles.section}>
          <Button variant="ghost" onClick={() => { void handlePickFiles(); }}>
            {files.length > 0 ? 'Change files…' : 'Select files…'}
          </Button>

          {files.length > 0 && (
            <ul className={styles.fileList}>
              {files.map((f) => (
                <li key={f} className={styles.fileItem}>
                  {baseName(f)}
                </li>
              ))}
            </ul>
          )}
        </div>

        <Button
          variant="primary"
          disabled={files.length === 0}
          loading={loading}
          onClick={() => { void handleDecrypt(); }}
        >
          Decrypt
        </Button>

        {error !== null && (
          <Alert variant="error" title="Decryption failed" message={error} />
        )}

        {results !== null && (
          <Alert
            variant="success"
            title="Done"
            message={results.join('\n') || 'Files decrypted successfully.'}
          />
        )}

        </div>
        <UssrBanner n={19} variant="fill" />
      </div>
    </div>
  );
}
