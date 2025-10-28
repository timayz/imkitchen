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
  // AC1: Parallel workers (4) for <5 minute execution time
  workers: 4,
  reporter: 'html',

  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    // AC9: Video recording for failed tests only (storage optimization)
    video: 'retain-on-failure',
    // Individual test timeout: 60 seconds (catch hung tests while allowing reasonable execution time)
    timeout: 60000,
  },

  projects: [
    // Setup project for authentication (runs once before all tests)
    {
      name: 'setup',
      testMatch: /.*\.setup\.ts/,
    },

    // Desktop browsers
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // AC1: Load authenticated session created by setup project
        storageState: './fixtures/.auth/user.json',
      },
      dependencies: ['setup'],
    },
    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        storageState: './fixtures/.auth/user.json',
      },
      dependencies: ['setup'],
    },
    {
      name: 'webkit',
      use: {
        ...devices['Desktop Safari'],
        storageState: './fixtures/.auth/user.json',
      },
      dependencies: ['setup'],
    },

    // Mobile devices - iOS Safari 14+
    {
      name: 'iphone-12',
      use: {
        ...devices['iPhone 12'],
        storageState: './fixtures/.auth/user.json',
      },
      dependencies: ['setup'],
    },
    {
      name: 'ipad-pro',
      use: {
        ...devices['iPad Pro'],
        storageState: './fixtures/.auth/user.json',
      },
      dependencies: ['setup'],
    },

    // Mobile devices - Android Chrome 90+
    {
      name: 'samsung-galaxy',
      use: {
        ...devices['Galaxy S9+'],
        storageState: './fixtures/.auth/user.json',
      },
      dependencies: ['setup'],
    },
  ],

  /* Run local dev server before starting tests */
  // webServer: {
  //   command: 'cargo run -- serve',
  //   url: 'http://localhost:3000',
  //   reuseExistingServer: !process.env.CI,
  // },
});
