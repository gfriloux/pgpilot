/**
 * Mock for @tauri-apps/api/event used in VITE_MOCK=true mode (E2E tests, dev without binary).
 *
 * listen() returns a no-op unlisten function. In mock mode there is no Tauri
 * backend to emit events, so listeners never fire — this is fine for tests
 * that only check UI state driven by mock invoke() responses.
 */

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function listen<T>(
  _event: string,
  _handler: (event: { payload: T }) => void,
): Promise<() => void> {
  return Promise.resolve(() => undefined);
}
