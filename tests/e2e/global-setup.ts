import { request, type FullConfig } from "@playwright/test";
import { ADMIN_USER_AGENT, ROOT_EMAIL, ROOT_PASSWORD } from "./support/api";

/**
 * Signs the administrator in once per run. The configured root address
 * becomes admin on sign-up (every run but the first gets "Email already
 * exists", which is fine). The token is handed to the workers through the
 * environment: sessions are bound to the User-Agent that created them, so
 * the admin fixture opens its browser context with the same one. One shared
 * session keeps the admin tests from racing each other on the root account
 * (concurrent writes to one user conflict).
 */
export default async function globalSetup(config: FullConfig) {
  const baseURL = config.projects[0]?.use.baseURL;
  if (!baseURL) throw new Error("baseURL is not configured");
  const api = await request.newContext({ baseURL, userAgent: ADMIN_USER_AGENT });
  try {
    await api.post("/api/v1/auth/register", {
      data: { email: ROOT_EMAIL, password: ROOT_PASSWORD },
    });
    const response = await api.post("/api/v1/auth/login", {
      data: { email: ROOT_EMAIL, password: ROOT_PASSWORD },
    });
    if (!response.ok()) {
      throw new Error(`root login failed (${response.status()}): ${await response.text()}`);
    }
    const body = await response.json();
    process.env.E2E_ROOT_TOKEN = body.token;
    process.env.E2E_ROOT_ID = body.user.id;
  } finally {
    await api.dispose();
  }
}
