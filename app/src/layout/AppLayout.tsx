import { useEffect, useState } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import {
  Key,
  Globe,
  Lock,
  LockOpen,
  PenLine,
  ShieldCheck,
  MessageCircle,
  Activity,
  Settings,
  type LucideProps,
} from 'lucide-react';
import { useConfigStore } from '../store/config';
import { getVersion } from '../ipc/keys';
import { useChatEvents } from '../hooks/useChatEvents';
import styles from './AppLayout.module.css';

type IconComponent = React.ComponentType<LucideProps>;

interface NavItem {
  path: string;
  label: string;
  Icon: IconComponent;
}

interface NavSection {
  label: string;
  items: NavItem[];
}

const NAV_SECTIONS: NavSection[] = [
  {
    label: 'Keys',
    items: [
      { path: '/', label: 'My Keys', Icon: Key },
      { path: '/public-keys', label: 'Public Keys', Icon: Globe },
    ],
  },
  {
    label: 'Operations',
    items: [
      { path: '/encrypt', label: 'Encrypt', Icon: Lock },
      { path: '/decrypt', label: 'Decrypt', Icon: LockOpen },
      { path: '/sign', label: 'Sign', Icon: PenLine },
      { path: '/verify', label: 'Verify', Icon: ShieldCheck },
    ],
  },
  {
    label: 'Tools',
    items: [
      { path: '/chat', label: 'Chat', Icon: MessageCircle },
      { path: '/health', label: 'Health', Icon: Activity },
      { path: '/settings', label: 'Settings', Icon: Settings },
    ],
  },
];

export default function AppLayout() {
  const theme = useConfigStore((s) => s.theme);
  const [version, setVersion] = useState<string>('...');
  useChatEvents();

  useEffect(() => {
    document.documentElement.dataset['theme'] = theme;
  }, [theme]);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion('(mock)'));
  }, []);

  return (
    <div className={styles.layout}>
      <aside className={styles.sidebar}>
        <div className={styles.logo}>
          PGPilot
        </div>

        <nav className={styles.nav}>
          {NAV_SECTIONS.map((section) => (
            <div key={section.label} className={styles.navGroup}>
              <span className={styles.navGroupLabel}>{section.label}</span>
              {section.items.map((item) => (
                <NavLink
                  key={item.path}
                  to={item.path}
                  end={item.path === '/'}
                  className={({ isActive }) =>
                    `${styles.navLink}${isActive ? ` ${styles.navLinkActive}` : ''}`
                  }
                >
                  <item.Icon size={15} className={styles.navIcon} strokeWidth={1.75} />
                  {item.label}
                </NavLink>
              ))}
            </div>
          ))}
        </nav>

        <div className={styles.footer}>v{version}</div>
      </aside>

      <main className={styles.content}>
        <Outlet />
      </main>
    </div>
  );
}
