import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  // Navigate to My Keys first so the keys store is populated via useKeys()
  await page.goto('/');
  await page.waitForSelector('[role="listbox"]', { state: 'visible' });
  // Now navigate to Encrypt — the Zustand store retains the loaded keys
  await page.getByRole('link', { name: 'Encrypt' }).click();
  await expect(page.locator('h1')).toContainText(/Encrypt/i);
});

test('Encrypt page loads with Select files button', async ({ page }) => {
  await expect(page.getByRole('button', { name: /Select files/i })).toBeVisible();
});

test('Recipients section is visible', async ({ page }) => {
  await expect(page.getByText(/Recipients/i)).toBeVisible();
});

test('recipient chips show all mock keys', async ({ page }) => {
  // All 3 mock keys should appear as recipient chips (aria-pressed attribute marks them as toggle buttons)
  await expect(page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' })).toBeVisible();
  await expect(page.locator('[aria-pressed]').filter({ hasText: 'Bob Martin' })).toBeVisible();
  await expect(page.locator('[aria-pressed]').filter({ hasText: 'Charlie Moreau' })).toBeVisible();
});

test('clicking a recipient chip toggles its selected state', async ({ page }) => {
  const aliceChip = page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' });
  // Initially not selected
  await expect(aliceChip).toHaveAttribute('aria-pressed', 'false');
  // Click to select
  await aliceChip.click();
  await expect(aliceChip).toHaveAttribute('aria-pressed', 'true');
  // Click again to deselect
  await aliceChip.click();
  await expect(aliceChip).toHaveAttribute('aria-pressed', 'false');
});

test('recipients counter updates when selecting a key', async ({ page }) => {
  await expect(page.getByText('Recipients (0 selected)')).toBeVisible();
  const aliceChip = page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' });
  await aliceChip.click();
  await expect(page.getByText('Recipients (1 selected)')).toBeVisible();
});

test('format toggle switches between .gpg and .asc', async ({ page }) => {
  const gpgBtn = page.getByRole('button', { name: '.gpg (binary)' });
  const ascBtn = page.getByRole('button', { name: '.asc (armored)' });

  // Both buttons are visible
  await expect(gpgBtn).toBeVisible();
  await expect(ascBtn).toBeVisible();

  // Switch to .asc
  await ascBtn.click();
  await expect(ascBtn).toBeVisible();

  // Switch back to .gpg
  await gpgBtn.click();
  await expect(gpgBtn).toBeVisible();
});

test('Encrypt button is visible', async ({ page }) => {
  await expect(page.getByRole('button', { name: /^Encrypt$/i })).toBeVisible();
});

test('trusted key chip shows checkmark indicator', async ({ page }) => {
  // Alice has trust: 'ultimate' — the trust span has aria-label="trusted"
  const aliceChip = page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' });
  await expect(aliceChip.locator('[aria-label="trusted"]')).toBeVisible();
});

test('untrusted key chip shows warning indicator', async ({ page }) => {
  // Bob has trust: 'marginal' — the trust span has aria-label="untrusted"
  const bobChip = page.locator('[aria-pressed]').filter({ hasText: 'Bob Martin' });
  await expect(bobChip.locator('[aria-label="untrusted"]')).toBeVisible();
});

test('Output format section is visible', async ({ page }) => {
  await expect(page.getByText('Output format')).toBeVisible();
});
