import { useState } from 'react';
import { Copy } from 'lucide-react';
import type { SubkeyInfo } from '../types/ipc';
import { formatDate } from '../utils/format';
import { renewSubkeyCmd, addSubkeyCmd } from '../ipc/keys';
import { useUiStore } from '../store/ui';
import { Badge } from './Badge';
import { Button } from './Button';
import { Select } from './Select';
import styles from './SubkeyCard.module.css';

type SubkeyUsage = 'S' | 'E' | 'A';

interface SubkeyCardProps {
  usage: SubkeyUsage;
  subkey: SubkeyInfo | undefined;
  keyFp: string;
  canEdit: boolean;
  onReload: () => void;
}

const USAGE_LABELS: Record<SubkeyUsage, string> = {
  S: 'Signature',
  E: 'Encryption',
  A: 'Auth SSH',
};

const USAGE_FULL: Record<SubkeyUsage, string> = {
  S: 'sign',
  E: 'encr',
  A: 'auth',
};

const EXPIRY_OPTIONS = [
  { value: '365', label: '1 year (365d)' },
  { value: '730', label: '2 years (730d)' },
  { value: '1825', label: '5 years (1825d)' },
];

function daysUntil(dateStr: string): number {
  const expiry = new Date(dateStr).getTime();
  const now = Date.now();
  return Math.floor((expiry - now) / (1000 * 60 * 60 * 24));
}

export function SubkeyCard({ usage, subkey, keyFp, canEdit, onReload }: SubkeyCardProps) {
  const title = USAGE_LABELS[usage];
  const setStatus = useUiStore((s) => s.setStatus);

  const [showForm, setShowForm] = useState(false);
  const [expiryDays, setExpiryDays] = useState('365');
  const [formLoading, setFormLoading] = useState(false);

  function handleConfirm(): void {
    const days = parseInt(expiryDays, 10);
    setFormLoading(true);

    const op =
      subkey !== undefined
        ? renewSubkeyCmd(keyFp, subkey.fingerprint, days)
        : addSubkeyCmd(keyFp, USAGE_FULL[usage], days);

    op
      .then(() => {
        setShowForm(false);
        const action = subkey !== undefined ? 'renewed' : 'added';
        setStatus('success', `${title} subkey ${action}.`);
        onReload();
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        setStatus('error', `Subkey operation failed: ${msg}`);
      })
      .finally(() => {
        setFormLoading(false);
      });
  }

  // ── Ghost card (subkey missing) ─────────────────────────────────
  if (subkey === undefined) {
    return (
      <div className={styles.cardGhost} aria-label={`${title} subkey — missing`}>
        <span className={styles.cardTitleGhost}>{title}</span>
        <span className={styles.ghostPlaceholder}>—</span>
        {canEdit && !showForm && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => { setShowForm(true); }}
            style={{ marginTop: '6px', alignSelf: 'flex-start' }}
          >
            Add
          </Button>
        )}
        {canEdit && showForm && (
          <div className={styles.renewForm}>
            <Select
              label="Expiry"
              options={EXPIRY_OPTIONS}
              value={expiryDays}
              onChange={setExpiryDays}
            />
            <div className={styles.renewActions}>
              <Button
                variant="primary"
                size="sm"
                loading={formLoading}
                onClick={handleConfirm}
              >
                Confirm
              </Button>
              <Button
                variant="ghost"
                size="sm"
                disabled={formLoading}
                onClick={() => { setShowForm(false); }}
              >
                Cancel
              </Button>
            </div>
          </div>
        )}
      </div>
    );
  }

  // ── Expiry badge ────────────────────────────────────────────────
  let expiryBadge: React.ReactNode;
  let expiryText: string;

  if (subkey.expires === null) {
    expiryText = 'No expiry';
    expiryBadge = null; // no badge for "no expiry" — plain text is enough
  } else {
    const days = daysUntil(subkey.expires);
    expiryText = formatDate(subkey.expires);
    if (days < 0) {
      expiryBadge = (
        <Badge color="error" size="sm">Expired</Badge>
      );
    } else if (days < 90) {
      expiryBadge = (
        <Badge color="warning" size="sm">{`${days}d`}</Badge>
      );
    } else {
      expiryBadge = null; // date shown in plain text, no duplicate badge
    }
  }

  function handleCopyFp(): void {
    if (subkey === undefined) return;
    void navigator.clipboard.writeText(subkey.fingerprint)
      .then(() => { setStatus('success', 'Fingerprint copied.'); });
  }

  return (
    <div className={styles.card} aria-label={`${title} subkey`}>
      <span className={styles.cardTitle}>{title}</span>
      <span className={styles.algo}>{subkey.algo}</span>
      <div className={styles.keyIdRow}>
        <span className={styles.keyId}>{subkey.key_id}</span>
        <button
          className={styles.copyBtn}
          onClick={handleCopyFp}
          title="Copy full fingerprint"
          aria-label="Copy fingerprint"
        >
          <Copy size={13} strokeWidth={1.75} />
        </button>
      </div>
      <div className={styles.expiryRow}>
        <span className={styles.expiryLabel}>{subkey.expires !== null ? expiryText : ''}</span>
        {expiryBadge}
      </div>

      {canEdit && !showForm && (
        <Button
          variant="ghost"
          size="sm"
          onClick={() => { setShowForm(true); }}
          style={{ marginTop: '6px', alignSelf: 'flex-start' }}
        >
          Renew
        </Button>
      )}

      {canEdit && showForm && (
        <div className={styles.renewForm}>
          <Select
            label="New expiry"
            options={EXPIRY_OPTIONS}
            value={expiryDays}
            onChange={setExpiryDays}
          />
          <div className={styles.renewActions}>
            <Button
              variant="primary"
              size="sm"
              loading={formLoading}
              onClick={handleConfirm}
            >
              Confirm
            </Button>
            <Button
              variant="ghost"
              size="sm"
              disabled={formLoading}
              onClick={() => { setShowForm(false); }}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
