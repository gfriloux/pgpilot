import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useKeys } from '../hooks/useKeys';
import { useKeysStore } from '../store/keys';
import { useUiStore } from '../store/ui';
import { checkKeyserver } from '../ipc/keys';
import { Button } from '../components/Button';
import { KeyListRow } from '../components/KeyListRow';
import { KeyDetail } from '../components/KeyDetail';
import { UssrBanner } from '../components/UssrBanner';
import styles from './MyKeys.module.css';

export default function PublicKeys() {
  const navigate = useNavigate();
  const { loading, error, reload } = useKeys();
  const allKeys = useKeysStore((s) => s.keys);
  const keys = allKeys.filter((k) => !k.has_secret);
  const selectedFp = useKeysStore((s) => s.selectedFp);
  const selectKey = useKeysStore((s) => s.selectKey);
  const setStatus = useUiStore((s) => s.setStatus);

  const [keyserverStatuses, setKeyserverStatuses] = useState<Record<string, boolean>>({});
  const checkedFps = useRef<Set<string>>(new Set());

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

  function handleAfterDelete(): void {
    selectKey(null);
  }

  // Backup is not applicable for public-only keys, but the prop is required.
  function handleBackup(_fp: string): void {
    setStatus('info', 'Backup is only available for secret keys.');
  }

  return (
    <div className={styles.page}>
      <div className={styles.listPanel}>
        <div className={styles.listHeader}>
          <div className={styles.listTitle}>Public Keys</div>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => { void navigate('/import'); }}
            aria-label="Import a key"
          >
            +
          </Button>
        </div>

        <UssrBanner n={23} />
        <div className={styles.columnHeader}>
          <span className={styles.columnHeaderName}>Name</span>
          <span className={styles.columnHeaderExpires}>Expires</span>
          <span className={styles.columnHeaderStatus}>Status</span>
        </div>

        <div className={styles.keyList} role="listbox" aria-label="Public keys">
          {loading && (
            <p className={styles.keyListEmpty}>Loading keys…</p>
          )}
          {error !== null && (
            <p className={styles.keyListError}>
              {error}{' '}
              <button onClick={reload} className={styles.retryBtn}>Retry</button>
            </p>
          )}
          {!loading && error === null && keys.length === 0 && (
            <p className={styles.keyListEmpty}>
              No public keys. Import one with the + button above.
            </p>
          )}
          {keys.map((key) => (
            <KeyListRow
              key={key.fingerprint}
              keyInfo={key}
              selected={selectedFp === key.fingerprint}
              {...(keyserverStatuses[key.fingerprint] !== undefined
                ? { published: keyserverStatuses[key.fingerprint] }
                : {})}
              onClick={() => {
                selectKey(
                  selectedFp === key.fingerprint ? null : key.fingerprint,
                );
              }}
            />
          ))}
        </div>
      </div>

      <div className={styles.detailPanel}>
        {selectedKey !== null ? (
          <KeyDetail
            keyInfo={selectedKey}
            bannerN={17}
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
