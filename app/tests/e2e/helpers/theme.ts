import type { Page } from 'playwright/test';

const BASE_URL = 'http://localhost:1421';

/**
 * Navigate to the app root so that localStorage is accessible, then clear
 * the persisted Zustand config so every test starts from a clean state.
 * Call this as the very first step in beforeEach, before any page.goto.
 */
export async function resetConfig(page: Page): Promise<void> {
  // Navigate to the app first so that the origin is correct for localStorage ops
  await page.goto(BASE_URL);
  await page.evaluate(() => {
    localStorage.removeItem('pgpilot-config');
  });
}

/**
 * Switch the active theme by writing directly into Zustand's persisted localStorage key,
 * then reload so that the store re-hydrates and applyTheme() runs.
 * The page must already be on the app origin before calling this.
 */
export async function setTheme(page: Page, theme: 'catppuccin' | 'ussr'): Promise<void> {
  await page.evaluate((t) => {
    localStorage.setItem(
      'pgpilot-config',
      JSON.stringify({ state: { theme: t, language: 'en' }, version: 0 }),
    );
  }, theme);
  await page.reload();
  // Wait for the layout to be rendered after reload
  await page.waitForSelector('aside', { state: 'visible' });
}
