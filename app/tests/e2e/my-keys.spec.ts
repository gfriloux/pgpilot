import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/');
  // Wait until the key list is populated (loading finishes)
  await page.waitForSelector('[role="listbox"]', { state: 'visible' });
});

test('displays the My Keys title and list area', async ({ page }) => {
  // My Keys uses a span.listTitle (not h1), so scope to the list panel
  const listPanel = page.locator('[role="listbox"]').locator('..');
  await expect(page.getByRole('listbox', { name: 'My keys' })).toBeVisible();
  // At least one element with text "My Keys" is visible on the page
  await expect(page.getByText('My Keys').first()).toBeVisible();
});

test('shows mock keys with has_secret — Alice and Charlie', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  // Alice has has_secret: true
  await expect(list.getByText('Alice Dupont')).toBeVisible();
  // Charlie has has_secret: true
  await expect(list.getByText('Charlie Moreau')).toBeVisible();
  // Bob has has_secret: false — should NOT appear in My Keys
  await expect(list.getByText('Bob Martin')).not.toBeVisible();
});

test('clicking a key shows detail panel with key name', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  // Detail panel shows the name in an h2
  await expect(page.locator('h2')).toContainText('Alice Dupont');
});

test('detail panel shows fingerprint and algorithm', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  // Fingerprint label in the info rows
  await expect(page.getByText('Fingerprint')).toBeVisible();
  // Algorithm label
  await expect(page.getByText('Algorithm')).toBeVisible();
  // Alice uses ed25519 — shown as the algorithm value
  // Filter to the algorithm value element specifically
  await expect(page.locator('span').filter({ hasText: /^ed25519$/ }).first()).toBeVisible();
});

test('detail panel for Alice shows Subkeys section', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  await expect(page.getByText('Subkeys')).toBeVisible();
});

test('Alice has all 3 subkeys (S, E, A) displayed', async ({ page }) => {
  const list = page.getByRole('listbox', { name: 'My keys' });
  await list.getByText('Alice Dupont').click();
  // SubkeyCard renders titles: Signature, Encryption, Auth SSH
  await expect(page.getByText('Signature')).toBeVisible();
  await expect(page.getByText('Encryption')).toBeVisible();
  await expect(page.getByText('Auth SSH')).toBeVisible();
});

test('Create key button is visible and navigates to create-key', async ({ page }) => {
  const createBtn = page.getByRole('button', { name: 'Create new key' });
  await expect(createBtn).toBeVisible();
  await createBtn.click();
  await expect(page).toHaveURL(/create-key/);
  await expect(page.locator('h1')).toContainText('Create Key');
});

test('placeholder is shown when no key is selected', async ({ page }) => {
  // No key selected initially
  await expect(page.getByText(/Select a key/i)).toBeVisible();
});
