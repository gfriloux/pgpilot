import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/import');
  await expect(page.locator('h1')).toContainText('Import Key');
});

test('Import page loads with 4 tabs', async ({ page }) => {
  await expect(page.getByRole('tab', { name: 'Paste' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'URL' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Keyserver' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'File' })).toBeVisible();
});

test('Paste tab is active by default', async ({ page }) => {
  await expect(page.getByRole('tab', { name: 'Paste' })).toHaveAttribute('aria-selected', 'true');
});

test('Paste tab shows textarea', async ({ page }) => {
  await expect(page.locator('textarea#paste-input')).toBeVisible();
});

test('switching to URL tab shows URL input', async ({ page }) => {
  await page.getByRole('tab', { name: 'URL' }).click();
  await expect(page.getByRole('tab', { name: 'URL' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByLabel('Key URL')).toBeVisible();
});

test('switching to Keyserver tab shows search input', async ({ page }) => {
  await page.getByRole('tab', { name: 'Keyserver' }).click();
  await expect(page.getByRole('tab', { name: 'Keyserver' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByLabel('Search query')).toBeVisible();
});

test('switching to File tab shows unavailability notice', async ({ page }) => {
  await page.getByRole('tab', { name: 'File' }).click();
  await expect(page.getByRole('tab', { name: 'File' })).toHaveAttribute('aria-selected', 'true');
  // File tab shows a notice about native dialog not being available
  await expect(page.getByRole('status')).toBeVisible();
});

test('Paste tab — typing in textarea enables the Import button', async ({ page }) => {
  const textarea = page.locator('textarea#paste-input');
  const importBtn = page.getByRole('button', { name: /^Import$/i });

  // Button is visible initially
  await expect(importBtn).toBeVisible();

  // Type PGP content
  await textarea.fill('-----BEGIN PGP PUBLIC KEY BLOCK-----\ntest\n-----END PGP PUBLIC KEY BLOCK-----');

  // Button should still be visible and not disabled
  await expect(importBtn).not.toBeDisabled();
});

test('Paste tab — empty paste shows validation error on Import click', async ({ page }) => {
  await page.getByRole('button', { name: /^Import$/i }).click();
  await expect(page.getByText(/Paste an armored PGP public key/i)).toBeVisible();
});

test('Keyserver tab — empty query shows validation error', async ({ page }) => {
  await page.getByRole('tab', { name: 'Keyserver' }).click();
  await page.getByRole('button', { name: /Search.*Import/i }).click();
  await expect(page.getByText(/Enter a fingerprint/i)).toBeVisible();
});

test('Keyserver tab — typing email activates search button', async ({ page }) => {
  await page.getByRole('tab', { name: 'Keyserver' }).click();
  const input = page.getByLabel('Search query');
  await input.fill('alice@example.com');
  const searchBtn = page.getByRole('button', { name: /Search.*Import/i });
  await expect(searchBtn).not.toBeDisabled();
});

test('Cancel button navigates to public-keys', async ({ page }) => {
  await page.getByRole('button', { name: /Cancel/i }).click();
  await expect(page).toHaveURL(/public-keys/);
});

test('tabs keyboard navigation works with ArrowRight', async ({ page }) => {
  // Focus Paste tab and press ArrowRight to move to URL tab
  await page.getByRole('tab', { name: 'Paste' }).focus();
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('tab', { name: 'URL' })).toHaveAttribute('aria-selected', 'true');
});
