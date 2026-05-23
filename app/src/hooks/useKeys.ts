import { useEffect } from 'react';
import { useAsync } from './useAsync';
import { listKeys } from '../ipc/keys';
import { useKeysStore } from '../store/keys';
import type { KeyInfo } from '../types/ipc';

interface UseKeysResult {
  keys: KeyInfo[];
  loading: boolean;
  error: string | null;
  reload: () => void;
}

export function useKeys(): UseKeysResult {
  const { data, loading, error, reload } = useAsync(listKeys, []);
  const keys = useKeysStore((s) => s.keys);
  const setKeys = useKeysStore((s) => s.setKeys);
  const setLoading = useKeysStore((s) => s.setLoading);
  const setError = useKeysStore((s) => s.setError);

  useEffect(() => {
    setLoading(loading);
  }, [loading, setLoading]);

  useEffect(() => {
    setError(error);
  }, [error, setError]);

  useEffect(() => {
    if (data !== null) {
      setKeys(data);
    }
  }, [data, setKeys]);

  return { keys, loading, error, reload };
}
