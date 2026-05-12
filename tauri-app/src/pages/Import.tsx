import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { importKeyText, importKeyUrl, importKeyKeyserver } from '../ipc/keys';
import { useKeys } from '../hooks/useKeys';
import { useUiStore } from '../store/ui';
import { Tabs } from '../components/Tabs';
import { Input } from '../components/Input';
import { Select } from '../components/Select';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import styles from './Import.module.css';

const KEYSERVER_OPTIONS = [
  { value: 'https://keys.openpgp.org', label: 'keys.openpgp.org' },
  { value: 'https://keyserver.ubuntu.com', label: 'keyserver.ubuntu.com' },
];

const TABS = [
  { id: 'paste', label: 'Paste' },
  { id: 'url', label: 'URL' },
  { id: 'keyserver', label: 'Keyserver' },
  { id: 'file', label: 'File' },
];

export default function Import() {
  const navigate = useNavigate();
  const { reload } = useKeys();
  const setStatus = useUiStore((s) => s.setStatus);

  const [activeTab, setActiveTab] = useState('paste');
  const [loading, setLoading] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

  // Paste tab
  const [pasteContent, setPasteContent] = useState('');

  // URL tab
  const [urlValue, setUrlValue] = useState('');
  const [urlError, setUrlError] = useState<string | undefined>(undefined);

  // Keyserver tab
  const [ksQuery, setKsQuery] = useState('');
  const [ksUrl, setKsUrl] = useState('https://keys.openpgp.org');
  const [ksQueryError, setKsQueryError] = useState<string | undefined>(undefined);

  function afterSuccess(label: string): void {
    reload();
    setStatus('success', `Key imported from ${label}.`);
    void navigate('/public-keys');
  }

  function wrapImport(op: Promise<void>, label: string): void {
    setImportError(null);
    setLoading(true);
    op
      .then(() => { afterSuccess(label); })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        setImportError(msg);
      })
      .finally(() => {
        setLoading(false);
      });
  }

  // ── Paste ─────────────────────────────────────────────────────────
  function handlePasteImport(): void {
    if (pasteContent.trim().length === 0) {
      setImportError('Paste an armored PGP public key block first.');
      return;
    }
    wrapImport(importKeyText(pasteContent.trim()), 'pasted text');
  }

  // ── URL ───────────────────────────────────────────────────────────
  function handleUrlImport(): void {
    if (urlValue.trim().length === 0) {
      setUrlError('Enter a URL.');
      return;
    }
    setUrlError(undefined);
    wrapImport(importKeyUrl(urlValue.trim()), 'URL');
  }

  // ── Keyserver ─────────────────────────────────────────────────────
  function handleKsImport(): void {
    if (ksQuery.trim().length === 0) {
      setKsQueryError('Enter a fingerprint, key ID, or email address.');
      return;
    }
    setKsQueryError(undefined);
    wrapImport(importKeyKeyserver(ksQuery.trim(), ksUrl), 'keyserver');
  }

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <h1 className={styles.title}>Import Key</h1>

        {importError !== null && (
          <Alert variant="error" message={importError} dismissible />
        )}

        <Tabs tabs={TABS} activeTab={activeTab} onChange={(id) => {
          setActiveTab(id);
          setImportError(null);
        }} />

        {/* ── Paste tab ──────────────────────────────────────────── */}
        {activeTab === 'paste' && (
          <div className={styles.tabPanel} role="tabpanel">
            <label className={styles.textareaLabel} htmlFor="paste-input">
              Armored PGP public key
            </label>
            <textarea
              id="paste-input"
              className={styles.textarea}
              value={pasteContent}
              onChange={(e) => { setPasteContent(e.currentTarget.value); }}
              placeholder={'-----BEGIN PGP PUBLIC KEY BLOCK-----\n…\n-----END PGP PUBLIC KEY BLOCK-----'}
              rows={10}
              disabled={loading}
              spellCheck={false}
            />
            <div className={styles.actions}>
              <Button variant="ghost" size="md" onClick={() => { void navigate('/public-keys'); }} disabled={loading}>
                Cancel
              </Button>
              <Button variant="primary" size="md" loading={loading} onClick={handlePasteImport}>
                Import
              </Button>
            </div>
          </div>
        )}

        {/* ── URL tab ────────────────────────────────────────────── */}
        {activeTab === 'url' && (
          <div className={styles.tabPanel} role="tabpanel">
            <Input
              label="Key URL"
              type="text"
              value={urlValue}
              onChange={(e) => { setUrlValue(e.currentTarget.value); }}
              {...(urlError !== undefined ? { error: urlError } : {})}
              placeholder="https://example.com/alice.asc"
              disabled={loading}
              autoFocus
            />
            <div className={styles.actions}>
              <Button variant="ghost" size="md" onClick={() => { void navigate('/public-keys'); }} disabled={loading}>
                Cancel
              </Button>
              <Button variant="primary" size="md" loading={loading} onClick={handleUrlImport}>
                Import
              </Button>
            </div>
          </div>
        )}

        {/* ── Keyserver tab ──────────────────────────────────────── */}
        {activeTab === 'keyserver' && (
          <div className={styles.tabPanel} role="tabpanel">
            <Input
              label="Search query"
              type="text"
              value={ksQuery}
              onChange={(e) => { setKsQuery(e.currentTarget.value); }}
              {...(ksQueryError !== undefined ? { error: ksQueryError } : {})}
              placeholder="alice@example.com or fingerprint or key ID"
              hint="Accepts email address, 40-char fingerprint, or long key ID"
              disabled={loading}
              autoFocus
            />
            <Select
              label="Keyserver"
              options={KEYSERVER_OPTIONS}
              value={ksUrl}
              onChange={setKsUrl}
              disabled={loading}
            />
            <div className={styles.actions}>
              <Button variant="ghost" size="md" onClick={() => { void navigate('/public-keys'); }} disabled={loading}>
                Cancel
              </Button>
              <Button variant="primary" size="md" loading={loading} onClick={handleKsImport}>
                Search &amp; Import
              </Button>
            </div>
          </div>
        )}

        {/* ── File tab ───────────────────────────────────────────── */}
        {activeTab === 'file' && (
          <div className={styles.tabPanel} role="tabpanel">
            <div className={styles.fileNotice} role="status">
              <span className={styles.fileNoticeIcon} aria-hidden="true">i</span>
              <p>
                File selection is not available in this version — the native file dialog requires
                additional Tauri integration. Use the <strong>Paste</strong> or{' '}
                <strong>Keyserver</strong> tabs to import a key.
              </p>
            </div>
            <div className={styles.actions}>
              <Button variant="ghost" size="md" onClick={() => { void navigate('/public-keys'); }}>
                Cancel
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
