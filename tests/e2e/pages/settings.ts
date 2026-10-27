import type { Page } from "@playwright/test";

export class SettingsGeneralPage {
  constructor(private readonly page: Page) {}

  async goto() {
    await this.page.goto("/settings/general");
  }

  profileForm() {
    return this.page.locator('form[action="/settings/general/profile"]');
  }

  preferencesForm() {
    return this.page.locator('form[action="/settings/general"]');
  }

  async saveProfile(description: string) {
    const form = this.profileForm();
    await form.locator('textarea[name="description"]').fill(description);
    await form.getByRole("button", { name: "Save" }).click();
  }

  async saveHouseholdSize(size: number) {
    const form = this.preferencesForm();
    await form.locator('input[name="household_size"]').fill(String(size));
    await form.getByRole("button", { name: "Save" }).click();
  }

  async replayTours() {
    await this.page
      .locator('form[action="/settings/general/tours/reset"]')
      .getByRole("button")
      .click();
  }
}
