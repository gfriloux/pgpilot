import { create } from 'zustand';

type StatusKind = 'success' | 'error' | 'info';

interface UiStore {
  status: { kind: StatusKind; message: string } | null;
  setStatus: (kind: StatusKind, message: string) => void;
  clearStatus: () => void;
}

export const useUiStore = create<UiStore>()((set) => ({
  status: null,
  setStatus: (kind, message) => set({ status: { kind, message } }),
  clearStatus: () => set({ status: null }),
}));
