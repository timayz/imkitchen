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

  sourceToggle(name: "dishes" | "main_courses") {
    return this.preferencesForm().locator(`input[name="${name}"]`);
  }

  /** Flip a source toggle through its label; the input itself is visually hidden. */
  async flipSource(name: "dishes" | "main_courses") {
    await this.preferencesForm()
      .locator(`label:has(input[name="${name}"])`)
      .click();
  }

  async savePreferences() {
    await this.preferencesForm().getByRole("button", { name: "Save" }).click();
  }

  async replayTours() {
    await this.page
      .locator('form[action="/settings/general/tours/reset"]')
      .getByRole("button")
      .click();
  }
}
