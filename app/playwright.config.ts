import { defineConfig } from 'playwright/test';
import { existsSync, readdirSync } from 'node:fs';

/**
 * On NixOS the Playwright-downloaded Chromium binary cannot run because it is
 * a dynamically linked ELF that needs /lib64/ld-linux-x86-64.so.2.
 *
 * This config scans the Nix store dynamically for a patched Chromium so it
 * stays valid across nixpkgs updates (no hardcoded hashes).  Falls back to
 * the downloaded browser in non-NixOS environments (e.g. GitHub Actions CI).
 *
 * Override with: PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/path/to/chrome npm run test:e2e
 */
function resolveChromiumExecutable(): string | undefined {
  const envOverride = process.env['PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH'];
  if (envOverride) return envOverride;

  // On NixOS the Nix store holds all derivations under /nix/store.
  // Match wrapped `chromium-X.Y.Z` only (not chromium-unwrapped-*, which
  // lacks sandboxing wrappers).  Sort descending for a deterministic pick
  // when multiple Chromium generations coexist in the store.
  if (existsSync('/nix/store')) {
    try {
      const matches = readdirSync('/nix/store')
        .filter((e) => /-chromium-\d+\.\d+/.test(e) && !e.includes('unwrapped'))
        .sort()
        .reverse();
      for (const entry of matches) {
        const candidate = `/nix/store/${entry}/bin/chromium`;
        if (existsSync(candidate)) return candidate;
      }
    } catch {
      // /nix/store unreadable — fall through to the downloaded browser
    }
  }

  return undefined;
}

const chromiumExecutable = resolveChromiumExecutable();

/**
 * A Chromium from the Nix store carries its own library paths.  An inherited
 * LD_LIBRARY_PATH (dev shell, or any wrapper the tests are started from) makes
 * it load libraries built against another glibc and die at launch with
 * "version `GLIBC_x.y' not found".  Start it without that variable.
 */
function envWithoutLdLibraryPath(): Record<string, string> {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (key !== 'LD_LIBRARY_PATH' && value !== undefined) env[key] = value;
  }
  return env;
}

const launchOptions =
  chromiumExecutable === undefined
    ? undefined
    : {
        executablePath: chromiumExecutable,
        ...(chromiumExecutable.startsWith('/nix/store/')
          ? { env: envWithoutLdLibraryPath() }
          : {}),
      };

export default defineConfig({
  testDir: './tests/e2e',
  timeout: 15_000,
  retries: 1,
  use: {
    baseURL: 'http://localhost:1421',
    headless: true,
  },
  projects: [
    {
      name: 'chromium',
      use: {
        browserName: 'chromium',
        ...(launchOptions !== undefined ? { launchOptions } : {}),
      },
    },
  ],
  webServer: {
    command: 'VITE_MOCK=true npm run dev',
    url: 'http://localhost:1421',
    reuseExistingServer: !process.env['CI'],
    timeout: 15_000,
  },
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
  ],
});
