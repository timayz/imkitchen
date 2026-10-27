import type { Locator, Page } from "@playwright/test";

export class KitchenPage {
  readonly title: Locator;
  readonly card: Locator;
  readonly hero: Locator;
  readonly generateModal: Locator;

  constructor(private readonly page: Page) {
    this.title = page.locator('[data-tour~="kitchen-title"]');
    this.card = page.locator("#current-recipe-card");
    this.hero = page.locator('[data-tour~="kitchen-hero"]');
    this.generateModal = page.locator("#modal-generate");
  }

  async goto() {
    await this.page.goto("/");
  }

  /** The generate / regenerate button (one is visible per viewport). */
  generateButton() {
    return this.page.locator('[data-tour~="kitchen-cta"]:visible').first();
  }

  async openGenerate() {
    await this.generateButton().click();
    await this.generateModal.waitFor();
  }

  async generate(count: 3 | 7 | 14 | 30) {
    await this.openGenerate();
    await this.generateModal.locator(`.gen-preset[data-count="${count}"]`).click();
    await this.generateModal.getByRole("button", { name: "Generate" }).click();
    await this.generateModal.waitFor({ state: "detached" });
  }

  /** Every non-focused recipe row ("Focus" / "Cooked" / "Cooking"). */
  rows() {
    return this.card.locator('button[ts-req$="/select-dish"]');
  }

  row(name: string) {
    return this.rows().filter({ hasText: name });
  }

  removeButtons() {
    return this.card.getByTitle("Remove from list");
  }

  async startCooking() {
    await this.page.locator('[data-tour~="kitchen-start"]').click();
    await this.page.waitForURL(/\/kitchen\/[^/]+\/cook$/);
  }
}

export class CookingPage {
  readonly screen: Locator;

  constructor(private readonly page: Page) {
    this.screen = page.locator("#cooking-screen");
  }

  async start() {
    await this.screen.getByRole("button", { name: "Start cooking" }).click();
  }

  async next() {
    await this.screen.getByRole("button", { name: "Next step" }).click();
  }

  async back() {
    await this.screen.getByRole("button", { name: "Back" }).click();
  }

  done() {
    return this.screen.getByRole("link", { name: "Done! Serve it up" });
  }
}
