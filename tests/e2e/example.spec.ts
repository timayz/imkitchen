import { test, expect } from '@playwright/test';

test('health check returns OK', async ({ page }) => {
  await page.goto('/health');

  const text = await page.textContent('body');
  expect(text).toBe('OK');
});

test('home page loads', async ({ page }) => {
  await page.goto('/');

  const text = await page.textContent('body');
  expect(text).toContain('ImKitchen');
});
