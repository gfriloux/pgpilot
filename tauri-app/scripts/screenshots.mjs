/**
 * screenshots.mjs
 * Launches Vite in mock mode, captures a screenshot of every view in both
 * themes (Catppuccin and USSR), then saves them to docs/screenshots/.
 *
 * Usage:
 *   VITE_MOCK=true node scripts/screenshots.mjs
 */

import { chromium } from 'playwright';
import { spawn } from 'child_process';
import { mkdirSync, existsSync } from 'fs';

const BASE_URL = 'http://localhost:1421';
const OUT_DIR = new URL('../docs/screenshots/', import.meta.url).pathname;

const NIX_CHROMIUM_CANDIDATES = [
  '/nix/store/h16ak6ir2p2n8nhqgbg6iqzy17243h3j-playwright-chromium-headless-shell/chrome-headless-shell-linux64/chrome-headless-shell',
  '/nix/store/6js92rbq8qyyyrblvn7s2nvr0grmyydh-playwright-chromium/chrome-linux64/chrome',
  '/nix/store/5d32m7b4znr83wh3ajaxwr5kynplqri3-playwright-browsers/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
];
const executablePath =
  process.env['PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH'] ??
  NIX_CHROMIUM_CANDIDATES.find(existsSync);

const VIEWS = [
  { path: '/', name: 'my-keys' },
  { path: '/public-keys', name: 'public-keys' },
  { path: '/encrypt', name: 'encrypt' },
  { path: '/decrypt', name: 'decrypt' },
  { path: '/sign', name: 'sign' },
  { path: '/verify', name: 'verify' },
  { path: '/chat', name: 'chat' },
  { path: '/health', name: 'health' },
  { path: '/settings', name: 'settings' },
];

/**
 * Set the theme in localStorage so Zustand re-hydrates with it on reload.
 * @param {import('playwright').Page} page
 * @param {'catppuccin'|'ussr'} theme
 */
async function applyTheme(page, theme) {
  await page.evaluate((t) => {
    localStorage.setItem(
      'pgpilot-config',
      JSON.stringify({ state: { theme: t, language: 'en' }, version: 0 }),
    );
  }, theme);
  await page.reload();
  // Wait for the sidebar to be rendered before capturing
  await page.waitForSelector('aside', { state: 'visible', timeout: 8000 });
}

/**
 * Wait for the Vite dev server to be reachable.
 * @param {string} url
 * @param {number} retries
 * @param {number} intervalMs
 */
async function waitForServer(url, retries = 30, intervalMs = 500) {
  const { default: http } = await import('http');
  for (let i = 0; i < retries; i++) {
    await new Promise((resolve) => {
      const req = http.get(url, () => resolve(true));
      req.on('error', () => resolve(false));
      req.setTimeout(300, () => { req.destroy(); resolve(false); });
    }).then(async (ok) => {
      if (!ok) await new Promise((r) => setTimeout(r, intervalMs));
      return ok;
    });
    // Try a fetch to confirm readiness
    try {
      const res = await fetch(url);
      if (res.ok || res.status === 200) return;
    } catch {
      // keep waiting
    }
    await new Promise((r) => setTimeout(r, intervalMs));
  }
  throw new Error(`Server at ${url} did not start in time.`);
}

async function main() {
  mkdirSync(OUT_DIR, { recursive: true });

  // Check if a server is already running
  let serverProcess = null;
  let serverAlreadyRunning = false;

  try {
    const res = await fetch(BASE_URL, { signal: AbortSignal.timeout(1000) });
    if (res.ok || res.status === 200) {
      serverAlreadyRunning = true;
      console.log(`[screenshots] Using existing server at ${BASE_URL}`);
    }
  } catch {
    // Server not running — start it
  }

  if (!serverAlreadyRunning) {
    console.log('[screenshots] Starting Vite mock server…');
    serverProcess = spawn('npm', ['run', 'dev'], {
      env: { ...process.env, VITE_MOCK: 'true' },
      stdio: ['ignore', 'pipe', 'pipe'],
      shell: true,
    });

    serverProcess.stdout?.on('data', (d) => process.stdout.write(d));
    serverProcess.stderr?.on('data', (d) => process.stderr.write(d));

    await waitForServer(BASE_URL);
    console.log('[screenshots] Server ready.');
  }

  const browser = await chromium.launch({ headless: true, ...(executablePath ? { executablePath } : {}) });
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();

  // Pre-navigate so localStorage is writable
  await page.goto(BASE_URL);
  await page.waitForSelector('aside', { state: 'visible', timeout: 10000 });

  let total = 0;

  for (const theme of /** @type {const} */ (['catppuccin', 'ussr'])) {
    console.log(`\n[screenshots] Theme: ${theme}`);
    await applyTheme(page, theme);

    for (const view of VIEWS) {
      await page.goto(`${BASE_URL}${view.path}`);

      // Wait for content to stabilise: sidebar + main content present
      await page.waitForSelector('aside', { state: 'visible', timeout: 8000 });
      // Small pause for async data (mock delay = 60ms) to resolve
      await page.waitForTimeout(200);

      const fileName = `${view.name}--${theme}.png`;
      const outPath = `${OUT_DIR}${fileName}`;
      await page.screenshot({ path: outPath, fullPage: false });
      console.log(`  [ok] ${fileName}`);
      total++;
    }
  }

  await browser.close();

  if (serverProcess) {
    serverProcess.kill('SIGTERM');
    console.log('\n[screenshots] Server stopped.');
  }

  console.log(`\n[screenshots] Done. ${total} screenshots saved to ${OUT_DIR}`);
}

main().catch((err) => {
  console.error('[screenshots] Error:', err);
  process.exit(1);
});
