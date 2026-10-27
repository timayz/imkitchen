import { defineConfig, devices, type ReporterDescription } from "@playwright/test";

/**
 * Playwright configuration for the imkitchen end-to-end tests.
 *
 * Locally the tests go through Traefik (`make up`), which terminates TLS
 * with the mkcert certificate in .docker/traefik/certs and proxies to the
 * dev server on port 3000; MailDev from the same compose file receives the
 * outgoing mail. In CI (`.github/workflows/e2e.yaml`) they talk to the
 * server directly over http.
 *
 * The web server runs against its own SQLite file (never the dev database)
 * and is migrated before it starts. `reuseExistingServer` means a dev server
 * already listening on port 3000 is used as is outside CI.
 *
 * Environment:
 * - PLAYWRIGHT_BASE_URL  where the browser goes (default https://imkitchen.localhost)
 * - IMKITCHEN_BIN        server command (default `cargo run -q --`)
 * - E2E_DATABASE_URL     SQLite URL for the test database (default sqlite:e2e.db)
 * - MAILDEV_URL          MailDev REST API (default http://localhost:1080)
 * - E2E_ROOT_EMAIL / E2E_ROOT_PASSWORD  the `[root]` account (config defaults)
 *
 * See https://playwright.dev/docs/test-configuration
 */
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? "https://imkitchen.localhost";
const devServerURL = "http://localhost:3000";
const bin = process.env.IMKITCHEN_BIN ?? "cargo run -q --";
const databaseURL = process.env.E2E_DATABASE_URL ?? "sqlite:e2e.db?mode=rwc";

const reporter: ReporterDescription[] = [["html", { open: "never" }], ["list"]];
if (process.env.CI) reporter.push(["github"]);

export default defineConfig({
  testDir: "./tests/e2e",
  globalSetup: "./tests/e2e/global-setup.ts",

  /* Every test registers its own account, so files and tests run in parallel. */
  fullyParallel: true,
  workers: process.env.CI ? 2 : undefined,

  /* Fail the build on CI if you accidentally left test.only in the source code */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,

  timeout: 45_000,
  expect: { timeout: 10_000 },

  reporter,

  use: {
    baseURL,
    /* Capture what went wrong: trace on the first retry, video and screenshot on failure. */
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    /* The PWA service worker reloads a freshly loaded page once it takes
       control, which races with the first interaction: keep it out of the
       tests (the PWA itself is not what they cover). */
    serviceWorkers: "block",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      /* Phone layout (bottom navigation, folded aisles…) for the tests tagged @mobile. */
      name: "mobile-chrome",
      use: { ...devices["Pixel 7"] },
      grep: /@mobile/,
    },
  ],

  webServer: {
    command: `${bin} migrate && ${bin} serve`,
    url: `${devServerURL}/health`,
    env: {
      IMKITCHEN__DATABASE__URL: databaseURL,
      IMKITCHEN__SERVER__URL: baseURL,
      IMKITCHEN__SERVER__PORT: "3000",
    },
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
