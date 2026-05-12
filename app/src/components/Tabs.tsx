import { useRef } from 'react';
import styles from './Tabs.module.css';

export interface Tab {
  id: string;
  label: string;
}

export interface TabsProps {
  tabs: Tab[];
  activeTab: string;
  onChange: (id: string) => void;
}

export function Tabs({ tabs, activeTab, onChange }: TabsProps) {
  const listRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
    const buttons = listRef.current?.querySelectorAll<HTMLButtonElement>('[role="tab"]');
    if (!buttons) return;

    let next = index;
    if (e.key === 'ArrowRight') {
      next = (index + 1) % tabs.length;
    } else if (e.key === 'ArrowLeft') {
      next = (index - 1 + tabs.length) % tabs.length;
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = tabs.length - 1;
    } else {
      return;
    }

    e.preventDefault();
    const target = buttons[next];
    if (target) {
      target.focus();
      const tab = tabs[next];
      if (tab) onChange(tab.id);
    }
  };

  return (
    <div role="tablist" ref={listRef} className={styles.tablist} aria-label="Navigation tabs">
      {tabs.map((tab, i) => (
        <button
          key={tab.id}
          role="tab"
          aria-selected={tab.id === activeTab}
          tabIndex={tab.id === activeTab ? 0 : -1}
          className={[styles.tab, tab.id === activeTab ? styles.active : undefined]
            .filter(Boolean)
            .join(' ')}
          onClick={() => onChange(tab.id)}
          onKeyDown={(e) => handleKeyDown(e, i)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
