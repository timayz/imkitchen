import type { Locator, Page } from "@playwright/test";

export class DishesPage {
  readonly cards: Locator;

  constructor(private readonly page: Page) {
    this.cards = page.locator("article");
  }

  async goto() {
    await this.page.goto("/dishes");
  }

  card(name: string) {
    return this.cards.filter({ has: this.page.locator("h2", { hasText: name }) });
  }

  async compose() {
    await this.page.getByRole("link", { name: "Compose a dish" }).first().click();
    await this.page.waitForURL(/\/dishes\/new$/);
  }
}

export class DishEditorPage {
  readonly form: Locator;

  constructor(private readonly page: Page) {
    this.form = page.locator("#dish-form");
  }

  async name(name: string) {
    await this.form.locator("#name").fill(name);
  }

  /** Course options are visually-hidden radios / checkboxes inside labels. */
  async pick(recipeId: string) {
    await this.form.locator(`input[value="${recipeId}"]`).check({ force: true });
  }

  async save() {
    await this.page.getByRole("button", { name: "Save" }).first().click();
    await this.page.waitForURL(/\/dishes$/);
  }
}
