import { test, expect } from "./fixtures";
import { AdminUsersPage } from "./pages/admin";
import { PASSWORD, login } from "./support/api";

test.describe("admin", () => {
  test("regular users cannot open the admin area", async ({ page, account }) => {
    await page.goto("/admin/users");
    await expect(page.getByRole("heading", { name: "Access Forbidden" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "User Management" })).toHaveCount(0);
    void account;
  });

  test("users are listed, searched and filtered", async ({ account, admin }) => {
    const users = new AdminUsersPage(admin.page);
    await users.goto();
    await expect(admin.page.getByRole("heading", { name: "User Management" })).toBeVisible();

    await users.search.fill(account.email);
    await expect(users.row(account.id)).toBeVisible();
    await expect(users.rows()).toHaveCount(1);
    await expect(users.row(account.id)).toContainText("Active");

    await users.filterRole("Admin");
    await expect(users.row(account.id)).toHaveCount(0);
    await users.filterRole("");
    await expect(users.row(account.id)).toBeVisible();
  });

  test("suspending a user locks them out until reactivated", async ({ page, account, admin }) => {
    const users = new AdminUsersPage(admin.page);
    await users.goto();
    await users.search.fill(account.email);
    const row = users.row(account.id);
    await expect(row).toBeVisible();

    await row.getByRole("button", { name: "Suspend" }).click();
    await expect(row).toContainText("Suspended");

    await expect
      .poll(async () => {
        const response = await page.request.post("/api/v1/auth/login", {
          data: { email: account.email, password: PASSWORD },
        });
        return response.status();
      })
      .not.toBe(200);
    await page.goto("/recipes");
    await expect(page).toHaveURL(/\/login$/);

    await row.getByRole("button", { name: "Reactivate" }).click();
    await expect(row).toContainText("Active");
    await expect
      .poll(async () => {
        const response = await page.request.post("/api/v1/auth/login", {
          data: { email: account.email, password: PASSWORD },
        });
        return response.status();
      })
      .toBe(200);
  });

  test("the edit modal changes a user's role", async ({ page, account, admin }) => {
    const users = new AdminUsersPage(admin.page);
    await users.goto();
    await users.search.fill(account.email);
    await expect(users.row(account.id)).toBeVisible();
    await expect(users.row(account.id)).toContainText("User");

    await users.setRole(account.id, "Chef");
    await expect(users.row(account.id)).toContainText("Chef");

    const session = await login(page.request, account.email, PASSWORD);
    const me = await page.request.get("/api/v1/me", {
      headers: { authorization: `Bearer ${session.token}` },
    });
    expect((await me.json()).is_chef).toBe(true);
  });
});
