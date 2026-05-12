import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/');
  await page.waitForSelector('aside', { state: 'visible' });
});

test('sidebar is visible on load', async ({ page }) => {
  await expect(page.locator('aside')).toBeVisible();
  await expect(page.getByText('PGPilot')).toBeVisible();
});

test('sidebar contains all nav sections', async ({ page }) => {
  // Section labels are in span.navGroupLabel elements inside the sidebar
  const sidebar = page.locator('aside');
  // Use exact text match scoped to the sidebar to avoid ambiguity
  await expect(sidebar.getByText('Keys', { exact: true })).toBeVisible();
  await expect(sidebar.getByText('Operations', { exact: true })).toBeVisible();
  await expect(sidebar.getByText('Tools', { exact: true })).toBeVisible();
});

test('navigates to My Keys and shows content', async ({ page }) => {
  await page.getByRole('link', { name: 'My Keys' }).click();
  // My Keys uses a span.listTitle, not an h1
  const sidebar = page.locator('aside');
  await expect(sidebar.getByRole('link', { name: 'My Keys' })).toHaveAttribute('aria-current', 'page');
  // The page has a listTitle span with "My Keys"
  await expect(page.getByText('My Keys').first()).toBeVisible();
});

test('navigates to Public Keys', async ({ page }) => {
  await page.getByRole('link', { name: 'Public Keys' }).click();
  // Public Keys uses a div.listTitle
  await expect(page.getByText('Public Keys').first()).toBeVisible();
});

test('navigates to Encrypt', async ({ page }) => {
  await page.getByRole('link', { name: 'Encrypt' }).click();
  await expect(page.locator('h1')).toContainText(/Encrypt/i);
});

test('navigates to Decrypt', async ({ page }) => {
  await page.getByRole('link', { name: 'Decrypt' }).click();
  await expect(page.locator('h1')).toBeVisible();
});

test('navigates to Sign', async ({ page }) => {
  await page.getByRole('link', { name: 'Sign' }).click();
  await expect(page.locator('h1')).toContainText(/Sign/i);
});

test('navigates to Verify', async ({ page }) => {
  await page.getByRole('link', { name: 'Verify' }).click();
  await expect(page.locator('h1')).toBeVisible();
});

test('navigates to Chat', async ({ page }) => {
  await page.getByRole('link', { name: 'Chat' }).click();
  // Chat uses span.listTitle (no h1) — find the "Chat" text in the list panel header
  await expect(page.getByText('Chat').first()).toBeVisible();
  // Alternatively verify the link became active
  await expect(page.getByRole('link', { name: 'Chat' })).toHaveAttribute('aria-current', 'page');
});

test('navigates to Health', async ({ page }) => {
  await page.getByRole('link', { name: 'Health' }).click();
  await expect(page.locator('h1')).toContainText(/Diagnostic/i);
});

test('navigates to Settings', async ({ page }) => {
  await page.getByRole('link', { name: 'Settings' }).click();
  await expect(page.locator('h1')).toContainText(/Settings/i);
});

test('active nav link has active CSS class when on My Keys', async ({ page }) => {
  // Already on '/' after beforeEach
  const myKeysLink = page.getByRole('link', { name: 'My Keys' });
  // The active link gains aria-current="page" from react-router's NavLink
  await expect(myKeysLink).toHaveAttribute('aria-current', 'page');
});

test('active nav link changes when switching pages', async ({ page }) => {
  await page.getByRole('link', { name: 'Settings' }).click();
  await expect(page.locator('h1')).toContainText(/Settings/i);
  const settingsLink = page.getByRole('link', { name: 'Settings' });
  await expect(settingsLink).toHaveAttribute('aria-current', 'page');

  // My Keys should no longer be active
  const myKeysLink = page.getByRole('link', { name: 'My Keys' });
  await expect(myKeysLink).not.toHaveAttribute('aria-current', 'page');
});
