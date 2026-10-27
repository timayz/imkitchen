import { test as base, type BrowserContext, type Page } from "@playwright/test";
import {
  ADMIN_USER_AGENT,
  Api,
  PASSWORD,
  ROOT_EMAIL,
  ROOT_PASSWORD,
  installSession,
  register,
  uniqueEmail,
  type Session,
} from "./support/api";

/**
 * Shared fixtures. Every test gets its own freshly registered account, so
 * tests never share state and can run in parallel against one server.
 *
 * - `account`: a signed-in user (cookie installed on the default context),
 *   with the guided tours skipped so overlays never intercept clicks.
 * - `api`: the JSON API client for that account, for seeding.
 * - `admin`: a second browser context signed in as the configured root
 *   administrator (one session for the whole run, see global-setup.ts),
 *   with its own page and API client.
 */

export type Admin = {
  context: BrowserContext;
  page: Page;
  api: Api;
  session: Session;
};

export const test = base.extend<{ account: Session; api: Api; admin: Admin }>({
  account: async ({ context, baseURL }, use) => {
    const session = await register(context.request, uniqueEmail(), PASSWORD);
    await installSession(context, baseURL!, session.token);
    await new Api(context.request, session.token).skipTours();
    await use(session);
  },

  api: async ({ context, account }, use) => {
    await use(new Api(context.request, account.token));
  },

  admin: async ({ browser, baseURL }, use) => {
    const token = process.env.E2E_ROOT_TOKEN;
    const id = process.env.E2E_ROOT_ID;
    if (!token || !id) throw new Error("no admin session: global-setup did not run");
    const context = await browser.newContext({ userAgent: ADMIN_USER_AGENT });
    const session: Session = { id, email: ROOT_EMAIL, password: ROOT_PASSWORD, token };
    await installSession(context, baseURL!, token);
    const api = new Api(context.request, token);
    const page = await context.newPage();
    await use({ context, page, api, session });
    await context.close();
  },
});

export { expect } from "@playwright/test";
