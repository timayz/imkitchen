import { test, expect } from "./fixtures";
import { CookingPage, KitchenPage } from "./pages/kitchen";
import { uniqueName } from "./support/api";

/**
 * The kitchen is the home page: the list of recipes to cook next (what the
 * issue called the meal plan), generated from the library or filled by hand.
 */
test.describe("kitchen", () => {
  test("a fresh account is pointed at the library, then at generation", async ({ page, api }) => {
    const kitchen = new KitchenPage(page);
    await kitchen.goto();
    await expect(page.getByRole("heading", { name: "Your list is empty" })).toBeVisible();
    await expect(page.locator('[data-tour~="kitchen-cta"]')).toHaveAttribute("href", "/recipes");

    await api.createRecipe({ name: uniqueName("First stew") });
    await expect
      .poll(async () => {
        await kitchen.goto();
        return page.getByRole("heading", { name: "Your collection is ready" }).isVisible();
      })
      .toBe(true);
    await expect(kitchen.generateButton()).toBeVisible();
  });

  test("generating fills the list from the library @mobile", async ({ page, api }) => {
    const names = [uniqueName("Gen curry"), uniqueName("Gen pasta"), uniqueName("Gen tacos")];
    for (const name of names) await api.createRecipe({ name });
    await api.waitForKitchen((overview) => overview.has_recipes === true);

    const kitchen = new KitchenPage(page);
    await kitchen.goto();
    await kitchen.generate(3);

    await expect(page.getByRole("heading", { name: "Up next" })).toBeVisible();
    await expect(kitchen.hero).toBeVisible();
    await expect(page.getByText(/0\/3 cooked/)).toBeVisible();
    // One recipe is focused in the hero, the other two are rows.
    await expect(kitchen.rows()).toHaveCount(2);
    for (const name of names) await expect(kitchen.card).toContainText(name);
  });

  test("the focused recipe is switched and recipes are removed by hand", async ({ page, api }) => {
    const first = await api.createRecipe({ name: uniqueName("Row one") });
    const second = await api.createRecipe({ name: uniqueName("Row two") });
    await api.addToList(first.id);
    await api.addToList(second.id);

    const kitchen = new KitchenPage(page);
    await kitchen.goto();
    await expect(kitchen.rows()).toHaveCount(1);
    const focused = (await kitchen.hero.locator("h2").textContent())?.trim();
    const other = focused === first.name ? second : first;

    await kitchen.row(other.name).click();
    await expect(kitchen.hero.locator("h2")).toHaveText(other.name);

    await kitchen.removeButtons().first().click();
    await expect(kitchen.rows()).toHaveCount(0);
    await expect(page.getByText(/0\/1 cooked/)).toBeVisible();

    await kitchen.removeButtons().first().click();
    await expect(page.getByRole("heading", { name: "Your collection is ready" })).toBeVisible();
  });

  test("regenerating replaces the current list", async ({ page, api }) => {
    for (const name of ["Regen a", "Regen b", "Regen c", "Regen d"]) {
      await api.createRecipe({ name: uniqueName(name) });
    }
    await api.generate(2);

    const kitchen = new KitchenPage(page);
    await kitchen.goto();
    await expect(page.getByText(/0\/2 cooked/)).toBeVisible();

    await kitchen.generate(3);
    await expect(page.getByText(/0\/3 cooked/)).toBeVisible();
    await expect(kitchen.rows()).toHaveCount(2);
  });

  test("cooking walks through the steps and marks the recipe done @mobile", async ({ page, api }) => {
    const recipe = await api.createRecipe({
      name: uniqueName("Stepwise omelette"),
      instructions: [
        { description: "Beat the eggs.", time_next: 0 },
        { description: "Pour into the pan.", time_next: 2 },
        { description: "Fold and serve.", time_next: 0 },
      ],
    });
    await api.addToList(recipe.id);

    const kitchen = new KitchenPage(page);
    await kitchen.goto();
    await kitchen.startCooking();

    const cooking = new CookingPage(page);
    await cooking.start();
    await expect(cooking.screen).toContainText("Beat the eggs.");
    await expect(cooking.screen).toContainText(/step 1 of 3/i);

    await cooking.next();
    await expect(cooking.screen).toContainText("Pour into the pan.");
    await cooking.back();
    await expect(cooking.screen).toContainText("Beat the eggs.");

    await cooking.next();
    await cooking.next();
    await expect(cooking.screen).toContainText("Fold and serve.");
    await expect(cooking.screen).toContainText(/final step/i);
    await cooking.done().click();

    await page.waitForURL("/");
    await expect(page.getByText("Everything is cooked")).toBeVisible();
  });
});
