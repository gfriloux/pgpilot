# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: encrypt.spec.ts >> untrusted key chip shows warning indicator
- Location: tests/e2e/encrypt.spec.ts:75:1

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('[aria-pressed]').filter({ hasText: 'Bob Martin' }).locator('[aria-label="untrusted"]')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('[aria-pressed]').filter({ hasText: 'Bob Martin' }).locator('[aria-label="untrusted"]')

```

```yaml
- complementary:
  - text: PGPilot
  - navigation:
    - text: Keys
    - link "My Keys":
      - /url: /
    - link "Public Keys":
      - /url: /public-keys
    - text: Operations
    - link "Encrypt":
      - /url: /encrypt
    - link "Decrypt":
      - /url: /decrypt
    - link "Sign":
      - /url: /sign
    - link "Verify":
      - /url: /verify
    - text: Tools
    - link "Chat":
      - /url: /chat
    - link "Health":
      - /url: /health
    - link "Settings":
      - /url: /settings
  - text: v0.8.0-mock
- main:
  - heading "Encrypt Files" [level=1]
  - paragraph: Files
  - button "Select files"
  - paragraph: Recipients (0 selected)
  - paragraph: Output format
  - button ".gpg (binary)"
  - button ".asc (armored)"
  - button "Encrypt"
```

# Test source

```ts
  1  | import { test, expect } from 'playwright/test';
  2  | import { resetConfig } from './helpers/theme';
  3  | 
  4  | test.beforeEach(async ({ page }) => {
  5  |   await resetConfig(page);
  6  |   // Navigate to My Keys first so the keys store is populated via useKeys()
  7  |   await page.goto('/');
  8  |   await page.waitForSelector('[role="listbox"]', { state: 'visible' });
  9  |   // Now navigate to Encrypt — the Zustand store retains the loaded keys
  10 |   await page.getByRole('link', { name: 'Encrypt' }).click();
  11 |   await expect(page.locator('h1')).toContainText(/Encrypt/i);
  12 | });
  13 | 
  14 | test('Encrypt page loads with Select files button', async ({ page }) => {
  15 |   await expect(page.getByRole('button', { name: /Select files/i })).toBeVisible();
  16 | });
  17 | 
  18 | test('Recipients section is visible', async ({ page }) => {
  19 |   await expect(page.getByText(/Recipients/i)).toBeVisible();
  20 | });
  21 | 
  22 | test('recipient chips show all mock keys', async ({ page }) => {
  23 |   // All 3 mock keys should appear as recipient chips (aria-pressed attribute marks them as toggle buttons)
  24 |   await expect(page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' })).toBeVisible();
  25 |   await expect(page.locator('[aria-pressed]').filter({ hasText: 'Bob Martin' })).toBeVisible();
  26 |   await expect(page.locator('[aria-pressed]').filter({ hasText: 'Charlie Moreau' })).toBeVisible();
  27 | });
  28 | 
  29 | test('clicking a recipient chip toggles its selected state', async ({ page }) => {
  30 |   const aliceChip = page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' });
  31 |   // Initially not selected
  32 |   await expect(aliceChip).toHaveAttribute('aria-pressed', 'false');
  33 |   // Click to select
  34 |   await aliceChip.click();
  35 |   await expect(aliceChip).toHaveAttribute('aria-pressed', 'true');
  36 |   // Click again to deselect
  37 |   await aliceChip.click();
  38 |   await expect(aliceChip).toHaveAttribute('aria-pressed', 'false');
  39 | });
  40 | 
  41 | test('recipients counter updates when selecting a key', async ({ page }) => {
  42 |   await expect(page.getByText('Recipients (0 selected)')).toBeVisible();
  43 |   const aliceChip = page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' });
  44 |   await aliceChip.click();
  45 |   await expect(page.getByText('Recipients (1 selected)')).toBeVisible();
  46 | });
  47 | 
  48 | test('format toggle switches between .gpg and .asc', async ({ page }) => {
  49 |   const gpgBtn = page.getByRole('button', { name: '.gpg (binary)' });
  50 |   const ascBtn = page.getByRole('button', { name: '.asc (armored)' });
  51 | 
  52 |   // Both buttons are visible
  53 |   await expect(gpgBtn).toBeVisible();
  54 |   await expect(ascBtn).toBeVisible();
  55 | 
  56 |   // Switch to .asc
  57 |   await ascBtn.click();
  58 |   await expect(ascBtn).toBeVisible();
  59 | 
  60 |   // Switch back to .gpg
  61 |   await gpgBtn.click();
  62 |   await expect(gpgBtn).toBeVisible();
  63 | });
  64 | 
  65 | test('Encrypt button is visible', async ({ page }) => {
  66 |   await expect(page.getByRole('button', { name: /^Encrypt$/i })).toBeVisible();
  67 | });
  68 | 
  69 | test('trusted key chip shows checkmark indicator', async ({ page }) => {
  70 |   // Alice has trust: 'ultimate' — the trust span has aria-label="trusted"
  71 |   const aliceChip = page.locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' });
  72 |   await expect(aliceChip.locator('[aria-label="trusted"]')).toBeVisible();
  73 | });
  74 | 
  75 | test('untrusted key chip shows warning indicator', async ({ page }) => {
  76 |   // Bob has trust: 'marginal' — the trust span has aria-label="untrusted"
  77 |   const bobChip = page.locator('[aria-pressed]').filter({ hasText: 'Bob Martin' });
> 78 |   await expect(bobChip.locator('[aria-label="untrusted"]')).toBeVisible();
     |                                                             ^ Error: expect(locator).toBeVisible() failed
  79 | });
  80 | 
  81 | test('Output format section is visible', async ({ page }) => {
  82 |   await expect(page.getByText('Output format')).toBeVisible();
  83 | });
  84 | 
```