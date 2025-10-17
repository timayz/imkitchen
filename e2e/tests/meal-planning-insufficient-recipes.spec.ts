import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Insufficient Recipes Handling (Story 3.10)
 *
 * Tests verify that users with fewer than 7 favorite recipes receive
 * helpful guidance and cannot generate meal plans until threshold is met.
 *
 * Acceptance Criteria:
 * - AC-1: "Generate Meal Plan" button visible but triggers validation
 * - AC-2: Error message shows current count and required count
 * - AC-3: Helpful guidance shows how many more recipes needed
 * - AC-4: Direct links to "Add Recipe" and "Discover Recipes" pages
 * - AC-5: Friendly styling (not alarming red)
 * - AC-6: Validation prevents wasted algorithm execution
 * - AC-7: Count updates in real-time as user adds/removes favorites
 */

// Helper function to clean up test recipes
async function deleteAllUserRecipes(page: any) {
  // Navigate to recipes page
  await page.goto('/recipes');

  // Delete all recipes (if any exist)
  while (await page.locator('button:has-text("Delete")').first().isVisible().catch(() => false)) {
    await page.locator('button:has-text("Delete")').first().click();
    await page.waitForTimeout(100); // Brief wait for deletion
  }
}

test.describe('Insufficient Recipes for Meal Plan Generation', () => {
  test.beforeEach(async ({ page }) => {
    // Login as test user
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');

    // Clean up: Delete all existing recipes
    await deleteAllUserRecipes(page);
  });

  test('AC-7: Dashboard shows progress indicator with < 7 favorites', async ({ page }) => {
    // Navigate to dashboard
    await page.goto('/dashboard');

    // Verify progress indicator is displayed
    await expect(page.locator('text=/You have .* favorite recipes/')).toBeVisible();

    // Verify "Add Recipe" and "Discover Recipes" buttons are shown
    await expect(page.locator('a:has-text("Add Recipe")')).toBeVisible();
    await expect(page.locator('a:has-text("Discover Recipes")')).toBeVisible();

    // Verify "Generate Meal Plan" button is NOT shown when insufficient
    const generateButton = page.locator('button:has-text("Generate Meal Plan")');
    await expect(generateButton).not.toBeVisible();
  });

  test('AC-7: Count updates when user adds recipes to favorites', async ({ page }) => {
    // Start with 0 recipes on dashboard
    await page.goto('/dashboard');

    // Verify initial state shows 0/7
    await expect(page.locator('text=/0\/7/')).toBeVisible();

    // Create first recipe and mark as favorite
    await page.goto('/recipes/new');
    await page.fill('input[name="title"]', `Test Recipe ${Date.now()}`);
    await page.fill('input[name="ingredient_name[]"]', 'Test Ingredient');
    await page.fill('input[name="ingredient_quantity[]"]', '1');
    await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
    await page.fill('textarea[name="instruction_text[]"]', 'Test instruction');
    await page.fill('input[name="prep_time_min"]', '10');
    await page.fill('input[name="cook_time_min"]', '20');
    await page.fill('input[name="serving_size"]', '2');

    // Mark as favorite
    await page.check('input[name="is_favorite"]');

    // Submit
    await page.click('button[type="submit"]:has-text("Create Recipe")');
    await page.waitForURL(/\/recipes\/[a-zA-Z0-9-]+$/);

    // Return to dashboard and verify count updated
    await page.goto('/dashboard');
    await expect(page.locator('text=/1\/7/')).toBeVisible();
    await expect(page.locator('text=/6 more needed/')).toBeVisible();
  });

  test('AC-1, AC-6: Attempting generation with insufficient recipes shows validation error', async ({ page }) => {
    // Create 5 favorite recipes (below threshold)
    for (let i = 1; i <= 5; i++) {
      await page.goto('/recipes/new');
      await page.fill('input[name="title"]', `Recipe ${i} - ${Date.now()}`);
      await page.fill('input[name="ingredient_name[]"]', 'Ingredient');
      await page.fill('input[name="ingredient_quantity[]"]', '1');
      await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
      await page.fill('textarea[name="instruction_text[]"]', 'Cook it');
      await page.fill('input[name="prep_time_min"]', '10');
      await page.fill('input[name="cook_time_min"]', '20');
      await page.fill('input[name="serving_size"]', '2');
      await page.check('input[name="is_favorite"]');
      await page.click('button[type="submit"]');
      await page.waitForURL(/\/recipes\//);
    }

    // Navigate to dashboard
    await page.goto('/dashboard');

    // Verify progress shows 5/7
    await expect(page.locator('text=/5\/7/')).toBeVisible();

    // Note: The dashboard should show action buttons, not generate button
    // If user somehow triggers POST /plan/generate, they should see error page

    // Simulate direct POST request (bypassing UI) to test validation
    const response = await page.request.post('/plan/generate');
    expect(response.status()).toBe(422); // Unprocessable Entity
  });

  test('AC-2, AC-3, AC-4, AC-5: Error page shows friendly message and action buttons', async ({ page }) => {
    // Create 4 favorite recipes
    for (let i = 1; i <= 4; i++) {
      await page.goto('/recipes/new');
      await page.fill('input[name="title"]', `Recipe ${i} - ${Date.now()}`);
      await page.fill('input[name="ingredient_name[]"]', 'Ingredient');
      await page.fill('input[name="ingredient_quantity[]"]', '1');
      await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
      await page.fill('textarea[name="instruction_text[]"]', 'Cook it');
      await page.fill('input[name="prep_time_min"]', '10');
      await page.fill('input[name="cook_time_min"]', '20');
      await page.fill('input[name="serving_size"]', '2');
      await page.check('input[name="is_favorite"]');
      await page.click('button[type="submit"]');
      await page.waitForURL(/\/recipes\//);
    }

    // Trigger POST /plan/generate to see error page
    await page.goto('/plan/generate', { waitUntil: 'networkidle' });

    // AC-2: Verify error message includes counts
    await expect(page.locator('text=/You need at least 7 favorite recipes/')).toBeVisible();
    await expect(page.locator('text=/You currently have 4/')).toBeVisible();

    // AC-3: Verify helpful guidance
    await expect(page.locator('text=/Add 3 more recipe/')).toBeVisible();

    // AC-4: Verify action buttons present
    await expect(page.locator('a:has-text("Add New Recipe")')).toBeVisible();
    await expect(page.locator('a:has-text("Discover Community Recipes")')).toBeVisible();

    // AC-5: Verify friendly styling (soft amber/orange, not red)
    const errorContainer = page.locator('.bg-amber-50, .bg-amber-100, .bg-yellow-50');
    await expect(errorContainer).toBeVisible();

    // Verify informational icon (not alarming triangle)
    const infoIcon = page.locator('svg').filter({ hasText: /info/i }).or(page.locator('.text-amber-600'));
    await expect(infoIcon).toBeVisible();
  });

  test('AC-7: Button enables when user reaches 7 favorites', async ({ page }) => {
    // Create 7 favorite recipes
    for (let i = 1; i <= 7; i++) {
      await page.goto('/recipes/new');
      await page.fill('input[name="title"]', `Recipe ${i} - ${Date.now()}`);
      await page.fill('input[name="ingredient_name[]"]', 'Ingredient');
      await page.fill('input[name="ingredient_quantity[]"]', '1');
      await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
      await page.fill('textarea[name="instruction_text[]"]', 'Cook it');
      await page.fill('input[name="prep_time_min"]', '10');
      await page.fill('input[name="cook_time_min"]', '20');
      await page.fill('input[name="serving_size"]', '2');
      await page.check('input[name="is_favorite"]');
      await page.click('button[type="submit"]');
      await page.waitForURL(/\/recipes\//);
    }

    // Navigate to dashboard
    await page.goto('/dashboard');

    // Verify count shows 7/7 or just shows enabled button
    // The template should hide progress and show "Generate Meal Plan" button
    await expect(page.locator('button:has-text("Generate Meal Plan")')).toBeVisible();

    // Verify button is enabled (not disabled)
    const button = page.locator('button:has-text("Generate Meal Plan")');
    await expect(button).toBeEnabled();

    // Click should successfully trigger generation (not show error)
    await button.click();
    await page.waitForURL('/plan', { timeout: 10000 });

    // Should land on meal plan calendar page (successful generation)
    await expect(page.locator('text=/Meal Plan/')).toBeVisible();
  });

  test('AC-7: Unfavoriting recipes updates count and disables button', async ({ page }) => {
    // Create 7 favorite recipes first
    const recipeIds: string[] = [];
    for (let i = 1; i <= 7; i++) {
      await page.goto('/recipes/new');
      await page.fill('input[name="title"]', `Recipe ${i} - ${Date.now()}`);
      await page.fill('input[name="ingredient_name[]"]', 'Ingredient');
      await page.fill('input[name="ingredient_quantity[]"]', '1');
      await page.selectOption('select[name="ingredient_unit[]"]', 'cup');
      await page.fill('textarea[name="instruction_text[]"]', 'Cook it');
      await page.fill('input[name="prep_time_min"]', '10');
      await page.fill('input[name="cook_time_min"]', '20');
      await page.fill('input[name="serving_size"]', '2');
      await page.check('input[name="is_favorite"]');
      await page.click('button[type="submit"]');
      await page.waitForURL(/\/recipes\/([a-zA-Z0-9-]+)$/);

      // Extract recipe ID from URL
      const url = page.url();
      const match = url.match(/\/recipes\/([a-zA-Z0-9-]+)$/);
      if (match) recipeIds.push(match[1]);
    }

    // Verify button is enabled with 7 favorites
    await page.goto('/dashboard');
    await expect(page.locator('button:has-text("Generate Meal Plan")')).toBeVisible();

    // Unfavorite 2 recipes (dropping to 5 favorites)
    for (let i = 0; i < 2; i++) {
      await page.goto(`/recipes/${recipeIds[i]}`);
      await page.click('button:has-text("Favorite"), button:has-text("★")');
      await page.waitForTimeout(200); // Wait for unfavorite action
    }

    // Return to dashboard
    await page.goto('/dashboard');

    // Verify count updated to 5/7
    await expect(page.locator('text=/5\/7/')).toBeVisible();

    // Verify "Generate Meal Plan" button is hidden/disabled
    await expect(page.locator('button:has-text("Generate Meal Plan")')).not.toBeVisible();

    // Verify action buttons are shown instead
    await expect(page.locator('a:has-text("Add Recipe")')).toBeVisible();
  });
});
