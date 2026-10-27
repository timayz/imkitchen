import { test, expect } from "./fixtures";
import { toast } from "./pages/auth";
import { SettingsGeneralPage } from "./pages/settings";
import { mailAvailable, waitForMail } from "./support/mail";

test.describe("settings", () => {
  test("the profile description is saved", async ({ page, account }) => {
    const settings = new SettingsGeneralPage(page);
    await settings.goto();
    await settings.saveProfile("Weeknight cook, big on one-pot dinners.");
    await expect(toast(page)).toBeVisible();

    await page.reload();
    await expect(settings.profileForm().locator('textarea[name="description"]')).toHaveValue(
      "Weeknight cook, big on one-pot dinners.",
    );
    void account;
  });

  test("meal preferences are saved", async ({ page, account }) => {
    const settings = new SettingsGeneralPage(page);
    await settings.goto();
    await settings.saveHouseholdSize(6);
    await expect(toast(page)).toBeVisible();

    await page.reload();
    await expect(settings.preferencesForm().locator('input[name="household_size"]')).toHaveValue("6");
    void account;
  });

  test("replaying the tours brings the kitchen tour back", async ({ page, account }) => {
    const settings = new SettingsGeneralPage(page);
    await settings.goto();
    await settings.replayTours();

    await page.goto("/");
    await expect(page.locator("#tour-root")).toBeVisible();
    await page.locator("#tour-root [data-tour-skip]").click();
    await expect(page.locator("#tour-root")).toHaveCount(0);
    void account;
  });

  test("the account page emails a password reset link", async ({ page, context, account }) => {
    test.skip(
      !(await mailAvailable(context.request)),
      "MailDev is not reachable (start it with `make up` or set MAILDEV_URL)",
    );
    await page.goto("/settings/account");
    await page.getByRole("button", { name: "Send reset link" }).click();
    await expect(toast(page)).toContainText("emailed you a secure link");

    const mail = await waitForMail(context.request, account.email);
    expect(mail.text ?? mail.html).toMatch(/\/reset-password\/new\//);
  });

  test("the premium upgrade page offers the plans when monetization is on", async ({ page, api }) => {
    const me = await api.me();
    test.skip(!me.premium_enabled, "monetization is off ([premium] config absent): every account is premium");

    await page.goto("/");
    await expect(page.getByRole("button", { name: "Get Premium" }).first()).toBeVisible();

    await page.goto("/upgrade");
    await expect(page.getByRole("heading", { name: "Select Your Plan" })).toBeVisible();
    await expect(page.locator('input[name="plan"][value="annual"]')).toBeChecked();
    await page.locator('input[name="plan"][value="monthly"]').check({ force: true });
    // Payment needs a card: the button stays disabled until Stripe is ready.
    await expect(page.getByRole("button", { name: "Pay Now" }).first()).toBeDisabled();
  });
});
