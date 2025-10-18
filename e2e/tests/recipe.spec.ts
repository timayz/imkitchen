import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Recipe Creation Flow (Story 2.1 / Story 2.11 AC-2)
 *
 * Tests cover the full user journey from form submission to detail page display,
 * ensuring all recipe fields persist correctly and instruction reordering works.
 */

// Test helper to generate unique recipe titles
function uniqueRecipeTitle(base: string): string {
  return `${base} ${Date.now()}`;
}

test.describe('Recipe Creation Flow', () => {
  // Before each test, ensure we have a logged-in user
  test.beforeEach(async ({ page }) => {
    // Navigate to login page
    await page.goto('/login');

    // Login as test user (assumes test user exists or registration flow)
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for redirect to dashboard
    await page.waitForURL('/dashboard');
  });

  test('user can create recipe with all fields', async ({ page }) => {
    const recipeTitle = uniqueRecipeTitle('E2E Test Recipe');

    // Navigate to create recipe form
    await page.goto('/recipes/new');

    // Verify form loaded
    await expect(page.locator('h1')).toContainText('Create New Recipe');

    // Fill in recipe title
    await page.fill('input[name="title"]', recipeTitle);

    // Fill in first ingredient (default row exists)
    await page.fill('input[name="ingredient_name[]"]', 'Chicken');
    await page.fill('input[name="ingredient_quantity[]"]', '2');
    await page.selectOption('select[name="ingredient_unit[]"]', 'lb');

    // Add second ingredient
    await page.click('button:has-text("Add Ingredient")');
    const ingredientRows = page.locator('.ingredient-row');
    await expect(ingredientRows).toHaveCount(2);

    await page.fill('input[name="ingredient_name[]"]:nth-of-type(2)', 'Salt');
    await page.fill('input[name="ingredient_quantity[]"]:nth-of-type(2)', '1');
    await page.selectOption('select[name="ingredient_unit[]"]:nth-of-type(2)', 'tsp');

    // Fill in first instruction (default row exists)
    await page.fill('textarea[name="instruction_text[]"]', 'Marinate chicken overnight');
    await page.fill('input[name="instruction_timer[]"]', '');

    // Add second instruction
    await page.click('button:has-text("Add Step")');
    const instructionRows = page.locator('.instruction-row');
    await expect(instructionRows).toHaveCount(2);

    await page.fill('textarea[name="instruction_text[]"]:nth-of-type(2)', 'Cook chicken for 30 minutes');
    await page.fill('input[name="instruction_timer[]"]:nth-of-type(2)', '30');

    // Fill in timing information
    await page.fill('input[name="prep_time_min"]', '20');
    await page.fill('input[name="cook_time_min"]', '30');
    await page.fill('input[name="advance_prep_hours"]', '24');
    await page.fill('input[name="serving_size"]', '4');

    // Submit form
    await page.click('button[type="submit"]:has-text("Create Recipe")');

    // Wait for redirect to recipe detail page
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Verify recipe detail page displays correct data
    await expect(page.locator('h1')).toContainText(recipeTitle);

    // Verify ingredients displayed
    await expect(page.locator('text=Chicken')).toBeVisible();
    await expect(page.locator('text=2 lb')).toBeVisible();
    await expect(page.locator('text=Salt')).toBeVisible();
    await expect(page.locator('text=1 tsp')).toBeVisible();

    // Verify instructions displayed
    await expect(page.locator('text=Marinate chicken overnight')).toBeVisible();
    await expect(page.locator('text=Cook chicken for 30 minutes')).toBeVisible();
    await expect(page.locator('text=30 min')).toBeVisible(); // Timer indicator

    // Verify timing information displayed
    await expect(page.locator('text=20 min')).toBeVisible(); // Prep time
    await expect(page.locator('text=30 min')).toBeVisible(); // Cook time (may match timer)
    await expect(page.locator('text=24 hours')).toBeVisible(); // Advance prep
    await expect(page.locator('text=4')).toBeVisible(); // Serving size
  });

  test('recipe creation validates required fields', async ({ page }) => {
    // Navigate to create recipe form
    await page.goto('/recipes/new');

    // Try to submit form without filling required fields
    await page.click('button[type="submit"]:has-text("Create Recipe")');

    // Verify validation errors appear (browser HTML5 validation or server-side errors)
    // HTML5 validation will prevent submission, so check for validation state
    const titleInput = page.locator('input[name="title"]');
    await expect(titleInput).toHaveAttribute('required');

    // Fill only title (missing ingredients and instructions)
    await page.fill('input[name="title"]', 'Test Recipe');

    // Clear default ingredient row to test validation
    await page.fill('input[name="ingredient_name[]"]', '');
    await page.fill('textarea[name="instruction_text[]"]', '');

    // Submit form
    await page.click('button[type="submit"]:has-text("Create Recipe")');

    // Verify still on form page (validation failed)
    // Note: This test assumes server-side validation returns error page
    // If HTML5 validation prevents submission, we'd need to check :invalid state
  });

  test('created recipe displays on detail page', async ({ page }) => {
    const recipeTitle = uniqueRecipeTitle('Simple E2E Recipe');

    // Navigate to create recipe form
    await page.goto('/recipes/new');

    // Fill in minimal required fields
    await page.fill('input[name="title"]', recipeTitle);
    await page.fill('input[name="ingredient_name[]"]', 'Eggs');
    await page.fill('input[name="ingredient_quantity[]"]', '3');
    await page.selectOption('select[name="ingredient_unit[]"]', 'piece');
    await page.fill('textarea[name="instruction_text[]"]', 'Crack eggs into bowl');
    await page.fill('input[name="prep_time_min"]', '5');

    // Submit form
    await page.click('button[type="submit"]:has-text("Create Recipe")');

    // Wait for redirect
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Verify recipe displays on detail page
    await expect(page.locator('h1')).toContainText(recipeTitle);
    await expect(page.locator('text=Eggs')).toBeVisible();
    await expect(page.locator('text=Crack eggs into bowl')).toBeVisible();

    // Verify recipe persists: navigate away and back
    const recipeUrl = page.url();
    await page.goto('/dashboard');
    await page.goto(recipeUrl);

    // Verify data still displays
    await expect(page.locator('h1')).toContainText(recipeTitle);
    await expect(page.locator('text=Eggs')).toBeVisible();
  });

  test('user can reorder instruction steps', async ({ page }) => {
    const recipeTitle = uniqueRecipeTitle('Reorder Test Recipe');

    // Navigate to create recipe form
    await page.goto('/recipes/new');

    // Fill in recipe with multiple instructions
    await page.fill('input[name="title"]', recipeTitle);
    await page.fill('input[name="ingredient_name[]"]', 'Flour');
    await page.fill('input[name="ingredient_quantity[]"]', '2');
    await page.selectOption('select[name="ingredient_unit[]"]', 'cups');

    // Add 3 instruction steps
    await page.fill('textarea[name="instruction_text[]"]', 'Step 1: Mix ingredients');

    await page.click('button:has-text("Add Step")');
    await page.fill('textarea[name="instruction_text[]"]:nth-of-type(2)', 'Step 2: Knead dough');

    await page.click('button:has-text("Add Step")');
    await page.fill('textarea[name="instruction_text[]"]:nth-of-type(3)', 'Step 3: Bake at 350F');

    // Verify initial order (step numbers)
    const stepNumbers = page.locator('.instruction-number');
    await expect(stepNumbers.nth(0)).toHaveText('1');
    await expect(stepNumbers.nth(1)).toHaveText('2');
    await expect(stepNumbers.nth(2)).toHaveText('3');

    // Click "Move down" button on step 1 (swap with step 2)
    const firstRow = page.locator('.instruction-row').nth(0);
    await firstRow.locator('button:has-text("↓")').click();

    // Verify step numbers updated dynamically
    await expect(stepNumbers.nth(0)).toHaveText('1');
    await expect(stepNumbers.nth(1)).toHaveText('2');
    await expect(stepNumbers.nth(2)).toHaveText('3');

    // Verify instruction text order changed
    const instructions = page.locator('textarea[name="instruction_text[]"]');
    await expect(instructions.nth(0)).toHaveValue('Step 2: Knead dough');
    await expect(instructions.nth(1)).toHaveValue('Step 1: Mix ingredients');
    await expect(instructions.nth(2)).toHaveValue('Step 3: Bake at 350F');

    // Submit form and verify order persists
    await page.click('button[type="submit"]:has-text("Create Recipe")');
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Check instructions display in correct order on detail page
    const detailInstructions = page.locator('.instruction-step');
    await expect(detailInstructions.nth(0)).toContainText('Step 2: Knead dough');
    await expect(detailInstructions.nth(1)).toContainText('Step 1: Mix ingredients');
    await expect(detailInstructions.nth(2)).toContainText('Step 3: Bake at 350F');
  });

  test('user can edit existing recipe', async ({ page }) => {
    const originalTitle = uniqueRecipeTitle('Original Recipe');
    const updatedTitle = uniqueRecipeTitle('Updated Recipe');

    // Create recipe first
    await page.goto('/recipes/new');
    await page.fill('input[name="title"]', originalTitle);
    await page.fill('input[name="ingredient_name[]"]', 'Sugar');
    await page.fill('input[name="ingredient_quantity[]"]', '1');
    await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
    await page.fill('textarea[name="instruction_text[]"]', 'Original instruction');
    await page.fill('input[name="prep_time_min"]', '10');
    await page.click('button[type="submit"]:has-text("Create Recipe")');

    // Wait for redirect to detail page
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);
    const recipeUrl = page.url();
    const recipeId = recipeUrl.split('/').pop();

    // Navigate to edit form
    await page.goto(`/recipes/${recipeId}/edit`);

    // Verify form is prepopulated
    await expect(page.locator('input[name="title"]')).toHaveValue(originalTitle);
    await expect(page.locator('input[name="ingredient_name[]"]')).toHaveValue('Sugar');

    // Update recipe
    await page.fill('input[name="title"]', updatedTitle);
    await page.fill('input[name="ingredient_quantity[]"]', '2'); // Change quantity
    await page.fill('input[name="prep_time_min"]', '15'); // Change prep time

    // Submit update
    await page.click('button[type="submit"]:has-text("Update Recipe")');

    // Verify redirect back to detail page
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Verify updates persisted
    await expect(page.locator('h1')).toContainText(updatedTitle);
    await expect(page.locator('text=2 cup')).toBeVisible();
    await expect(page.locator('text=15 min')).toBeVisible();
  });

  test('user can add and remove ingredients dynamically', async ({ page }) => {
    await page.goto('/recipes/new');

    // Verify initial state: 1 ingredient row exists
    let ingredientRows = page.locator('.ingredient-row');
    await expect(ingredientRows).toHaveCount(1);

    // Add 2 more ingredients
    await page.click('button:has-text("Add Ingredient")');
    await page.click('button:has-text("Add Ingredient")');
    await expect(ingredientRows).toHaveCount(3);

    // Remove middle ingredient row
    const secondRow = page.locator('.ingredient-row').nth(1);
    await secondRow.locator('button:has-text("Remove")').click();

    // Verify row removed
    await expect(ingredientRows).toHaveCount(2);
  });

  test('user can add and remove instruction steps dynamically', async ({ page }) => {
    await page.goto('/recipes/new');

    // Verify initial state: 1 instruction row exists
    let instructionRows = page.locator('.instruction-row');
    await expect(instructionRows).toHaveCount(1);

    // Add 2 more steps
    await page.click('button:has-text("Add Step")');
    await page.click('button:has-text("Add Step")');
    await expect(instructionRows).toHaveCount(3);

    // Verify step numbers renumber correctly (1, 2, 3)
    const stepNumbers = page.locator('.instruction-number');
    await expect(stepNumbers.nth(0)).toHaveText('1');
    await expect(stepNumbers.nth(1)).toHaveText('2');
    await expect(stepNumbers.nth(2)).toHaveText('3');

    // Remove second instruction
    const secondRow = page.locator('.instruction-row').nth(1);
    await secondRow.locator('button:has-text("Remove")').click();

    // Verify row removed and renumbered (1, 2)
    await expect(instructionRows).toHaveCount(2);
    await expect(stepNumbers.nth(0)).toHaveText('1');
    await expect(stepNumbers.nth(1)).toHaveText('2');
  });
});

test.describe('Recipe List and Detail Views', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
  });

  test('user can view their recipe library', async ({ page }) => {
    // Navigate to recipes page
    await page.goto('/recipes');

    // Verify page loaded
    await expect(page.locator('h1')).toContainText('Recipes');

    // Verify recipes display (assuming user has at least one recipe from previous tests)
    // This is a smoke test - actual recipe content depends on test data
    const recipeCards = page.locator('.recipe-card');
    await expect(recipeCards.first()).toBeVisible();
  });

  test('user can navigate from recipe list to recipe detail', async ({ page }) => {
    // Go to recipe library
    await page.goto('/recipes');

    // Click on first recipe card
    const firstRecipe = page.locator('.recipe-card').first();
    await firstRecipe.click();

    // Verify navigated to detail page
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);
    await expect(page.locator('h1')).toBeVisible();
  });

  test('user can delete a recipe', async ({ page }) => {
    const recipeTitle = uniqueRecipeTitle('Recipe to Delete');

    // Create recipe to delete
    await page.goto('/recipes/new');
    await page.fill('input[name="title"]', recipeTitle);
    await page.fill('input[name="ingredient_name[]"]', 'Test');
    await page.fill('input[name="ingredient_quantity[]"]', '1');
    await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
    await page.fill('textarea[name="instruction_text[]"]', 'Test');
    await page.click('button[type="submit"]:has-text("Create Recipe")');

    // Wait for redirect
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Click delete button (with confirmation if implemented)
    const deleteButton = page.locator('button:has-text("Delete")');
    await deleteButton.click();

    // If confirmation dialog exists, accept it
    page.on('dialog', dialog => dialog.accept());

    // Verify redirect to recipes list or dashboard
    await page.waitForURL(/\/(recipes|dashboard)/);

    // Verify recipe no longer appears in list
    await page.goto('/recipes');
    await expect(page.locator(`text=${recipeTitle}`)).not.toBeVisible();
  });
});

/**
 * E2E Tests for Recipe Complexity (Story 3.12 AC-8)
 *
 * Tests verify that complexity badges are visible on recipe cards and detail pages,
 * and that users can filter recipes by complexity in the recipe library.
 */
test.describe('Recipe Complexity Display and Filtering', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
  });

  test('simple recipe displays green complexity badge', async ({ page }) => {
    const recipeTitle = uniqueRecipeTitle('Simple Recipe');

    // Create a simple recipe (5 ingredients, 4 steps, no advance prep)
    await page.goto('/recipes/new');
    await page.fill('input[name="title"]', recipeTitle);

    // Add 5 ingredients
    await page.fill('input[name="ingredient_name[]"]', 'Salt');
    await page.fill('input[name="ingredient_quantity[]"]', '1');
    await page.selectOption('select[name="ingredient_unit[]"]', 'tsp');

    for (let i = 2; i <= 5; i++) {
      await page.click('button:has-text("Add Ingredient")');
      await page.fill(`input[name="ingredient_name[]"]:nth-of-type(${i})`, `Ingredient ${i}`);
      await page.fill(`input[name="ingredient_quantity[]"]:nth-of-type(${i})`, '1');
      await page.selectOption(`select[name="ingredient_unit[]"]:nth-of-type(${i})`, 'cup');
    }

    // Add 4 instruction steps
    await page.fill('textarea[name="instruction_text[]"]', 'Step 1');

    for (let i = 2; i <= 4; i++) {
      await page.click('button:has-text("Add Step")');
      await page.fill(`textarea[name="instruction_text[]"]:nth-of-type(${i})`, `Step ${i}`);
    }

    // No advance prep (leave blank)
    await page.fill('input[name="prep_time_min"]', '10');
    await page.fill('input[name="cook_time_min"]', '15');

    // Submit recipe
    await page.click('button[type="submit"]:has-text("Create Recipe")');
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Verify green "Simple" complexity badge on detail page
    const complexityBadge = page.locator('text=Simple').first();
    await expect(complexityBadge).toBeVisible();
    await expect(complexityBadge).toHaveClass(/bg-green/);

    // Navigate to recipe list and verify badge appears on card
    await page.goto('/recipes');
    const recipeCard = page.locator('.recipe-card', { hasText: recipeTitle }).first();
    const cardBadge = recipeCard.locator('text=Simple');
    await expect(cardBadge).toBeVisible();
    await expect(cardBadge).toHaveClass(/bg-green/);
  });

  test('complex recipe displays red complexity badge', async ({ page }) => {
    const recipeTitle = uniqueRecipeTitle('Complex Recipe');

    // Create a complex recipe (20 ingredients, 15 steps, 4-hour advance prep)
    await page.goto('/recipes/new');
    await page.fill('input[name="title"]', recipeTitle);

    // Add 20 ingredients
    await page.fill('input[name="ingredient_name[]"]', 'Ingredient 1');
    await page.fill('input[name="ingredient_quantity[]"]', '1');
    await page.selectOption('select[name="ingredient_unit[]"]', 'cup');

    for (let i = 2; i <= 20; i++) {
      await page.click('button:has-text("Add Ingredient")');
      await page.fill(`input[name="ingredient_name[]"]:nth-of-type(${i})`, `Ingredient ${i}`);
      await page.fill(`input[name="ingredient_quantity[]"]:nth-of-type(${i})`, '1');
      await page.selectOption(`select[name="ingredient_unit[]"]:nth-of-type(${i})`, 'cup');
    }

    // Add 15 instruction steps
    await page.fill('textarea[name="instruction_text[]"]', 'Step 1');

    for (let i = 2; i <= 15; i++) {
      await page.click('button:has-text("Add Step")');
      await page.fill(`textarea[name="instruction_text[]"]:nth-of-type(${i})`, `Step ${i}`);
    }

    // Add 4-hour advance prep
    await page.fill('input[name="prep_time_min"]', '30');
    await page.fill('input[name="cook_time_min"]', '60');
    await page.fill('input[name="advance_prep_hours"]', '4');

    // Submit recipe
    await page.click('button[type="submit"]:has-text("Create Recipe")');
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Verify red "Complex" complexity badge on detail page
    const complexityBadge = page.locator('text=Complex').first();
    await expect(complexityBadge).toBeVisible();
    await expect(complexityBadge).toHaveClass(/bg-red/);

    // Navigate to recipe list and verify badge appears on card
    await page.goto('/recipes');
    const recipeCard = page.locator('.recipe-card', { hasText: recipeTitle }).first();
    const cardBadge = recipeCard.locator('text=Complex');
    await expect(cardBadge).toBeVisible();
    await expect(cardBadge).toHaveClass(/bg-red/);
  });

  test('user can filter recipes by complexity', async ({ page }) => {
    // Go to recipe library
    await page.goto('/recipes');

    // Verify complexity filter section exists
    await expect(page.locator('text=Complexity')).toBeVisible();

    // Click "Simple" complexity filter
    await page.click('a[href*="complexity=simple"]');

    // Verify URL includes complexity filter
    await expect(page).toHaveURL(/complexity=simple/);

    // Verify only simple recipes are displayed (all visible badges should be green "Simple")
    const complexityBadges = page.locator('span', { hasText: /^(Simple|Moderate|Complex)$/ });
    const badgeCount = await complexityBadges.count();

    if (badgeCount > 0) {
      // Verify all visible badges say "Simple"
      for (let i = 0; i < badgeCount; i++) {
        const badge = complexityBadges.nth(i);
        await expect(badge).toHaveText('Simple');
        await expect(badge).toHaveClass(/bg-green/);
      }
    }

    // Click "Moderate" complexity filter
    await page.click('a[href*="complexity=moderate"]');
    await expect(page).toHaveURL(/complexity=moderate/);

    // Click "Complex" complexity filter
    await page.click('a[href*="complexity=complex"]');
    await expect(page).toHaveURL(/complexity=complex/);

    // Verify active filter is highlighted
    const activeFilter = page.locator('a[href*="complexity=complex"]');
    await expect(activeFilter).toHaveClass(/bg-red-50|text-red-800|font-medium/);
  });

  test('complexity filter is visually highlighted when active', async ({ page }) => {
    await page.goto('/recipes');

    // Click "Simple" filter
    const simpleFilter = page.locator('a[href*="complexity=simple"]');
    await simpleFilter.click();

    // Verify active state styling
    await expect(simpleFilter).toHaveClass(/bg-green-50|text-green-800|font-medium/);

    // Click "All Recipes" to clear filter
    await page.click('a[href="/recipes"]:has-text("All Recipes")');

    // Verify filter no longer highlighted
    await expect(simpleFilter).not.toHaveClass(/bg-green-50|text-green-800|font-medium/);
  });
});
