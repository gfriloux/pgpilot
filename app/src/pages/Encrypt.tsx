import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useKeysStore } from '../store/keys';
import { useConfigStore } from '../store/config';
import { encryptFiles } from '../ipc/keys';
import type { KeyInfo } from '../types/ipc';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import { Modal } from '../components/Modal';
import { UssrBanner } from '../components/UssrBanner';
import styles from './Encrypt.module.css';

function basename(p: string): string {
  return p.split('/').pop() ?? p;
}

function isTrusted(key: KeyInfo): boolean {
  return key.trust === 'full' || key.trust === 'ultimate';
}

export default function Encrypt() {
  const keys = useKeysStore((s) => s.keys);
  const theme = useConfigStore((s) => s.theme);

  const [files, setFiles] = useState<string[]>([]);
  const [selectedFps, setSelectedFps] = useState<Set<string>>(new Set());
  const [armor, setArmor] = useState(false);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showTrustModal, setShowTrustModal] = useState(false);

  const title =
    theme === 'ussr' ? 'Encrypt for the State' : 'Encrypt Files';

  async function handlePickFiles(): Promise<void> {
    const picked = await open({ multiple: true });
    if (picked === null) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    setFiles((prev) => {
      const merged = [...prev];
      for (const p of paths) {
        if (!merged.includes(p)) merged.push(p);
      }
      return merged;
    });
  }

  function removeFile(path: string): void {
    setFiles((prev) => prev.filter((f) => f !== path));
  }

  function toggleRecipient(fp: string): void {
    setSelectedFps((prev) => {
      const next = new Set(prev);
      if (next.has(fp)) {
        next.delete(fp);
      } else {
        next.add(fp);
      }
      return next;
    });
  }

  function getUntrustedCount(): number {
    return [...selectedFps].filter((fp) => {
      const k = keys.find((x) => x.fingerprint === fp);
      return k !== undefined && !isTrusted(k);
    }).length;
  }

  function handleEncryptClick(): void {
    setError(null);
    setResult(null);
    if (files.length === 0) {
      setError('Select at least one file to encrypt.');
      return;
    }
    if (selectedFps.size === 0) {
      setError('Select at least one recipient.');
      return;
    }
    const untrusted = getUntrustedCount();
    if (untrusted > 0) {
      setShowTrustModal(true);
    } else {
      runEncrypt(false);
    }
  }

  function runEncrypt(forceTrust: boolean): void {
    setShowTrustModal(false);
    setLoading(true);
    encryptFiles(files, [...selectedFps], armor, forceTrust)
      .then((paths) => {
        setResult(paths);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }

  const untrustedCount = getUntrustedCount();

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <div className={styles.cardContent}>
        <h1 className={styles.title}>{title}</h1>

        {error !== null && (
          <Alert variant="error" message={error} dismissible />
        )}

        {result !== null && (
          <div>
            <Alert
              variant="success"
              title="Encryption complete"
              message={`${result.length} file${result.length !== 1 ? 's' : ''} encrypted`}
              dismissible
            />
            <ul className={styles.resultList}>
              {result.map((p) => (
                <li key={p}>{p}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Files */}
        <section>
          <p className={styles.sectionLabel}>Files</p>
          <div className={styles.fileRow}>
            <Button variant="ghost" size="md" onClick={() => { void handlePickFiles(); }} disabled={loading}>
              Select files
            </Button>
            {files.length > 0 && (
              <span style={{ fontSize: '0.8125rem', color: 'var(--text-muted)' }}>
                {files.length} file{files.length !== 1 ? 's' : ''} selected
              </span>
            )}
          </div>
          {files.length > 0 && (
            <div className={styles.fileList}>
              {files.map((f) => (
                <div key={f} className={styles.fileItem}>
                  <span>{basename(f)}</span>
                  <button
                    type="button"
                    onClick={() => removeFile(f)}
                    aria-label={`Remove ${basename(f)}`}
                    disabled={loading}
                  >
                    &times;
                  </button>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Recipients */}
        <section>
          <p className={styles.sectionLabel}>
            Recipients ({selectedFps.size} selected)
          </p>
          <div className={styles.chipGrid}>
            {keys.map((key) => {
              const selected = selectedFps.has(key.fingerprint);
              const trusted = isTrusted(key);
              return (
                <button
                  key={key.fingerprint}
                  type="button"
                  className={[styles.chip, selected ? styles.chipSelected : ''].join(' ')}
                  onClick={() => toggleRecipient(key.fingerprint)}
                  disabled={loading}
                  aria-pressed={selected}
                >
                  <div className={styles.chipNames}>
                    <div className={styles.chipName}>{key.name}</div>
                    <div className={styles.chipEmail}>{key.email}</div>
                  </div>
                  <span
                    className={[
                      styles.chipTrust,
                      trusted ? styles.chipTrustOk : styles.chipTrustWarn,
                    ].join(' ')}
                    title={trusted ? 'Trusted' : 'Untrusted'}
                    aria-label={trusted ? 'trusted' : 'untrusted'}
                  >
                    {trusted ? '✓' : '⚠'}
                  </span>
                </button>
              );
            })}
          </div>
        </section>

        {/* Format */}
        <section>
          <p className={styles.sectionLabel}>Output format</p>
          <div className={styles.formatRow}>
            <Button
              variant={!armor ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setArmor(false)}
              disabled={loading}
            >
              .gpg (binary)
            </Button>
            <Button
              variant={armor ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setArmor(true)}
              disabled={loading}
            >
              .asc (armored)
            </Button>
          </div>
        </section>

        {/* Actions */}
        <div className={styles.actions}>
          <Button
            variant="primary"
            size="md"
            loading={loading}
            onClick={handleEncryptClick}
          >
            {loading ? 'Encrypting…' : 'Encrypt'}
          </Button>
        </div>

        </div>
        <UssrBanner n={16} variant="fill" />
      </div>

      {/* Trust warning modal */}
      {showTrustModal && (
        <Modal
          title="Untrusted recipients"
          onClose={() => setShowTrustModal(false)}
        >
          <div className={styles.modalBody}>
            <p>
              {untrustedCount} recipient{untrustedCount !== 1 ? 's' : ''} have
              insufficient trust level. Encrypting to untrusted keys may be
              unsafe. Proceed anyway?
            </p>
            <div className={styles.modalActions}>
              <Button
                variant="ghost"
                size="md"
                onClick={() => setShowTrustModal(false)}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                size="md"
                onClick={() => runEncrypt(true)}
              >
                Encrypt anyway
              </Button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}
