import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Theme } from '../types';

interface ConfigStore {
  theme: Theme;
  language: 'en' | 'fr';
  setTheme: (t: Theme) => void;
  setLanguage: (l: 'en' | 'fr') => void;
}

function applyTheme(theme: Theme): void {
  document.documentElement.classList.toggle('theme-ussr', theme === 'ussr');
}

export const useConfigStore = create<ConfigStore>()(
  persist(
    (set) => ({
      theme: 'catppuccin',
      language: 'en',
      setTheme: (theme) => {
        applyTheme(theme);
        set({ theme });
      },
      setLanguage: (language) => set({ language }),
    }),
    {
      name: 'pgpilot-config',
      onRehydrateStorage: () => (state) => {
        if (state) {
          applyTheme(state.theme);
        }
      },
    },
  ),
);
