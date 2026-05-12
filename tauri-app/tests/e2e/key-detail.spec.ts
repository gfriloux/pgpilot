import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/');
  await page.waitForSelector('[role="listbox"]', { state: 'visible' });
});

test('Alice detail — Migrate to YubiKey button is disabled (no card)', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.locator('h2')).toContainText('Alice Dupont');

  // The migrate button is rendered but disabled because cardConnected === false
  const migrateBtn = page.getByRole('button', { name: /Migrate to YubiKey/i });
  await expect(migrateBtn).toBeVisible();
  await expect(migrateBtn).toBeDisabled();
});

test('Alice detail — Revocation Certificate section is visible', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.locator('h2')).toContainText('Alice Dupont');

  // The revocation certificate section title
  await expect(page.getByText('Revocation Certificate')).toBeVisible();
});

test('Alice detail — trust picker is NOT visible (has secret)', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.locator('h2')).toContainText('Alice Dupont');

  // Trust picker (Set trust radiogroup) only shows for public keys
  await expect(page.getByRole('radiogroup', { name: 'Trust level' })).not.toBeVisible();
});

test('Alice detail — trust level shows Ultimate', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.getByText('Ultimate')).toBeVisible();
});

test('Alice detail — Export public button is visible', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.getByRole('button', { name: /Export public/i })).toBeVisible();
});

test('Alice detail — Delete button is visible', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.getByRole('button', { name: /Delete/i }).first()).toBeVisible();
});

test('Bob detail — trust picker IS visible (public-only key)', async ({ page }) => {
  // Bob is a public-only key — we access him via Public Keys page
  await page.getByRole('link', { name: 'Public Keys' }).click();
  await page.waitForSelector('[role="listbox"]', { state: 'visible' });

  const list = page.getByRole('listbox');
  await list.getByText('Bob Martin').click();
  await expect(page.locator('h2')).toContainText('Bob Martin');

  // Trust picker should be visible for public-only key
  await expect(page.getByRole('radiogroup', { name: 'Trust level' })).toBeVisible();
});

test('Bob detail — Migrate to YubiKey button is NOT rendered (no secret)', async ({ page }) => {
  await page.getByRole('link', { name: 'Public Keys' }).click();
  await page.waitForSelector('[role="listbox"]', { state: 'visible' });

  const list = page.getByRole('listbox');
  await list.getByText('Bob Martin').click();
  await expect(page.locator('h2')).toContainText('Bob Martin');

  // Migrate button only renders when has_secret && !on_card
  await expect(page.getByRole('button', { name: /Migrate to YubiKey/i })).not.toBeVisible();
});

test('switching between keys updates the detail panel', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.locator('h2')).toContainText('Alice Dupont');

  await list.getByText('Charlie Moreau').click();
  await expect(page.locator('h2')).toContainText('Charlie Moreau');
});
