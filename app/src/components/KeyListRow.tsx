import type { KeyInfo } from '../types/ipc';
import { formatDate } from '../utils/format';
import styles from './KeyListRow.module.css';

interface KeyListRowProps {
  keyInfo: KeyInfo;
  selected: boolean;
  /** Whether this key has a known keyserver publication status */
  published?: boolean;
  onClick: () => void;
}

export function KeyListRow({ keyInfo, selected, published, onClick }: KeyListRowProps) {
  const expiresLabel = keyInfo.expires !== null ? formatDate(keyInfo.expires) : '—';

  // Slot 1: card (YubiKey)
  const cardSlot = keyInfo.on_card ? (
    <span className={`${styles.statusSlot} ${styles.iconCard}`} aria-label="On YubiKey">
      &#9672;
    </span>
  ) : (
    <span className={styles.statusSlot} aria-hidden="true" />
  );

  // Slot 2: keyserver publication
  let keyserverSlot: React.ReactNode;
  if (published === true) {
    keyserverSlot = (
      <span className={`${styles.statusSlot} ${styles.iconPublished}`} aria-label="Published">
        &#10003;
      </span>
    );
  } else if (published === false) {
    keyserverSlot = (
      <span className={`${styles.statusSlot} ${styles.iconNotPublished}`} aria-label="Not published">
        &#9675;
      </span>
    );
  } else {
    keyserverSlot = <span className={styles.statusSlot} aria-hidden="true" />;
  }

  // Slot 3: trust — only for public keys (no secret)
  let trustSlot: React.ReactNode;
  if (!keyInfo.has_secret) {
    const trustOk = keyInfo.trust === 'full' || keyInfo.trust === 'ultimate';
    trustSlot = trustOk ? (
      <span className={`${styles.statusSlot} ${styles.iconTrustOk}`} aria-label="Trust OK">
        &#10003;
      </span>
    ) : (
      <span className={`${styles.statusSlot} ${styles.iconTrustWarn}`} aria-label="Trust low">
        &#9888;
      </span>
    );
  } else {
    trustSlot = <span className={styles.statusSlot} aria-hidden="true" />;
  }

  return (
    <div
      role="option"
      aria-selected={selected}
      tabIndex={0}
      className={`${styles.row}${selected ? ` ${styles.rowSelected}` : ''}`}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick();
        }
      }}
    >
      <div className={styles.nameCol}>
        <span className={styles.nameText}>{keyInfo.name}</span>
        <span className={styles.emailText}>{keyInfo.email}</span>
      </div>

      <div className={styles.expiresCol}>{expiresLabel}</div>

      <div className={styles.statusCol} aria-label="Status">
        {cardSlot}
        {keyserverSlot}
        {trustSlot}
      </div>
    </div>
  );
}
