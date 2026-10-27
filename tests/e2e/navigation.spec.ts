import { test, expect } from "./fixtures";

/**
 * The shell around every signed-in page: the sidebar on desktop, the bottom
 * bar on phones (the `@mobile` tests also run in the mobile project).
 */
test.describe("navigation", () => {
  test("the landing page is public and links to sign-up and login", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("link", { name: "Log in" }).first()).toBeVisible();
    await expect(page.locator('a[href="/register"]').first()).toBeVisible();
  });

  test("the main sections are reachable from the navigation @mobile", async ({ page, account }) => {
    await page.goto("/");
    await expect(page.locator('[data-tour~="kitchen-title"]')).toBeVisible();

    await page.getByRole("link", { name: "Recipes" }).click();
    await expect(page).toHaveURL(/\/recipes/);
    await expect(page.getByRole("heading", { name: "Pick Recipes for Your Menu" })).toBeVisible();

    await page.getByRole("link", { name: "Groceries" }).click();
    await expect(page).toHaveURL(/\/groceries$/);
    await expect(page.getByRole("heading", { name: "Shopping run" })).toBeVisible();

    await page.getByRole("link", { name: "Settings" }).click();
    await expect(page).toHaveURL(/\/settings\/general$/);

    await page.getByRole("link", { name: /^Kitchen/ }).click();
    await expect(page).toHaveURL(/\/$/);
    await expect(page.locator('[data-tour~="kitchen-title"]')).toBeVisible();
    void account;
  });

  test("the kitchen shows the list and links to the groceries @mobile", async ({ page, api }) => {
    const recipe = await api.createRecipe({ name: "Nav lentil stew" });
    await api.addToList(recipe.id);

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Up next" })).toBeVisible();
    await expect(page.locator("#current-recipe-card")).toContainText("Nav lentil stew");
    await expect(page.getByText(/0\/1 cooked/)).toBeVisible();

    await page.locator("header").getByRole("link", { name: "Groceries" }).click();
    await expect(page).toHaveURL(/\/groceries$/);
    await expect(page.locator("#groceries-body")).toContainText("Rice");
  });

  test("the demo kitchen is browsable without an account", async ({ page }) => {
    await page.goto("/demo/kitchen");
    await expect(page.locator('[data-tour~="kitchen-title"]')).toBeVisible();
    await expect(page.getByText("You're browsing imkitchen as a guest.")).toBeVisible();
  });
});
