import { test as base } from '@playwright/test';

/**
 * Authentication fixtures for E2E tests (Story 10.1 - AC1)
 *
 * Provides `authenticatedPage` fixture that automatically loads the authenticated
 * session state created by auth.setup.ts. Tests using this fixture skip manual login.
 *
 * Usage:
 *   import { test, expect } from './fixtures/auth';
 *
 *   test('protected route test', async ({ authenticatedPage }) => {
 *     await authenticatedPage.goto('/plan');
 *     // Already authenticated via JWT cookie
 *   });
 */

export type AuthFixtures = {
  authenticatedPage: typeof base;
};

/**
 * Authenticated page fixture with JWT cookie pre-loaded
 *
 * The storage state (JWT cookie) is automatically applied via playwright.config.ts
 * setting: storageState: './fixtures/.auth/user.json'
 */
export const test = base.extend<AuthFixtures>({
  authenticatedPage: async ({ page }, use) => {
    // Page already has authenticated storage state loaded from config
    // No additional setup needed here - just pass through
    await use(page);
  },
});

export { expect } from '@playwright/test';
