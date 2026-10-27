import type { Locator, Page } from "@playwright/test";

export class AdminUsersPage {
  readonly list: Locator;
  readonly search: Locator;

  constructor(private readonly page: Page) {
    this.list = page.locator("#users-list");
    this.search = page.locator('input[name="search"]');
  }

  async goto() {
    await this.page.goto("/admin/users");
  }

  row(userId: string) {
    return this.list.locator(`#user-${userId}`);
  }

  rows() {
    return this.list.locator("tbody tr[id^='user-']");
  }

  async filterRole(role: "" | "User" | "Chef" | "Admin") {
    await this.page.locator('select[name="role"]').selectOption(role);
  }

  async filterState(state: string) {
    await this.page.locator('select[name="state"]').selectOption(state);
  }

  async setRole(userId: string, role: "User" | "Chef" | "Admin") {
    await this.row(userId).getByRole("button", { name: "Edit" }).click();
    const modal = this.page.locator("#admin-user-edit-modal");
    await modal.waitFor();
    await modal.locator('select[name="role"]').selectOption(role);
    await modal.getByRole("button", { name: "Save" }).click();
    await modal.waitFor({ state: "detached" });
  }
}
