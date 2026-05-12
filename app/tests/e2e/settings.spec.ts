import { test, expect } from 'playwright/test';
import { resetConfig, setTheme } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/settings');
  await expect(page.locator('h1')).toContainText('Settings');
});

test('Settings page loads with two theme cards', async ({ page }) => {
  await expect(page.getByRole('button', { name: /Catppuccin theme/i })).toBeVisible();
  await expect(page.getByRole('button', { name: /USSR theme/i })).toBeVisible();
});

test('clicking USSR theme applies theme-ussr class to html element', async ({ page }) => {
  await page.getByRole('button', { name: /USSR theme/i }).click();
  // The applyTheme function toggles 'theme-ussr' on document.documentElement
  await expect(page.locator('html')).toHaveClass(/theme-ussr/);
});

test('clicking Catppuccin theme removes theme-ussr class', async ({ page }) => {
  // First switch to USSR
  await page.getByRole('button', { name: /USSR theme/i }).click();
  await expect(page.locator('html')).toHaveClass(/theme-ussr/);

  // Switch back to Catppuccin
  await page.getByRole('button', { name: /Catppuccin theme/i }).click();
  await expect(page.locator('html')).not.toHaveClass(/theme-ussr/);
});

test('clicking FR language button stores language in localStorage', async ({ page }) => {
  await page.getByRole('button', { name: 'FR' }).click();
  const stored = await page.evaluate(() => {
    const raw = localStorage.getItem('pgpilot-config');
    if (!raw) return null;
    return JSON.parse(raw) as { state: { language: string } };
  });
  expect(stored?.state.language).toBe('fr');
});

test('clicking EN language button stores language in localStorage', async ({ page }) => {
  // First set FR
  await page.getByRole('button', { name: 'FR' }).click();
  // Then set back to EN
  await page.getByRole('button', { name: 'EN' }).click();
  const stored = await page.evaluate(() => {
    const raw = localStorage.getItem('pgpilot-config');
    if (!raw) return null;
    return JSON.parse(raw) as { state: { language: string } };
  });
  expect(stored?.state.language).toBe('en');
});

test('Catppuccin theme — page title text is readable', async ({ page }) => {
  await expect(page.locator('h1')).toContainText('Settings');
  // Ensure the h1 has visible text (not invisible)
  const h1 = page.locator('h1');
  await expect(h1).toBeVisible();
});

test('USSR theme — page title changes to Soviet variant', async ({ page }) => {
  await page.getByRole('button', { name: /USSR theme/i }).click();
  // In USSR mode the page title becomes "Commissariat Settings"
  await expect(page.locator('h1')).toContainText(/Commissariat/i);
});

test('both theme cards are visible and have correct aria-pressed', async ({ page }) => {
  // Default is Catppuccin
  const catBtn = page.getByRole('button', { name: /Catppuccin theme/i });
  const ussrBtn = page.getByRole('button', { name: /USSR theme/i });

  await expect(catBtn).toHaveAttribute('aria-pressed', 'true');
  await expect(ussrBtn).toHaveAttribute('aria-pressed', 'false');

  await ussrBtn.click();
  await expect(ussrBtn).toHaveAttribute('aria-pressed', 'true');
  await expect(catBtn).toHaveAttribute('aria-pressed', 'false');
});

test('USSR theme — sidebar background matches USSR design', async ({ page }) => {
  await setTheme(page, 'ussr');
  await page.goto('/settings');
  await expect(page.locator('h1')).toBeVisible();

  // Check the sidebar computed background color approximates the USSR dark value #0f0d09
  const sidebar = page.locator('aside');
  await expect(sidebar).toBeVisible();
  const bg = await sidebar.evaluate((el) => getComputedStyle(el).backgroundColor);
  // The USSR sidebar-bg CSS variable maps to #0f0d09 = rgb(15, 13, 9)
  expect(bg).toMatch(/rgb\(15,\s*13,\s*9\)/);
});
