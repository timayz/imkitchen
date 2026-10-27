import type { Page } from "@playwright/test";

/** Toasts are appended to `#toast-container` on every page. */
export function toast(page: Page) {
  return page.locator("#toast-container");
}

export class LoginPage {
  constructor(private readonly page: Page) {}

  async goto() {
    await this.page.goto("/login");
  }

  async submit(email: string, password: string) {
    await this.page.getByLabel("Email Address").fill(email);
    await this.page.getByLabel("Password").fill(password);
    await this.page.getByRole("button", { name: "Log In" }).click();
  }
}

export class RegisterPage {
  constructor(private readonly page: Page) {}

  async goto() {
    await this.page.goto("/register");
  }

  async submit(email: string, password: string, confirm = password) {
    await this.page.getByLabel("Email Address").fill(email);
    await this.page.getByLabel("Password", { exact: true }).fill(password);
    await this.page.getByLabel("Confirm Password").fill(confirm);
    await this.page.getByRole("button", { name: "Create Account" }).click();
  }
}

export class ResetPasswordPage {
  constructor(private readonly page: Page) {}

  async request(email: string) {
    await this.page.goto("/reset-password");
    await this.page.getByLabel("Email Address").fill(email);
    await this.page.getByRole("button", { name: "Send Reset Link" }).click();
  }

  async choose(path: string, password: string, confirm = password) {
    await this.page.goto(path);
    await this.page.getByLabel("New Password").fill(password);
    await this.page.getByLabel("Confirm Password").fill(confirm);
    await this.page.getByRole("button", { name: "Reset Password" }).click();
  }
}
