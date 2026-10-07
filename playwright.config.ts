import { defineConfig, devices } from '@playwright/test';

/**
 * Tests go through Traefik (`make up`), which terminates TLS with the mkcert
 * certificate in .docker/traefik/certs and proxies to the dev server. The
 * dev server itself (`cargo run serve`, port 3000 per config/default.toml)
 * is what webServer waits for, so a missing Traefik fails fast on the first
 * navigation instead of timing out on startup.
 */
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? 'https://imkitchen.localhost';
const devServerURL = 'http://localhost:3000';

/**
 * Playwright configuration for imkitchen e2e and accessibility testing
 *
 * See https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests/e2e',

  /* Run tests in files in parallel */
  fullyParallel: true,

  /* Fail the build on CI if you accidentally left test.only in the source code */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,

  /* Opt out of parallel tests on CI */
  workers: process.env.CI ? 1 : undefined,

  /* Reporter to use */
  reporter: [
    ['html'],
    ['list'],
    ...(process.env.CI ? [['github']] : [])
  ],

  /* Shared settings for all the projects below */
  use: {
    /* Base URL to use in actions like `await page.goto('/')` */
    baseURL,

    /* Collect trace when retrying the failed test */
    trace: 'on-first-retry',

    /* Screenshot on failure */
    screenshot: 'only-on-failure',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
    //
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },

    /* Mobile viewports */
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },

    /* Tablet viewports */
    // {
    //   name: 'Tablet',
    //   use: {
    //     ...devices['iPad Pro'],
    //   },
    // },
  ],

  /* Run your local dev server before starting the tests */
  webServer: {
    command: 'cargo run serve',
    url: devServerURL,
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
