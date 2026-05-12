import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useKeys } from '../hooks/useKeys';
import { useKeysStore } from '../store/keys';
import { useUiStore } from '../store/ui';
import { cardStatus, checkKeyserver } from '../ipc/keys';
import type { KeyInfo } from '../types/ipc';
import { Button } from '../components/Button';
import { KeyListRow } from '../components/KeyListRow';
import { KeyDetail } from '../components/KeyDetail';
import { UssrBanner } from '../components/UssrBanner';
import styles from './MyKeys.module.css';

/** Returns the number of days until the given ISO date string. Negative = past. */
function daysUntil(dateStr: string): number {
  return Math.floor((new Date(dateStr).getTime() - Date.now()) / (1000 * 60 * 60 * 24));
}

/** Keys expiring within 90 days (and not already expired) */
function findExpiringSoon(keys: KeyInfo[]): { key: KeyInfo; days: number }[] {
  const results: { key: KeyInfo; days: number }[] = [];
  for (const key of keys) {
    if (key.expires !== null) {
      const days = daysUntil(key.expires);
      if (days >= 0 && days < 90) {
        results.push({ key, days });
      }
    }
  }
  return results;
}

export default function MyKeys() {
  const navigate = useNavigate();
  const { loading, error, reload } = useKeys();
  const allKeys = useKeysStore((s) => s.keys);
  const keys = allKeys.filter((k) => k.has_secret);
  const selectedFp = useKeysStore((s) => s.selectedFp);
  const selectKey = useKeysStore((s) => s.selectKey);
  const setStatus = useUiStore((s) => s.setStatus);

  const [cardConnected, setCardConnected] = useState(false);
  const [keyserverStatuses, setKeyserverStatuses] = useState<Record<string, boolean>>({});
  const checkedFps = useRef<Set<string>>(new Set());

  useEffect(() => {
    cardStatus()
      .then((info) => { setCardConnected(info !== null); })
      .catch(() => { setCardConnected(false); });
  }, []);

  // Check keyserver publication status for each key (once per fingerprint)
  useEffect(() => {
    keys.forEach((key) => {
      if (!checkedFps.current.has(key.fingerprint)) {
        checkedFps.current.add(key.fingerprint);
        checkKeyserver(key.fingerprint)
          .then((pub) => {
            setKeyserverStatuses((prev) => ({ ...prev, [key.fingerprint]: pub }));
          })
          .catch(() => undefined);
      }
    });
  }, [keys]);

  const selectedKey = selectedFp !== null
    ? keys.find((k) => k.fingerprint === selectedFp) ?? null
    : null;

  const expiringSoon = findExpiringSoon(keys);

  function handleBackup(fp: string): void {
    setStatus('info', `Backup: file dialog not yet implemented for ${fp.slice(0, 8)}…`);
  }

  function handleAfterDelete(): void {
    selectKey(null);
    void navigate('/');
  }

  return (
    <div className={styles.page}>
      {/* ── List panel ──────────────────────────────────────────── */}
      <div className={styles.listPanel}>
        {/* Header row: title + add button */}
        <div className={styles.listHeader}>
          <span className={styles.listTitle}>My Keys</span>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => { void navigate('/create-key'); }}
            aria-label="Create new key"
          >
            +
          </Button>
        </div>

        {/* Expiry warning banner */}
        {expiringSoon.length > 0 && (
          <div className={styles.expiryBanner} role="alert">
            <div className={styles.expiryBannerTitle}>Expiring soon</div>
            {expiringSoon.map(({ key, days }) => (
              <div key={key.fingerprint} className={styles.expiryBannerRow}>
                <span className={styles.expiryBannerName}>{key.name}</span>
                <span className={styles.expiryBannerDays}>Expires in {days}d</span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => { selectKey(key.fingerprint); }}
                  style={{ padding: '1px 6px', fontSize: '0.6875rem' }}
                >
                  Renew
                </Button>
              </div>
            ))}
          </div>
        )}

        {/* Column header */}
        <UssrBanner n={18} />
        <div className={styles.columnHeader} aria-hidden="true">
          <span className={styles.columnHeaderName}>Name</span>
          <span className={styles.columnHeaderExpires}>Expires</span>
          <span className={styles.columnHeaderStatus}>Status</span>
        </div>

        {/* Key list */}
        <div
          className={styles.keyList}
          role="listbox"
          aria-label="My keys"
          aria-orientation="vertical"
        >
          {loading && (
            <p className={styles.keyListEmpty}>Loading keys…</p>
          )}

          {!loading && error !== null && (
            <p className={styles.keyListError}>
              Error: {error}
              <button className={styles.retryBtn} onClick={reload}>
                Retry
              </button>
            </p>
          )}

          {!loading && error === null && keys.length === 0 && (
            <p className={styles.keyListEmpty}>
              No keys found. Create your first key with the + button above.
            </p>
          )}

          {keys.map((key) => (
            <KeyListRow
              key={key.fingerprint}
              keyInfo={key}
              selected={key.fingerprint === selectedFp}
              {...(keyserverStatuses[key.fingerprint] !== undefined
                ? { published: keyserverStatuses[key.fingerprint] }
                : {})}
              onClick={() => { selectKey(key.fingerprint); }}
            />
          ))}
        </div>
      </div>

      {/* ── Detail panel ────────────────────────────────────────── */}
      <div className={styles.detailPanel}>
        {selectedKey !== null ? (
          <KeyDetail
            keyInfo={selectedKey}
            bannerN={25}
            cardConnected={cardConnected}
            onBackup={handleBackup}
            onAfterDelete={handleAfterDelete}
            onReload={reload}
          />
        ) : (
          <div className={styles.detailPlaceholder}>
            &#8592; Select a key
          </div>
        )}
      </div>
    </div>
  );
}
