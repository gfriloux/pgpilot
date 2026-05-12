import { useEffect, useState } from 'react';
import { useConfigStore } from '../store/config';
import { runHealthChecks } from '../ipc/keys';
import type { HealthCheck, CheckStatus } from '../types/ipc';
import { Alert } from '../components/Alert';
import { UssrBanner } from '../components/UssrBanner';
import styles from './Health.module.css';

const STATUS_ICON: Record<CheckStatus, string> = {
  ok: '✓',
  info: 'i',
  warning: '!',
  error: '✕',
};

const STATUS_CSS: Record<CheckStatus, string> = {
  ok: styles.iconOk ?? '',
  info: styles.iconInfo ?? '',
  warning: styles.iconWarning ?? '',
  error: styles.iconError ?? '',
};

const CATEGORY_ORDER = ['Installation', 'Agent GPG', 'Sécurité'];

function groupByCategory(checks: HealthCheck[]): Map<string, HealthCheck[]> {
  const map = new Map<string, HealthCheck[]>();
  for (const check of checks) {
    const list = map.get(check.category);
    if (list !== undefined) {
      list.push(check);
    } else {
      map.set(check.category, [check]);
    }
  }
  return map;
}

export default function Health() {
  const theme = useConfigStore((s) => s.theme);
  const [checks, setChecks] = useState<HealthCheck[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const title =
    theme === 'ussr' ? 'Report to the Commissariat' : 'Diagnostic GPG';

  useEffect(() => {
    runHealthChecks()
      .then((result) => {
        setChecks(result);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  const grouped = checks !== null ? groupByCategory(checks) : null;

  // Build ordered list of categories (known order first, then any extras)
  const orderedCategories: string[] =
    grouped !== null
      ? [
          ...CATEGORY_ORDER.filter((c) => grouped.has(c)),
          ...[...grouped.keys()].filter((c) => !CATEGORY_ORDER.includes(c)),
        ]
      : [];

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <div className={styles.cardContent}>
        <h1 className={styles.title}>{title}</h1>

        {loading && (
          <div className={styles.loading}>
            <span className={styles.spinner} aria-hidden="true" />
            Running checks…
          </div>
        )}

        {error !== null && (
          <Alert variant="error" message={error} />
        )}

        {grouped !== null &&
          orderedCategories.map((cat) => {
            const items = grouped.get(cat) ?? [];
            return (
              <section key={cat} className={styles.category}>
                <p className={styles.categoryLabel}>{cat}</p>
                {items.map((check) => (
                  <div key={`${cat}-${check.name}`} className={styles.checkRow}>
                    <span
                      className={[
                        styles.statusIcon,
                        STATUS_CSS[check.status],
                      ].join(' ')}
                      aria-label={check.status}
                    >
                      {STATUS_ICON[check.status]}
                    </span>
                    <div className={styles.checkBody}>
                      <span className={styles.checkName}>{check.name}</span>
                      {check.current_value !== null && (
                        <code className={styles.checkValue}>
                          {check.current_value}
                        </code>
                      )}
                      <span className={styles.checkExplanation}>
                        {check.explanation}
                      </span>
                      {check.fix !== null && (
                        <code className={styles.checkFix}>{check.fix}</code>
                      )}
                    </div>
                  </div>
                ))}
              </section>
            );
          })}

        </div>
        <UssrBanner n={12} variant="fill" />
      </div>
    </div>
  );
}
