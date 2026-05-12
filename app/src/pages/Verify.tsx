import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useConfigStore } from '../store/config';
import { verifySignatureCmd } from '../ipc/keys';
import type { VerifyResult, VerifyOutcome } from '../types/ipc';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import { UssrBanner } from '../components/UssrBanner';
import styles from './Verify.module.css';

function basename(p: string): string {
  return p.split('/').pop() ?? p;
}

interface OutcomeDisplay {
  cssClass: string;
  icon: string;
  label: string;
}

const CSS_VALID   = styles.resultValid   ?? '';
const CSS_ERROR   = styles.resultError   ?? '';
const CSS_WARNING = styles.resultWarning ?? '';

function outcomeDisplay(outcome: VerifyOutcome): OutcomeDisplay {
  if (outcome === 'valid') {
    return { cssClass: CSS_VALID, icon: '✓', label: 'Valid signature' };
  }
  if (outcome === 'bad_sig') {
    return {
      cssClass: CSS_ERROR,
      icon: '✕',
      label: 'Invalid signature — file has been modified',
    };
  }
  if (outcome === 'unknown_key') {
    return {
      cssClass: CSS_WARNING,
      icon: '⚠',
      label: 'Unknown key — import the signer\'s key first',
    };
  }
  if (outcome === 'expired_key') {
    return {
      cssClass: CSS_WARNING,
      icon: '⚠',
      label: 'Expired key at time of signature',
    };
  }
  if (outcome === 'revoked_key') {
    return {
      cssClass: CSS_ERROR,
      icon: '✕',
      label: 'Revoked key',
    };
  }
  // { error: string }
  return {
    cssClass: CSS_ERROR,
    icon: '✕',
    label: `GPG error: ${outcome.error}`,
  };
}

export default function Verify() {
  const theme = useConfigStore((s) => s.theme);

  const [file, setFile] = useState<string | null>(null);
  const [sigFile, setSigFile] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<VerifyResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const title = theme === 'ussr' ? 'Verify for the Commissariat' : 'Verify Signature';

  async function handlePickFile(): Promise<void> {
    const picked = await open({ multiple: false });
    if (picked === null) return;
    setFile(Array.isArray(picked) ? (picked[0] ?? null) : picked);
    setResult(null);
    setError(null);
  }

  async function handlePickSig(): Promise<void> {
    const picked = await open({
      multiple: false,
      filters: [{ name: 'Signature', extensions: ['sig'] }],
    });
    if (picked === null) return;
    setSigFile(Array.isArray(picked) ? (picked[0] ?? null) : picked);
    setResult(null);
    setError(null);
  }

  function handleVerify(): void {
    setError(null);
    setResult(null);

    if (file === null) {
      setError('Select the file to verify first.');
      return;
    }

    setLoading(true);
    verifySignatureCmd(file, sigFile)
      .then((r) => {
        setResult(r);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }

  const display = result !== null ? outcomeDisplay(result.outcome) : null;

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <div className={styles.cardContent}>
        <h1 className={styles.title}>{title}</h1>

        {error !== null && (
          <Alert variant="error" message={error} dismissible />
        )}

        {/* File to verify */}
        <section>
          <p className={styles.sectionLabel}>File to verify</p>
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

        {/* Signature file (optional) */}
        <section>
          <p className={styles.sectionLabel}>
            Signature file{' '}
            <span className={styles.fileOptional}>(optional &mdash; auto-detected)</span>
          </p>
          <div className={styles.fileRow}>
            <Button
              variant="ghost"
              size="md"
              onClick={() => { void handlePickSig(); }}
              disabled={loading}
            >
              Select .sig
            </Button>
            {sigFile !== null && (
              <span className={styles.fileName}>{basename(sigFile)}</span>
            )}
          </div>
        </section>

        {/* Actions */}
        <div className={styles.actions}>
          <Button
            variant="primary"
            size="md"
            loading={loading}
            onClick={handleVerify}
          >
            {loading ? 'Verifying…' : 'Verify'}
          </Button>
        </div>

        {/* Result */}
        {result !== null && display !== null && (
          <div className={[styles.result, display.cssClass].join(' ')}>
            <div className={styles.resultHeader}>
              <span className={styles.resultIcon} aria-hidden="true">
                {display.icon}
              </span>
              <span>{display.label}</span>
            </div>

            {(result.signer_name !== null ||
              result.signer_fp !== null ||
              result.signed_at !== null) && (
              <div className={styles.resultMeta}>
                {result.signer_name !== null && (
                  <div className={styles.resultMetaRow}>
                    <span className={styles.metaLabel}>Signer:</span>
                    <span>{result.signer_name}</span>
                  </div>
                )}
                {result.signer_fp !== null && (
                  <div className={styles.resultMetaRow}>
                    <span className={styles.metaLabel}>Fingerprint:</span>
                    <span className={styles.metaMono}>{result.signer_fp}</span>
                  </div>
                )}
                {result.signed_at !== null && (
                  <div className={styles.resultMetaRow}>
                    <span className={styles.metaLabel}>Signed at:</span>
                    <span>{result.signed_at}</span>
                  </div>
                )}
                <div className={styles.resultMetaRow}>
                  <span className={styles.metaLabel}>Trust:</span>
                  <span>{result.signer_trust}</span>
                </div>
              </div>
            )}
          </div>
        )}

        </div>
        <UssrBanner n={17} variant="fill" />
      </div>
    </div>
  );
}
