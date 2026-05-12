import { test, expect } from 'playwright/test';
import { setTheme, resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/');
  await page.waitForSelector('aside', { state: 'visible' });
  await setTheme(page, 'ussr');
});

test('theme-ussr class is set on html element after setTheme', async ({ page }) => {
  await expect(page.locator('html')).toHaveClass(/theme-ussr/);
});

test('Settings page in USSR theme — propaganda banner is visible', async ({ page }) => {
  await page.goto('/settings');
  await expect(page.locator('h1')).toBeVisible();
  // UssrBanner n=29 renders when theme is ussr
  const banner = page.locator('img[src="/banners/29.png"]');
  await expect(banner).toBeVisible();
});

test('Health page in USSR theme — banner 12 is visible', async ({ page }) => {
  await page.goto('/health');
  // Wait for health checks to load
  await expect(page.getByText('Installation')).toBeVisible();
  const banner = page.locator('img[src="/banners/12.png"]');
  await expect(banner).toBeVisible();
});

test('sidebar background is USSR dark color', async ({ page }) => {
  const sidebar = page.locator('aside');
  await expect(sidebar).toBeVisible();
  const bg = await sidebar.evaluate((el) => getComputedStyle(el).backgroundColor);
  // USSR sidebar-bg = #0f0d09 = rgb(15, 13, 9)
  expect(bg).toMatch(/rgb\(15,\s*13,\s*9\)/);
});

test('Encrypt page in USSR theme — banner 16 is visible', async ({ page }) => {
  await page.goto('/encrypt');
  await expect(page.locator('h1')).toContainText(/Encrypt/i);
  const banner = page.locator('img[src="/banners/16.png"]');
  await expect(banner).toBeVisible();
});

test('My Keys page in USSR theme — banner 18 is visible', async ({ page }) => {
  await page.goto('/');
  // UssrBanner n=18 is rendered in the list panel
  const banner = page.locator('img[src="/banners/18.png"]');
  await expect(banner).toBeVisible();
});

test('USSR theme — page title text is readable (visible)', async ({ page }) => {
  await page.goto('/settings');
  const h1 = page.locator('h1');
  await expect(h1).toBeVisible();
  // Ensure text content is non-empty
  const text = await h1.textContent();
  expect(text?.trim().length).toBeGreaterThan(0);
});

test('USSR theme — sidebar nav links text is visible', async ({ page }) => {
  // Sidebar text must be visible (not same color as background)
  const myKeysLink = page.getByRole('link', { name: 'My Keys' });
  await expect(myKeysLink).toBeVisible();
  const color = await myKeysLink.evaluate((el) => getComputedStyle(el).color);
  const bg = await page.locator('aside').evaluate((el) => getComputedStyle(el).backgroundColor);
  // Color must not equal the background (basic contrast check)
  expect(color).not.toBe(bg);
});

test('USSR Settings title is Commissariat Settings', async ({ page }) => {
  await page.goto('/settings');
  await expect(page.locator('h1')).toContainText('Commissariat Settings');
});

test('USSR Health title is Report to the Commissariat', async ({ page }) => {
  await page.goto('/health');
  await expect(page.locator('h1')).toContainText('Report to the Commissariat');
});

test('switching back to Catppuccin removes theme-ussr class', async ({ page }) => {
  await page.goto('/settings');
  await expect(page.locator('html')).toHaveClass(/theme-ussr/);
  await page.getByRole('button', { name: /Catppuccin theme/i }).click();
  await expect(page.locator('html')).not.toHaveClass(/theme-ussr/);
});
