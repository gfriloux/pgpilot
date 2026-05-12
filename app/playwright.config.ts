import { defineConfig } from 'playwright/test';
import { existsSync } from 'node:fs';

/**
 * On NixOS the Playwright-downloaded Chromium binary cannot run because it is
 * a dynamically linked ELF that needs /lib64/ld-linux-x86-64.so.2.
 *
 * This config auto-detects a Nix-patched Chromium from well-known Nix store
 * paths and uses it as the executable. Falls back to the downloaded browser in
 * non-NixOS environments (e.g. GitHub Actions CI).
 *
 * Override with: PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/path/to/chrome npm run test:e2e
 */
function resolveChromiumExecutable(): string | undefined {
  const envOverride = process.env['PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH'];
  if (envOverride) return envOverride;

  // Nix-patched headless shell (playwright-browsers derivation, revision 1208)
  const nixCandidates: string[] = [
    '/nix/store/5d32m7b4znr83wh3ajaxwr5kynplqri3-playwright-browsers/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
    '/nix/store/6js92rbq8qyyyrblvn7s2nvr0grmyydh-playwright-chromium/chrome-linux64/chrome',
  ];

  for (const candidate of nixCandidates) {
    if (existsSync(candidate)) return candidate;
  }

  return undefined;
}

const chromiumExecutable = resolveChromiumExecutable();

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
        ...(chromiumExecutable !== undefined
          ? { launchOptions: { executablePath: chromiumExecutable } }
          : {}),
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
