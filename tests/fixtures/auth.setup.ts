import { test as setup, expect } from '@playwright/test';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdir } from 'fs/promises';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const authFile = join(__dirname, '.auth', 'user.json');

/**
 * Authentication setup for E2E tests (Story 10.1 - AC1)
 *
 * This setup runs once before all tests to create an authenticated session.
 * The session state is saved to .auth/user.json and reused by all test files,
 * eliminating the need for manual login in each test.
 *
 * Environment variables:
 * - TEST_USER_EMAIL: Email for test user (default: test@example.com)
 * - TEST_USER_PASSWORD: Password for test user (default: password123)
 */
setup('authenticate', async ({ page }) => {
  // Navigate to login page
  await page.goto('/login');

  // Login with test credentials from environment variables
  const testEmail = process.env.TEST_USER_EMAIL || 'test@example.com';
  const testPassword = process.env.TEST_USER_PASSWORD || 'password123';

  await page.fill('input[name="email"]', testEmail);
  await page.fill('input[name="password"]', testPassword);
  await page.click('button[type="submit"]');

  // Wait for successful login (redirect to dashboard)
  await page.waitForURL('/dashboard');

  // Verify authentication succeeded
  await expect(page.locator('body')).not.toContainText('Login');

  // Ensure .auth directory exists before saving storage state
  await mkdir(dirname(authFile), { recursive: true });

  // Save authenticated state to file for reuse
  await page.context().storageState({ path: authFile });
});
