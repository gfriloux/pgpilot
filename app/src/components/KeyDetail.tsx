import { useState, useEffect } from 'react';
import type { KeyInfo, SubkeyInfo, TrustLevel } from '../types/ipc';
import { formatDate, formatFp } from '../utils/format';
import {
  exportPublicKeyArmored,
  deleteKey,
  publishKey,
  moveToCard,
  setKeyTrust,
  checkKeyserver,
  checkRevocationCert,
  generateRevocationCert,
} from '../ipc/keys';
import { useUiStore } from '../store/ui';
import { Button } from './Button';
import { Modal } from './Modal';
import { Tooltip } from './Tooltip';
import { Select } from './Select';
import { SubkeyCard } from './SubkeyCard';
import { UssrBanner } from './UssrBanner';
import styles from './KeyDetail.module.css';

interface KeyDetailProps {
  keyInfo: KeyInfo;
  /** Whether the key is known to be published on a keyserver */
  published?: boolean;
  /** USSR banner number shown at the bottom of the detail panel */
  bannerN?: 12 | 16 | 17 | 18 | 19 | 20 | 23 | 24 | 25 | 26 | 27 | 29;
  /** Whether a YubiKey/smartcard is currently connected */
  cardConnected?: boolean;
  onBackup: (fp: string) => void;
  /** Called after a successful delete so the parent can navigate away */
  onAfterDelete: () => void;
  /** Called after any mutation so the parent can reload the key list */
  onReload: () => void;
}

type ActiveModal =
  | { kind: 'delete' }
  | { kind: 'publish'; keyserver: string }
  | { kind: 'migrate' };

const KEYSERVER_OPTIONS = [
  { value: 'https://keys.openpgp.org', label: 'keys.openpgp.org' },
  { value: 'https://keyserver.ubuntu.com', label: 'keyserver.ubuntu.com' },
];

function daysUntil(dateStr: string): number {
  const expiry = new Date(dateStr).getTime();
  return Math.floor((expiry - Date.now()) / (1000 * 60 * 60 * 24));
}

function ExpiryValue({ expires }: { expires: string | null }) {
  if (expires === null) {
    return <span className={styles.infoValueMuted}>No expiry</span>;
  }
  const days = daysUntil(expires);
  if (days < 0) {
    return (
      <span className={styles.expiryExpired}>
        {formatDate(expires)} (expired)
      </span>
    );
  }
  if (days < 90) {
    return (
      <span className={styles.expirySoon}>
        {formatDate(expires)} ({days}d left)
      </span>
    );
  }
  return <span className={styles.infoValue}>{formatDate(expires)}</span>;
}

function TrustValue({ trust }: { trust: TrustLevel }) {
  const labelMap: Record<TrustLevel, string> = {
    undefined: 'Undefined',
    marginal: 'Marginal',
    full: 'Full',
    ultimate: 'Ultimate',
  };
  const classMap: Record<TrustLevel, string> = {
    undefined: styles.trustUndefined ?? '',
    marginal: styles.trustMarginal ?? '',
    full: styles.trustFull ?? '',
    ultimate: styles.trustUltimate ?? '',
  };
  return <span className={classMap[trust]}>{labelMap[trust] ?? trust}</span>;
}

function findSubkey(subkeys: SubkeyInfo[], usage: string): SubkeyInfo | undefined {
  return subkeys.find((sk) => sk.usage === usage);
}

export function KeyDetail({
  keyInfo,
  published: _published,
  bannerN,
  cardConnected = false,
  onBackup,
  onAfterDelete,
  onReload,
}: KeyDetailProps) {
  const fp = keyInfo.fingerprint;
  const setStatus = useUiStore((s) => s.setStatus);

  // Keyserver publication status
  const [ksPublished, setKsPublished] = useState<boolean | null>(null);

  // Revocation certificate
  const [revCertPath, setRevCertPath] = useState<string | null | undefined>(undefined);
  const [revCertGenerating, setRevCertGenerating] = useState(false);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setKsPublished(null);
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setRevCertPath(undefined);

    checkKeyserver(fp)
      .then(setKsPublished)
      .catch(() => { setKsPublished(null); });

    if (keyInfo.has_secret) {
      checkRevocationCert(fp)
        .then(setRevCertPath)
        .catch(() => { setRevCertPath(null); });
    }
  }, [fp, keyInfo.has_secret]);

  function handleGenerateRevCert(): void {
    setRevCertGenerating(true);
    generateRevocationCert(fp)
      .then((path) => {
        setRevCertPath(path);
        setStatus('success', 'Revocation certificate generated.');
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Failed: ${msg}`);
      })
      .finally(() => { setRevCertGenerating(false); });
  }

  function handleCopyRevPath(): void {
    if (revCertPath) {
      void navigator.clipboard.writeText(revCertPath)
        .then(() => { setStatus('success', 'Path copied.'); });
    }
  }

  const [modal, setModal] = useState<ActiveModal | null>(null);
  const [actionLoading, setActionLoading] = useState(false);
  const [publishKeyserver, setPublishKeyserver] = useState('https://keys.openpgp.org');
  const [trust, setTrust] = useState<'undefined' | 'marginal' | 'full'>(
    keyInfo.trust === 'ultimate' ? 'full' : (keyInfo.trust as 'undefined' | 'marginal' | 'full'),
  );

  // ── Export public key → clipboard ────────────────────────────────
  function handleExportPublic(): void {
    exportPublicKeyArmored(fp)
      .then((armored) =>
        navigator.clipboard.writeText(armored).then(() => {
          setStatus('success', 'Public key copied to clipboard.');
        }),
      )
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Export failed: ${msg}`);
      });
  }

  // ── Delete key ────────────────────────────────────────────────────
  function handleDeleteConfirm(): void {
    setActionLoading(true);
    deleteKey(fp, keyInfo.has_secret)
      .then(() => {
        setModal(null);
        setStatus('success', `Key ${fp.slice(0, 8)}… deleted.`);
        onReload();
        onAfterDelete();
      })
      .catch((err: unknown) => {
        setModal(null);
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Delete failed: ${msg}`);
      })
      .finally(() => {
        setActionLoading(false);
      });
  }

  // ── Publish key ───────────────────────────────────────────────────
  function handlePublishConfirm(): void {
    setActionLoading(true);
    publishKey(fp, publishKeyserver)
      .then(() => {
        setModal(null);
        setStatus('success', 'Key published to keyserver.');
        onReload();
      })
      .catch((err: unknown) => {
        setModal(null);
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Publish failed: ${msg}`);
      })
      .finally(() => {
        setActionLoading(false);
      });
  }

  // ── Migrate to YubiKey ────────────────────────────────────────────
  function handleMigrateConfirm(): void {
    setActionLoading(true);
    moveToCard(fp)
      .then(() => {
        setModal(null);
        setStatus('success', 'Key migrated to YubiKey.');
        onReload();
      })
      .catch((err: unknown) => {
        setModal(null);
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Migration failed: ${msg}`);
      })
      .finally(() => {
        setActionLoading(false);
      });
  }

  // ── Trust ─────────────────────────────────────────────────────────
  function handleTrustChange(level: 'undefined' | 'marginal' | 'full'): void {
    setTrust(level);
    setKeyTrust(fp, level)
      .then(() => {
        setStatus('success', `Trust set to ${level}.`);
        onReload();
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Trust change failed: ${msg}`);
      });
  }

  return (
    <>
      <div className={styles.detail}>
        <div className={styles.columns}>
          {/* ── Left column ────────────────────────────────────────── */}
          <div className={styles.leftCol}>
            <div className={styles.leftColContent}>
            <h2 className={styles.keyName}>{keyInfo.name}</h2>
            <p className={styles.keyEmail}>{keyInfo.email}</p>

            <div className={styles.infoRow}>
              <span className={styles.infoLabel}>Key ID</span>
              <span className={styles.infoValueMono}>{keyInfo.key_id}</span>
            </div>

            <div className={styles.infoRow}>
              <span className={styles.infoLabel}>Fingerprint</span>
              <span className={styles.infoValueMono}>{formatFp(fp)}</span>
            </div>

            <div className={styles.infoRow}>
              <span className={styles.infoLabel}>Algorithm</span>
              <span className={styles.infoValue}>{keyInfo.algo}</span>
            </div>

            <div className={styles.infoRow}>
              <span className={styles.infoLabel}>Created</span>
              <span className={styles.infoValue}>{formatDate(keyInfo.created)}</span>
            </div>

            <div className={styles.infoRow}>
              <span className={styles.infoLabel}>Expires</span>
              <ExpiryValue expires={keyInfo.expires} />
            </div>

            {keyInfo.on_card && (
              <div className={styles.infoRow}>
                <span className={styles.infoLabel}>Card</span>
                <span className={styles.cardRow}>
                  &#9672; YubiKey
                  {keyInfo.card_serial !== null && (
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.7rem' }}>
                      &nbsp;{keyInfo.card_serial}
                    </span>
                  )}
                </span>
              </div>
            )}

            <div className={styles.infoRow}>
              <span className={styles.infoLabel}>Trust</span>
              <TrustValue trust={keyInfo.trust} />
            </div>

            {/* Keyserver publication status */}
            {ksPublished === true ? (
              <div className={styles.keyserverPublishedRow}>
                <span className={styles.keyserverPublished}>&#9733; Published on keys.openpgp.org</span>
                <button
                  className={styles.ksLinkBtn}
                  onClick={() => {
                    window.open(`https://keys.openpgp.org/search?q=${fp.toUpperCase()}`, '_blank');
                  }}
                  title="Open on keys.openpgp.org"
                >
                  Link &#8599;
                </button>
              </div>
            ) : ksPublished === false ? (
              <div className={styles.keyserverUnpublishedRow}>
                <span className={styles.keyserverUnpublished}>&#9675; Not published on keyserver</span>
              </div>
            ) : null}

            {/* Trust picker — only for public keys (no secret) */}
            {!keyInfo.has_secret && (
              <div className={styles.trustPicker}>
                <span className={styles.infoLabel}>Set trust</span>
                <div className={styles.trustRadioGroup} role="radiogroup" aria-label="Trust level">
                  {(['undefined', 'marginal', 'full'] as const).map((lvl) => (
                    <label key={lvl} className={styles.trustRadioLabel}>
                      <input
                        type="radio"
                        name={`trust-${fp}`}
                        value={lvl}
                        checked={trust === lvl}
                        onChange={() => { handleTrustChange(lvl); }}
                        className={styles.trustRadioInput}
                      />
                      {lvl.charAt(0).toUpperCase() + lvl.slice(1)}
                    </label>
                  ))}
                </div>
              </div>
            )}

            <div className={styles.starSeparator} aria-hidden="true">&#9733;</div>

            {/* Action row 1 */}
            <div className={styles.actionRow}>
              <Button
                variant="primary"
                size="sm"
                onClick={handleExportPublic}
              >
                Export public
              </Button>
              {keyInfo.has_secret && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => { onBackup(fp); }}
                >
                  Backup
                </Button>
              )}
            </div>

            {/* Action row 2 */}
            <div className={styles.actionRow} style={{ marginTop: '8px' }}>
              {keyInfo.has_secret && !keyInfo.on_card && (
                <Tooltip
                  content={cardConnected ? 'Move subkeys to YubiKey (irreversible)' : 'No YubiKey connected'}
                  placement="top"
                >
                  <Button
                    variant="primary"
                    size="sm"
                    disabled={!cardConnected}
                    onClick={() => { setModal({ kind: 'migrate' }); }}
                  >
                    Migrate to YubiKey
                  </Button>
                </Tooltip>
              )}
              {ksPublished === false && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => { setModal({ kind: 'publish', keyserver: publishKeyserver }); }}
                >
                  Publish
                </Button>
              )}
              <Button
                variant="destructive"
                size="sm"
                onClick={() => { setModal({ kind: 'delete' }); }}
              >
                Delete
              </Button>
            </div>

            {/* ── Star separator ──────────────────────────────── */}
            {keyInfo.has_secret && (
              <div className={styles.starSeparator} aria-hidden="true">&#9733;</div>
            )}

            {/* ── Revocation certificate ──────────────────────── */}
            {keyInfo.has_secret && (
              <div className={styles.revCertSection}>
                <span className={styles.revCertTitle}>Revocation Certificate</span>
                {revCertPath === undefined ? (
                  <span className={styles.infoValueMuted}>Checking…</span>
                ) : revCertPath !== null ? (
                  <>
                    <span className={styles.revCertFound}>&#10003; Certificate found</span>
                    <code className={styles.revCertPath}>{revCertPath}</code>
                    <div className={styles.revCertActions}>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={handleCopyRevPath}
                      >
                        Copy path
                      </Button>
                    </div>
                  </>
                ) : (
                  <>
                    <span className={styles.revCertMissing}>Certificate not found</span>
                    <Button
                      variant="ghost"
                      size="sm"
                      loading={revCertGenerating}
                      onClick={handleGenerateRevCert}
                    >
                      Generate
                    </Button>
                  </>
                )}
              </div>
            )}

            {/* ── Star separator ──────────────────────────────── */}
            {keyInfo.has_secret && (
              <div className={styles.starSeparator} aria-hidden="true">&#9733;</div>
            )}

            </div>

            {bannerN !== undefined && (
              <div className={styles.bannerRow}>
                <UssrBanner n={bannerN} />
              </div>
            )}
          </div>

          {/* ── Right column — subkeys ──────────────────────────────── */}
          <div className={styles.rightCol}>
            <span className={styles.subkeysTitle}>Subkeys</span>
            <div className={styles.subkeysStack}>
              <SubkeyCard
                usage="S"
                subkey={findSubkey(keyInfo.subkeys, 'S')}
                keyFp={fp}
                canEdit={keyInfo.has_secret}
                onReload={onReload}
              />
              <SubkeyCard
                usage="E"
                subkey={findSubkey(keyInfo.subkeys, 'E')}
                keyFp={fp}
                canEdit={keyInfo.has_secret}
                onReload={onReload}
              />
              <SubkeyCard
                usage="A"
                subkey={findSubkey(keyInfo.subkeys, 'A')}
                keyFp={fp}
                canEdit={keyInfo.has_secret}
                onReload={onReload}
              />
            </div>
          </div>
        </div>

      </div>

      {/* ── Delete modal ────────────────────────────────────────────── */}
      {modal?.kind === 'delete' && (
        <Modal title="Delete key" onClose={() => { setModal(null); }}>
          <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
            Delete <strong style={{ color: 'var(--text-strong)' }}>{keyInfo.name}</strong> (
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem' }}>
              {fp.slice(0, 16)}…
            </span>
            )?{keyInfo.has_secret && ' This will remove the secret key.'}
            {' '}This action is irreversible.
          </p>
          <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
            <Button variant="ghost" size="sm" onClick={() => { setModal(null); }}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              size="sm"
              loading={actionLoading}
              onClick={handleDeleteConfirm}
            >
              Delete
            </Button>
          </div>
        </Modal>
      )}

      {/* ── Publish modal ────────────────────────────────────────────── */}
      {modal?.kind === 'publish' && (
        <Modal title="Publish to keyserver" onClose={() => { setModal(null); }}>
          <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
            Publish <strong style={{ color: 'var(--text-strong)' }}>{keyInfo.name}</strong> to a
            public keyserver. The public key will be permanently searchable by email and fingerprint.
          </p>
          <Select
            label="Keyserver"
            options={KEYSERVER_OPTIONS}
            value={publishKeyserver}
            onChange={setPublishKeyserver}
          />
          <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end', marginTop: '16px' }}>
            <Button variant="ghost" size="sm" onClick={() => { setModal(null); }}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              loading={actionLoading}
              onClick={handlePublishConfirm}
            >
              Publish
            </Button>
          </div>
        </Modal>
      )}

      {/* ── Migrate modal ────────────────────────────────────────────── */}
      {modal?.kind === 'migrate' && (
        <Modal title="Migrate to YubiKey" onClose={() => { setModal(null); }}>
          <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
            Move the secret key material for{' '}
            <strong style={{ color: 'var(--text-strong)' }}>{keyInfo.name}</strong> to your YubiKey.
            This operation is <strong style={{ color: 'var(--error)' }}>irreversible</strong> — the
            secret key will only exist on the card afterwards. Make sure you have a backup first.
          </p>
          <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
            <Button variant="ghost" size="sm" onClick={() => { setModal(null); }}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              loading={actionLoading}
              onClick={handleMigrateConfirm}
            >
              Migrate
            </Button>
          </div>
        </Modal>
      )}
    </>
  );
}
