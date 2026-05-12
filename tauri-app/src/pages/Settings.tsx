import { useConfigStore } from '../store/config';
import type { Theme } from '../types';
import { Button } from '../components/Button';
import { UssrBanner } from '../components/UssrBanner';
import styles from './Settings.module.css';

interface ThemePreviewDef {
  value: Theme;
  label: string;
  sidebar: string;
  content: string;
  accent: string;
  line: string;
}

const THEME_DEFS: ThemePreviewDef[] = [
  {
    value: 'catppuccin',
    label: 'Catppuccin',
    sidebar: '#303446',
    content: '#363a4f',
    accent: '#ca9ee6',
    line: '#49506a',
  },
  {
    value: 'ussr',
    label: 'USSR',
    sidebar: '#0f0d09',
    content: '#f0ebd8',
    accent: '#d82c20',
    line: '#d9d3bc',
  },
];

export default function Settings() {
  const theme = useConfigStore((s) => s.theme);
  const setTheme = useConfigStore((s) => s.setTheme);
  const language = useConfigStore((s) => s.language);
  const setLanguage = useConfigStore((s) => s.setLanguage);

  const pageTitle = theme === 'ussr' ? 'Commissariat Settings' : 'Settings';

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        <div className={styles.cardContent}>
        <h1 className={styles.title}>{pageTitle}</h1>

        {/* Theme */}
        <section className={styles.section}>
          <p className={styles.sectionLabel}>Theme</p>
          <div className={styles.themePicker}>
            {THEME_DEFS.map((def) => {
              const isActive = theme === def.value;
              return (
                <button
                  key={def.value}
                  type="button"
                  className={[
                    styles.themeCard,
                    isActive ? styles.themeCardActive : '',
                  ].join(' ')}
                  onClick={() => setTheme(def.value)}
                  aria-pressed={isActive}
                  aria-label={`${def.label} theme`}
                >
                  <div className={styles.themePreview}>
                    <div
                      className={styles.previewSidebar}
                      style={{ background: def.sidebar }}
                    />
                    <div
                      className={styles.previewContent}
                      style={{ background: def.content }}
                    >
                      <div
                        className={styles.previewLineAccent}
                        style={{ background: def.accent }}
                      />
                      <div
                        className={styles.previewLine}
                        style={{ background: def.line, width: '80%' }}
                      />
                      <div
                        className={styles.previewLine}
                        style={{ background: def.line, width: '60%' }}
                      />
                      <div
                        className={styles.previewLine}
                        style={{ background: def.line, width: '70%' }}
                      />
                    </div>
                  </div>
                  <div className={styles.themeCardLabel}>{def.label}</div>
                </button>
              );
            })}
          </div>
        </section>

        {/* Language */}
        <section className={styles.section}>
          <p className={styles.sectionLabel}>Language</p>
          <div className={styles.langRow}>
            {(['en', 'fr'] as const).map((lang) => (
              <Button
                key={lang}
                variant={language === lang ? 'primary' : 'ghost'}
                size="md"
                onClick={() => setLanguage(lang)}
                aria-pressed={language === lang}
              >
                {lang.toUpperCase()}
              </Button>
            ))}
          </div>
        </section>

        </div>
        <UssrBanner n={29} variant="fill" />
      </div>
    </div>
  );
}
