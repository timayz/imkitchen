import { expect, type Locator, type Page } from "@playwright/test";

export class RecipesPage {
  readonly list: Locator;
  readonly search: Locator;

  constructor(private readonly page: Page) {
    this.list = page.locator("#recipes-list");
    this.search = page.locator('input[name="search"]');
  }

  async goto(query: Record<string, string> = { mine: "true" }) {
    const params = new URLSearchParams(query).toString();
    await this.page.goto(`/recipes${params ? `?${params}` : ""}`);
  }

  card(recipeId: string) {
    return this.list.locator(`#recipe-${recipeId}`);
  }

  cards() {
    return this.list.locator("a[id^='recipe-']");
  }

  /**
   * The course chips are visually-hidden radios inside labels. The "All"
   * chip is the first one: the minifier strips its empty `value`.
   */
  async filterType(type: "" | "Appetizer" | "MainCourse" | "Accompaniment" | "Dessert" | "Beverage" | "Condiment") {
    const radios = this.page.locator('input[name="recipe_type"]');
    const radio = type === "" ? radios.first() : radios.and(this.page.locator(`[value="${type}"]`));
    await radio.check({ force: true });
  }

  async toggleMine() {
    await this.page.locator('input[name="mine"]').click({ force: true });
  }

  async startNewRecipe() {
    await this.page.getByRole("button", { name: "New recipe" }).click();
    await this.page.waitForURL(/\/recipes\/[^/]+\/edit$/);
  }
}

export type EditorInput = {
  name: string;
  description: string;
  type?: "Appetizer" | "MainCourse" | "Accompaniment" | "Dessert" | "Beverage" | "Condiment";
  prepTime?: number;
  cookTime?: number;
  serves?: number;
  ingredient?: { name: string; quantity: number; unit?: string; category?: string };
  step?: string;
};

export class RecipeEditorPage {
  readonly form: Locator;

  constructor(private readonly page: Page) {
    this.form = page.locator("#recipe-form");
  }

  async goto(recipeId: string) {
    await this.page.goto(`/recipes/${recipeId}/edit`);
  }

  /** Fills the fixed fields and the first ingredient / step rows. */
  async fill(input: EditorInput) {
    await this.form.locator("#name").fill(input.name);
    await this.form.locator("#description").fill(input.description);
    if (input.type) {
      await this.form.locator(`input[name="recipe_type"][value="${input.type}"]`).check({ force: true });
    }
    if (input.prepTime !== undefined) await this.form.locator("#prep_time").fill(String(input.prepTime));
    if (input.cookTime !== undefined) await this.form.locator("#cook_time").fill(String(input.cookTime));
    if (input.serves !== undefined) await this.form.locator("#household_size").fill(String(input.serves));
    if (input.ingredient) {
      // A new recipe starts without rows.
      if ((await this.form.locator(".ingredient-item").count()) === 0) await this.addIngredientRow();
      const row = this.form.locator(".ingredient-item").first();
      await row.locator('input[name="ingredients_quantity"]').fill(String(input.ingredient.quantity));
      await row.locator('input[name="ingredients_name"]').fill(input.ingredient.name);
      if (input.ingredient.unit !== undefined) {
        await row.locator('select[name="ingredients_unit"]').selectOption(input.ingredient.unit);
      }
      if (input.ingredient.category !== undefined) {
        await row.locator('select[name="ingredients_category"]').selectOption(input.ingredient.category);
      }
    }
    if (input.step) {
      if ((await this.form.locator(".instruction-item").count()) === 0) await this.addStepRow();
      await this.form.locator(".instruction-item textarea").first().fill(input.step);
    }
  }

  async addIngredientRow() {
    const rows = this.form.locator(".ingredient-item");
    const before = await rows.count();
    await this.page.getByRole("button", { name: "Add Ingredient" }).click();
    await rows.nth(before).waitFor();
    return rows.nth(before);
  }

  async addStepRow() {
    const rows = this.form.locator(".instruction-item");
    const before = await rows.count();
    await this.page.getByRole("button", { name: "Add Step" }).click();
    await rows.nth(before).waitFor();
    return rows.nth(before);
  }

  /** Saves (the editor stays open and toasts), then opens the detail page. */
  async save() {
    await this.page.getByRole("button", { name: "Save Recipe" }).click();
    await expect(this.page.locator("#toast-container")).toContainText("Recipe saved");
    const id = this.page.url().match(/\/recipes\/([^/]+)\/edit/)?.[1];
    if (!id) throw new Error(`not on an editor page: ${this.page.url()}`);
    // `/recipes/{id}` redirects to the slug URL once the projection has it.
    await this.page.goto(`/recipes/${id}`);
    await this.page.waitForURL(/\/r\/[^/?]+$/);
  }
}

export class RecipeDetailPage {
  readonly title: Locator;
  readonly addToList: Locator;
  readonly save: Locator;

  constructor(private readonly page: Page) {
    this.title = page.locator("h1");
    this.addToList = page.locator("#add-to-shopping-btn");
    this.save = page.locator("#save-btn");
  }

  async goto(slug: string) {
    await this.page.goto(`/r/${slug}`);
  }

  async edit() {
    await this.page.getByRole("link", { name: "Edit" }).click();
    await this.page.waitForURL(/\/recipes\/[^/]+\/edit$/);
  }

  async makePublic() {
    await this.page.locator("#share-button").getByRole("button", { name: "Make Public" }).click();
  }

  async makePrivate() {
    await this.page.locator("#share-button").getByRole("button", { name: "Make Private" }).click();
  }

  /** Opens the confirmation modal and confirms; the page then polls the deletion. */
  async delete() {
    await this.page.getByRole("button", { name: "Delete" }).click();
    const modal = this.page.locator("#delete-confirm");
    await modal.waitFor();
    await modal.getByRole("button", { name: "Delete" }).click();
    await this.page.waitForURL(/\/recipes(\?.*)?$/);
  }
}
