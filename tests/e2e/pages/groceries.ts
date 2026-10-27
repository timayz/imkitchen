import type { Locator, Page } from "@playwright/test";

export class GroceriesPage {
  readonly body: Locator;
  readonly counter: Locator;

  constructor(private readonly page: Page) {
    this.body = page.locator("#groceries-body");
    this.counter = page.locator("header p");
  }

  async goto() {
    await this.page.goto("/groceries");
  }

  aisles() {
    return this.body.locator('section[id^="category-"]');
  }

  items() {
    return this.body.locator('[data-tour~="groceries-item"]');
  }

  item(name: string) {
    return this.items().filter({ hasText: name });
  }

  /** The real checkbox is visually hidden behind a styled box: toggle via the label. */
  checkbox(name: string) {
    return this.item(name).locator('input[type="checkbox"]');
  }

  async toggle(name: string) {
    await this.item(name).click();
  }
}
