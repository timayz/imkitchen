import { test, expect } from "./fixtures";
import { AdminUsersPage } from "./pages/admin";
import { RecipeDetailPage, RecipeEditorPage, RecipesPage } from "./pages/recipes";
import { PASSWORD, installSession, register, uniqueEmail, uniqueName, Api } from "./support/api";

test.describe("recipe management", () => {
  test("a recipe is created in the editor and shown on its detail page", async ({ page, account }) => {
    const recipes = new RecipesPage(page);
    await recipes.goto();
    await recipes.startNewRecipe();

    const name = uniqueName("Lemon chicken");
    const editor = new RecipeEditorPage(page);
    await editor.fill({
      name,
      description: "Roast chicken with lemon and thyme.",
      type: "MainCourse",
      prepTime: 15,
      cookTime: 45,
      serves: 4,
      ingredient: { name: "Chicken", quantity: 1200, unit: "G", category: "Butcher" },
      step: "Rub the chicken with lemon and roast it.",
    });
    await editor.save();

    const detail = new RecipeDetailPage(page);
    await expect(detail.title).toHaveText(name);
    await expect(page.getByRole("heading", { name: "Ingredients" })).toBeVisible();
    await expect(page.getByText("Chicken", { exact: false }).first()).toBeVisible();
    await expect(page.getByRole("heading", { name: "Method" })).toBeVisible();
    await expect(page.getByText("Rub the chicken with lemon and roast it.")).toBeVisible();

    // The library lists it once the projection catches up.
    await expect
      .poll(async () => {
        await recipes.goto();
        return recipes.list.textContent();
      })
      .toContain(name);
    void account;
  });

  test("the editor rejects a recipe without a name", async ({ page, account }) => {
    const recipes = new RecipesPage(page);
    await recipes.goto();
    await recipes.startNewRecipe();

    const editor = new RecipeEditorPage(page);
    await editor.fill({ name: "", description: "Only a description." });
    await page.getByRole("button", { name: "Save Recipe" }).click();

    // Native validation keeps the form on the page.
    await expect(page).toHaveURL(/\/recipes\/[^/]+\/edit$/);
    const invalid = await editor.form.locator("#name").evaluate((el: HTMLInputElement) => el.validity.valid);
    expect(invalid).toBe(false);
    void account;
  });

  test("editing a recipe updates its detail page", async ({ page, api }) => {
    const recipe = await api.createRecipe({ name: uniqueName("Tomato soup") });

    const detail = new RecipeDetailPage(page);
    await detail.goto(recipe.slug);
    await detail.edit();

    const editor = new RecipeEditorPage(page);
    const renamed = uniqueName("Roasted tomato soup");
    await editor.form.locator("#name").fill(renamed);
    await editor.form.locator("#cook_time").fill("35");
    const row = await editor.addIngredientRow();
    await row.locator('input[name="ingredients_quantity"]').fill("1");
    await row.locator('select[name="ingredients_unit"]').selectOption("G");
    await row.locator('input[name="ingredients_name"]').fill("Basil");
    await row.locator('select[name="ingredients_category"]').selectOption("FruitsAndVegetables");
    await editor.save();

    await expect(detail.title).toHaveText(renamed);
    await expect(page.getByText("Basil")).toBeVisible();
  });

  test("deleting a recipe removes it from the library", async ({ page, api }) => {
    const recipe = await api.createRecipe({ name: uniqueName("Doomed pancakes") });

    const detail = new RecipeDetailPage(page);
    await detail.goto(recipe.slug);
    await detail.delete();

    const recipes = new RecipesPage(page);
    await expect(recipes.card(recipe.id)).toHaveCount(0);
    await page.goto(`/r/${recipe.slug}`);
    await expect(page.getByRole("heading", { name: "404" })).toBeVisible();
  });

  test("the library is searched and filtered live", async ({ page, api }) => {
    const pie = await api.createRecipe({ name: "Apple pie", recipe_type: "Dessert" });
    const chili = await api.createRecipe({ name: "Chili con carne", recipe_type: "MainCourse" });
    const soup = await api.createRecipe({ name: "Carrot soup", recipe_type: "Appetizer" });

    const recipes = new RecipesPage(page);
    await recipes.goto();
    await expect(recipes.cards()).toHaveCount(3);

    await recipes.search.fill("apple");
    await expect(recipes.card(pie.id)).toBeVisible();
    await expect(recipes.cards()).toHaveCount(1);

    await recipes.search.fill("");
    await recipes.filterType("MainCourse");
    await expect(recipes.card(chili.id)).toBeVisible();
    await expect(recipes.cards()).toHaveCount(1);

    await recipes.filterType("");
    await expect(recipes.cards()).toHaveCount(3);
    await expect(recipes.card(soup.id)).toBeVisible();

    await recipes.search.fill("nothing matches this");
    await expect(recipes.list).toContainText("No recipes found");
  });

  test("a chef shares a recipe with the community and another cook saves it", async ({
    page,
    browser,
    baseURL,
    account,
    api,
    admin,
  }) => {
    // Sharing needs the chef role (granted by an admin) and a username.
    const users = new AdminUsersPage(admin.page);
    await users.goto();
    await users.search.fill(account.email);
    await expect(users.row(account.id)).toBeVisible();
    await users.setRole(account.id, "Chef");
    await expect(users.row(account.id)).toContainText("Chef");

    const recipe = await api.createRecipe({ name: uniqueName("Shared paella") });
    const detail = new RecipeDetailPage(page);
    await detail.goto(recipe.slug);
    await detail.makePublic();

    // No username yet: the modal asks for one, then sharing goes through.
    const modal = page.locator("#modal-set-username");
    await modal.waitFor();
    await modal.locator('input[name="username"]').fill(`chef_${account.id.slice(-6).toLowerCase()}`);
    await modal.getByRole("button", { name: "Continue" }).click();
    await modal.waitFor({ state: "detached" });
    await detail.makePublic();
    await expect(page.locator("#share-button")).toContainText("Make Private");

    // Another cook sees it, saves it, and finds it under "Saved".
    const other = await browser.newContext();
    try {
      const session = await register(other.request, uniqueEmail("cook"), PASSWORD);
      await installSession(other, baseURL!, session.token);
      await new Api(other.request, session.token).skipTours();
      const otherPage = await other.newPage();
      const otherDetail = new RecipeDetailPage(otherPage);
      await expect
        .poll(
          async () => {
            await otherDetail.goto(recipe.slug);
            return otherDetail.title.textContent();
          },
          { timeout: 20_000 },
        )
        .toBe(recipe.name);
      await expect(otherDetail.save).toContainText("Save");
      await otherDetail.save.click();
      await expect(otherDetail.save).toContainText("Saved");
      await otherPage.reload();
      await expect(otherDetail.save).toContainText("Saved");

      // The community browse hides what is already saved; the "Saved"
      // filter is where it lives now (once the projection has it).
      const otherRecipes = new RecipesPage(otherPage);
      await expect
        .poll(
          async () => {
            await otherRecipes.goto({ in_meal_plan: "true" });
            return otherRecipes.card(recipe.id).isVisible();
          },
          { timeout: 20_000 },
        )
        .toBe(true);
      await otherRecipes.goto({ search: recipe.name });
      await expect(otherRecipes.list).toContainText("No recipes found");
    } finally {
      await other.close();
    }

    // Back to private: the other cook's page now answers 404.
    await detail.goto(recipe.slug);
    await detail.makePrivate();
    await expect(page.locator("#share-button")).toContainText("Make Public");
  });

  test("a recipe is added to the list from its detail page", async ({ page, api }) => {
    const recipe = await api.createRecipe({ name: uniqueName("Listed dal") });
    const detail = new RecipeDetailPage(page);
    await detail.goto(recipe.slug);

    await expect(detail.addToList).toContainText("Add to my list");
    await detail.addToList.click();
    await expect(detail.addToList).toContainText("In my list");

    await page.goto("/");
    await expect(page.locator("#current-recipe-card")).toContainText(recipe.name);
  });

  test("another cook's private recipe is not reachable", async ({ browser, api }) => {
    const recipe = await api.createRecipe({ name: uniqueName("Secret sauce") });
    const other = await browser.newContext();
    try {
      const session = await register(other.request, uniqueEmail("snoop"), PASSWORD);
      const response = await other.request.get(`/api/v1/recipes/${recipe.id}/edit`, {
        headers: { authorization: `Bearer ${session.token}` },
      });
      expect(response.status()).toBe(403);
    } finally {
      await other.close();
    }
  });
});
