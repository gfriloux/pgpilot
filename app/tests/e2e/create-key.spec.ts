import { test, expect } from 'playwright/test';
import { resetConfig } from './helpers/theme';

test.beforeEach(async ({ page }) => {
  await resetConfig(page);
  await page.goto('/create-key');
  await expect(page.locator('h1')).toContainText('Create Key');
});

test('form fields are visible', async ({ page }) => {
  await expect(page.getByLabel('Full name')).toBeVisible();
  await expect(page.getByLabel('Email address')).toBeVisible();
  await expect(page.getByLabel('Subkey expiry')).toBeVisible();
});

test('submitting empty form shows validation errors', async ({ page }) => {
  await page.getByRole('button', { name: /Create key/i }).click();
  await expect(page.getByText('Full name is required.')).toBeVisible();
  await expect(page.getByText('Email address is required.')).toBeVisible();
});

test('submitting name only shows email validation error', async ({ page }) => {
  await page.getByLabel('Full name').fill('Test User');
  await page.getByRole('button', { name: /Create key/i }).click();
  await expect(page.getByText('Email address is required.')).toBeVisible();
  await expect(page.getByText('Full name is required.')).not.toBeVisible();
});

test('submitting invalid email shows email validation error', async ({ page }) => {
  await page.getByLabel('Full name').fill('Test User');
  await page.getByLabel('Email address').fill('notanemail');
  await page.getByRole('button', { name: /Create key/i }).click();
  await expect(page.getByText('Enter a valid email address.')).toBeVisible();
});

test('submitting valid data navigates back to /', async ({ page }) => {
  await page.getByLabel('Full name').fill('Test User');
  await page.getByLabel('Email address').fill('test@example.com');
  await page.getByRole('button', { name: /Create key/i }).click();
  // Mock returns a fingerprint immediately, then navigate('/') is called
  await expect(page).toHaveURL('/');
  await expect(page.locator('h1, [class*="listTitle"]').filter({ hasText: 'My Keys' })).toBeVisible();
});

test('expiry select has the 3 expected options', async ({ page }) => {
  const select = page.getByLabel('Subkey expiry');
  await expect(select).toBeVisible();

  // Verify option values exist in the select
  const options = await select.locator('option').allTextContents();
  expect(options).toContain('1 year');
  expect(options).toContain('2 years (recommended)');
  expect(options).toContain('5 years');
});

test('changing expiry selection works', async ({ page }) => {
  const select = page.getByLabel('Subkey expiry');
  await select.selectOption('1825');
  await expect(select).toHaveValue('1825');
});

test('Cancel button navigates back to /', async ({ page }) => {
  await page.getByRole('button', { name: /Cancel/i }).click();
  await expect(page).toHaveURL('/');
});
