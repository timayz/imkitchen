import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for imkitchen E2E tests
 * See https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    // Desktop browsers
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },

    // Mobile devices - iOS Safari 14+
    {
      name: 'iphone-12',
      use: { ...devices['iPhone 12'] },
    },
    {
      name: 'ipad-pro',
      use: { ...devices['iPad Pro'] },
    },

    // Mobile devices - Android Chrome 90+
    {
      name: 'samsung-galaxy',
      use: { ...devices['Galaxy S9+'] },
    },
  ],

  /* Run local dev server before starting tests */
  // webServer: {
  //   command: 'cargo run -- serve',
  //   url: 'http://localhost:3000',
  //   reuseExistingServer: !process.env.CI,
  // },
});
