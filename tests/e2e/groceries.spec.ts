import { test, expect } from "./fixtures";
import { GroceriesPage } from "./pages/groceries";
import { uniqueName } from "./support/api";

test.describe("groceries", () => {
  test("an empty list says there is nothing to buy", async ({ page, account }) => {
    const groceries = new GroceriesPage(page);
    await groceries.goto();
    await expect(page.getByRole("heading", { name: "Nothing to buy yet" })).toBeVisible();
    void account;
  });

  test("the list groups the ingredients of every listed recipe by aisle @mobile", async ({ page, api }) => {
    const curry = await api.createRecipe({
      name: uniqueName("Grocery curry"),
      ingredients: [
        { name: "Chickpeas", quantity: 400, unit: "G", category: "Grocery" },
        { name: "Spinach", quantity: 200, unit: "G", category: "FruitsAndVegetables" },
      ],
    });
    const cake = await api.createRecipe({
      name: uniqueName("Grocery cake"),
      recipe_type: "Dessert",
      ingredients: [
        { name: "Butter", quantity: 250, unit: "G", category: "DairyAndEggs" },
        { name: "Spinach", quantity: 100, unit: "G", category: "FruitsAndVegetables" },
      ],
    });
    await api.addToList(curry.id);
    await api.addToList(cake.id);
    await api.waitForGroceries((list) => (list.total_items as number) >= 3);

    const groceries = new GroceriesPage(page);
    await groceries.goto();
    // On phones every aisle but the current one is folded, so count rather
    // than look.
    await expect(groceries.aisles()).toHaveCount(3);
    await expect(groceries.item("Chickpeas")).toHaveCount(1);
    await expect(groceries.item("Butter")).toHaveCount(1);
    // Same ingredient from two recipes is one line with the summed quantity,
    // scaled from the recipes' 2 servings to the default household of 4.
    await expect(groceries.item("Spinach")).toHaveCount(1);
    await expect(groceries.item("Spinach")).toContainText("600 g");
    await expect(groceries.counter).toContainText("0/3 items");
    await expect(groceries.counter).toContainText("2 recipes");
  });

  test("checking items persists across reloads and can be undone @mobile", async ({ page, api }) => {
    const recipe = await api.createRecipe({
      name: uniqueName("Tick soup"),
      ingredients: [
        { name: "Leek", quantity: 2, unit: null, category: "FruitsAndVegetables" },
        { name: "Potato", quantity: 500, unit: "G", category: "FruitsAndVegetables" },
      ],
    });
    await api.addToList(recipe.id);
    await api.waitForGroceries((list) => (list.total_items as number) >= 2);

    const groceries = new GroceriesPage(page);
    await groceries.goto();
    await expect(groceries.counter).toContainText("0/2 items");

    await groceries.toggle("Leek");
    await expect(groceries.checkbox("Leek")).toBeChecked();
    await expect
      .poll(async () => ((await api.groceries()).checked_items as number))
      .toBe(1);

    await page.reload();
    await expect(groceries.checkbox("Leek")).toBeChecked();
    await expect(groceries.checkbox("Potato")).not.toBeChecked();
    await expect(groceries.counter).toContainText("1/2 items");

    await groceries.toggle("Leek");
    await expect
      .poll(async () => ((await api.groceries()).checked_items as number))
      .toBe(0);
    await page.reload();
    await expect(groceries.checkbox("Leek")).not.toBeChecked();
    await expect(groceries.counter).toContainText("0/2 items");
  });

  test("removing the last recipe from the kitchen empties the list", async ({ page, api }) => {
    const recipe = await api.createRecipe({ name: uniqueName("Short-lived risotto") });
    await api.addToList(recipe.id);
    await api.waitForGroceries((list) => (list.total_items as number) >= 1);

    await page.goto("/");
    await page.locator("#current-recipe-card").getByTitle("Remove from list").first().click();
    await expect(page.getByRole("heading", { name: "Your collection is ready" })).toBeVisible();

    const groceries = new GroceriesPage(page);
    await expect
      .poll(async () => {
        await groceries.goto();
        return page.getByRole("heading", { name: "Nothing to buy yet" }).isVisible();
      })
      .toBe(true);
  });
});
