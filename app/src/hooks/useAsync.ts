import { useState, useEffect, useCallback, useRef } from 'react';

interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

interface UseAsyncResult<T> extends AsyncState<T> {
  reload: () => void;
}

export function useAsync<T>(
  fn: () => Promise<T>,
  deps: readonly unknown[],
): UseAsyncResult<T> {
  const [state, setState] = useState<AsyncState<T>>({
    data: null,
    loading: true,
    error: null,
  });

  const counter = useRef(0);

  const run = useCallback(() => {
    const id = ++counter.current;
    setState((prev) => ({ ...prev, loading: true, error: null }));
    fn()
      .then((data) => {
        if (id === counter.current) {
          setState({ data, loading: false, error: null });
        }
      })
      .catch((err: unknown) => {
        if (id === counter.current) {
          const message = err instanceof Error ? err.message : String(err);
          setState((prev) => ({ ...prev, loading: false, error: message }));
        }
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps, react-hooks/use-memo
  }, deps);

  useEffect(() => {
    run();
  }, [run]);

  return { ...state, reload: run };
}
