import { test, expect, type BrowserContext, type Page } from "@playwright/test";

// The guided kitchen tour on a fresh account: it opens on the first visit,
// walks through its steps, stays closed once done, and replays on demand.
//
// Each run registers its own user through the JSON API from the page's own
// request context (the session is bound to the browser's User-Agent) and
// signs in by storing the returned token as the web session cookie, which
// sidesteps the TwinSpark redirect and the first service-worker reload.

const password = "correct-horse-42";

async function signIn(page: Page, context: BrowserContext, baseURL: string) {
  const email = `tour-${Date.now()}-${Math.random().toString(36).slice(2, 8)}@example.test`;
  const registered = await page.request.post("/api/v1/auth/register", {
    data: { email, password },
  });
  expect(registered.status()).toBe(201);
  const { token } = await registered.json();
  await context.addCookies([
    { name: "auth_token", value: token, url: baseURL },
  ]);
}

test("the kitchen tour runs once and can be replayed from the help page link", async ({
  page,
  context,
  baseURL,
}) => {
  await signIn(page, context, baseURL!);
  await page.goto("/");

  // First visit: the kitchen tour opens on step 1 of 4.
  const root = page.locator("#tour-root");
  await expect(root).toBeVisible();
  const card = root.locator('[role="dialog"]');
  await expect(card.locator("[data-tour-count]")).toHaveText(/1 .* 4$/);
  await expect(card.locator("[data-tour-back]")).toBeHidden();

  await card.locator("[data-tour-next]").click();
  await expect(card.locator("[data-tour-count]")).toHaveText(/2 .* 4$/);
  await expect(card.locator("[data-tour-back]")).toBeVisible();

  await card.locator("[data-tour-back]").click();
  await expect(card.locator("[data-tour-count]")).toHaveText(/1 .* 4$/);

  await card.locator("[data-tour-next]").click();
  await card.locator("[data-tour-next]").click();
  await card.locator("[data-tour-next]").click();
  await expect(card.locator("[data-tour-count]")).toHaveText(/4 .* 4$/);
  await card.locator("[data-tour-next]").click();
  await expect(root).toHaveCount(0);

  // Completed: a reload shows the kitchen without the overlay.
  await page.reload();
  await expect(page.locator('[data-tour~="kitchen-title"]')).toBeVisible();
  await page.waitForTimeout(500);
  await expect(page.locator("#tour-root")).toHaveCount(0);

  // Replay on demand from the help page's link.
  await page.goto("/?tour=kitchen");
  await expect(page.locator("#tour-root")).toBeVisible();
  await expect(page.locator("#tour-root [data-tour-count]")).toHaveText(
    /1 .* 4$/,
  );

  // Skipping closes the overlay and drops the query parameter.
  await page.locator("#tour-root [data-tour-skip]").click();
  await expect(page.locator("#tour-root")).toHaveCount(0);
  expect(new URL(page.url()).searchParams.has("tour")).toBe(false);
});

test("the demo kitchen never shows a tour", async ({ page }) => {
  await page.goto("/demo/kitchen");
  await expect(page.locator('[data-tour~="kitchen-title"]')).toBeVisible();
  await expect(page.locator("#tour-catalog")).toHaveCount(0);
  await expect(page.locator("#tour-root")).toHaveCount(0);
});
