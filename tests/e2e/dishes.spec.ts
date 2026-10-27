import { test, expect } from "./fixtures";
import { DishEditorPage, DishesPage } from "./pages/dishes";
import { uniqueName } from "./support/api";

/**
 * A dish is a composed meal (one main course plus sides, sauces, a starter,
 * dessert or drink). Generation picks dishes first, and a dish can be put on
 * the list by hand.
 */
test.describe("dishes", () => {
  test("without a main course the composer is not offered", async ({ page, account }) => {
    const dishes = new DishesPage(page);
    await dishes.goto();
    await expect(page.getByRole("heading", { name: "No dishes yet" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Add a main course" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Compose a dish" })).toHaveCount(0);
    void account;
  });

  test("a dish is composed, added to the list and deleted", async ({ page, api }) => {
    const main = await api.createRecipe({ name: uniqueName("Dish roast") });
    const side = await api.createRecipe({ name: uniqueName("Dish potatoes"), recipe_type: "Accompaniment" });
    const dessert = await api.createRecipe({ name: uniqueName("Dish crumble"), recipe_type: "Dessert" });

    const dishes = new DishesPage(page);
    await dishes.goto();
    await dishes.compose();

    const editor = new DishEditorPage(page);
    const name = uniqueName("Sunday roast");
    await editor.name(name);
    await editor.pick(main.id);
    await editor.pick(side.id);
    await editor.pick(dessert.id);
    await editor.save();

    const card = dishes.card(name);
    await expect(card).toBeVisible();
    await expect(card).toContainText("3 recipes");
    await expect(card).toContainText(main.name);
    await expect(card).toContainText(side.name);
    await expect(card).toContainText(dessert.name);

    // On the list by hand: every recipe of the dish lands in the kitchen.
    await card.getByRole("button", { name: "Add to my list" }).click();
    await expect(card).toContainText("In my list");
    await api.waitForKitchen((overview) => overview.kind === "list" && overview.total_count === 3);

    await page.goto("/");
    for (const recipe of [main, side, dessert]) {
      await expect(page.locator("#current-recipe-card")).toContainText(recipe.name);
    }

    // Deleting the dish keeps its recipes in the library.
    await dishes.goto();
    page.once("dialog", (dialog) => dialog.accept());
    await card.getByRole("button", { name: "Delete" }).click();
    await expect(page.getByRole("heading", { name: "No dishes yet" })).toBeVisible();
    await page.goto(`/r/${main.slug}`);
    await expect(page.locator("h1")).toHaveText(main.name);
  });

  test("the composer requires a dish name", async ({ page, api }) => {
    const main = await api.createRecipe({ name: uniqueName("Nameless main") });
    await page.goto("/dishes/new");
    const editor = new DishEditorPage(page);
    await editor.pick(main.id);
    await page.getByRole("button", { name: "Save" }).first().click();
    await expect(page).toHaveURL(/\/dishes\/new$/);
    expect(await editor.form.locator("#name").evaluate((el: HTMLInputElement) => el.validity.valid)).toBe(false);
  });
});
