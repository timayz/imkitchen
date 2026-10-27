import { test, expect } from "./fixtures";
import { LoginPage, RegisterPage, ResetPasswordPage, toast } from "./pages/auth";
import { PASSWORD, login, register, uniqueEmail } from "./support/api";
import { mailAvailable, resetPasswordPath, waitForMail } from "./support/mail";

test.describe("registration", () => {
  test("a new account is created from the sign-up form and can log in", async ({ page }) => {
    const email = uniqueEmail("signup");

    await new RegisterPage(page).goto();
    await new RegisterPage(page).submit(email, PASSWORD);
    await page.waitForURL("/login");

    await new LoginPage(page).submit(email, PASSWORD);
    await page.waitForURL("/");
    await expect(page.locator('[data-tour~="kitchen-title"]')).toBeVisible();
  });

  test("mismatched passwords are rejected without creating the account", async ({ page }) => {
    const email = uniqueEmail("mismatch");
    const form = new RegisterPage(page);

    await form.goto();
    await form.submit(email, PASSWORD, "something-else-1");
    await expect(toast(page)).toContainText("Passwords don't match");
    await expect(page).toHaveURL(/\/register$/);

    // Logging in proves nothing was created.
    const response = await page.request.post("/api/v1/auth/login", {
      data: { email, password: PASSWORD },
    });
    expect(response.ok()).toBe(false);
  });

  test("an email that is already taken is refused", async ({ page }) => {
    const email = uniqueEmail("taken");
    await register(page.request, email, PASSWORD);

    const form = new RegisterPage(page);
    await form.goto();
    await form.submit(email, PASSWORD);
    await expect(toast(page)).toContainText("Email already exists");
  });
});

test.describe("login", () => {
  test("wrong credentials show an error and stay on the login page", async ({ page }) => {
    const email = uniqueEmail("login");
    await register(page.request, email, PASSWORD);

    const form = new LoginPage(page);
    await form.goto();
    await form.submit(email, "not-the-password");
    await expect(toast(page)).toContainText("Invalid email or password");
    await expect(page).toHaveURL(/\/login$/);
  });

  test("protected pages redirect anonymous visitors to the login page", async ({ page }) => {
    for (const path of ["/recipes", "/groceries", "/settings/general", "/dishes"]) {
      await page.goto(path);
      await expect(page, `${path} should require a session`).toHaveURL(/\/login$/);
    }
  });

  test("logging out ends the session", async ({ page, account }) => {
    await page.goto("/settings/account");
    await page.getByRole("link", { name: "Log out" }).click();
    await page.waitForURL("/");

    await page.goto("/recipes");
    await expect(page).toHaveURL(/\/login$/);

    // The token itself was revoked, not just the cookie.
    const response = await page.request.get("/api/v1/me", {
      headers: { authorization: `Bearer ${account.token}` },
    });
    expect(response.status()).toBe(401);
  });
});

test.describe("password reset", () => {
  test("the emailed link sets a new password", async ({ page, context, account }) => {
    test.skip(
      !(await mailAvailable(context.request)),
      "MailDev is not reachable (start it with `make up` or set MAILDEV_URL)",
    );
    await context.clearCookies();

    const reset = new ResetPasswordPage(page);
    await reset.request(account.email);
    await expect(page.getByRole("heading", { name: "Check Your Email" })).toBeVisible();

    const mail = await waitForMail(context.request, account.email);
    const newPassword = "brand-new-pass-7";
    await reset.choose(resetPasswordPath(mail), newPassword);
    await expect(page.getByRole("heading", { name: "Password Reset Successful" })).toBeVisible();

    const old = await page.request.post("/api/v1/auth/login", {
      data: { email: account.email, password: account.password },
    });
    expect(old.ok()).toBe(false);
    await login(page.request, account.email, newPassword);

    const form = new LoginPage(page);
    await form.goto();
    await form.submit(account.email, newPassword);
    await page.waitForURL("/");
  });

  test("mismatched passwords on the reset form are rejected", async ({ page, context }) => {
    await context.clearCookies();
    const reset = new ResetPasswordPage(page);
    await reset.choose("/reset-password/new/not-a-real-request", "brand-new-pass-7", "other-pass-7");
    await expect(toast(page)).toContainText("Passwords don't match");
  });
});
