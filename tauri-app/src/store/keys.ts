import { create } from 'zustand';
import type { KeyInfo } from '../types/ipc';

interface KeysStore {
  keys: KeyInfo[];
  loading: boolean;
  error: string | null;
  selectedFp: string | null;
  setKeys: (keys: KeyInfo[]) => void;
  setLoading: (v: boolean) => void;
  setError: (e: string | null) => void;
  selectKey: (fp: string | null) => void;
}

export const useKeysStore = create<KeysStore>()((set) => ({
  keys: [],
  loading: false,
  error: null,
  selectedFp: null,
  setKeys: (keys) => set({ keys }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  selectKey: (selectedFp) => set({ selectedFp }),
}));
