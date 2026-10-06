import { test, expect } from '@playwright/test';

test('home page is served over the configured baseURL', async ({ page, baseURL }) => {
  const response = await page.goto('/');
  expect(response?.ok()).toBe(true);
  expect(page.url()).toBe(`${baseURL}/`);
});
