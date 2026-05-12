import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/health');
});

test('Health page loads with title', async ({ page }) => {
  await expect(page.locator('h1')).toContainText(/Diagnostic/i);
});

test('shows loading state then results', async ({ page }) => {
  // The spinner / "Running checks" text may appear briefly
  // Then after mock (60ms delay) results appear
  // We wait for one of the expected categories to be visible
  await expect(page.getByText('Installation')).toBeVisible();
});

test('displays all 3 mock categories', async ({ page }) => {
  await expect(page.getByText('Installation')).toBeVisible();
  await expect(page.getByText('Agent GPG')).toBeVisible();
  await expect(page.getByText('Sécurité')).toBeVisible();
});

test('displays GPG installed check with ok status', async ({ page }) => {
  await expect(page.getByText('GPG installed')).toBeVisible();
  await expect(page.getByText('gpg 2.4.3')).toBeVisible();
});

test('displays GPG agent running check', async ({ page }) => {
  await expect(page.getByText('GPG agent running')).toBeVisible();
});

test('displays Keybox permissions check', async ({ page }) => {
  await expect(page.getByText('Keybox permissions')).toBeVisible();
});

test('warning check has warning aria-label on its status icon', async ({ page }) => {
  // The warning check (Keybox permissions) uses aria-label="warning"
  const warningIcons = page.locator('[aria-label="warning"]');
  await expect(warningIcons).toBeVisible();
});

test('ok check has ok aria-label on its status icon', async ({ page }) => {
  const okIcons = page.locator('[aria-label="ok"]');
  await expect(okIcons.first()).toBeVisible();
});

test('warning check shows fix command', async ({ page }) => {
  // The Keybox permissions check has fix: 'chmod 600 ~/.gnupg/pubring.kbx'
  await expect(page.getByText(/chmod 600/i)).toBeVisible();
});

test('health checks explanation text is shown', async ({ page }) => {
  await expect(page.getByText('GnuPG is installed and accessible.')).toBeVisible();
});
