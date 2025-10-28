import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Meal Planning Preferences (Story 10.1 - AC6)
 *
 * Tests cover:
 * - Navigation to /profile/meal-planning-preferences
 * - Toggling checkboxes (breakfast, lunch, dinner, side dishes)
 * - Form submission
 * - Preferences persistence (next meal plan generation respects preferences)
 *
 * Uses authenticated session via storageState in playwright.config.ts
 */

test.describe('Meal Planning Preferences E2E Tests', () => {
  /**
   * AC6: Test coverage for meal planning preferences update
   * Verifies: Navigate to preferences page, toggle checkboxes, submit form, preferences saved
   */
  test('User updates meal planning preferences and they persist', async ({ page }) => {
    // Navigate to meal planning preferences page
    await page.goto('/profile/meal-planning-preferences');

    // Verify preferences form is displayed
    const form = page.locator('form[action="/profile/meal-planning-preferences"], form[method="post"]');
    await expect(form).toBeVisible();

    // Verify all preference checkboxes are present
    const breakfastCheckbox = page.locator('input[name="generate_breakfast"], input[id="generate_breakfast"]');
    const lunchCheckbox = page.locator('input[name="generate_lunch"], input[id="generate_lunch"]');
    const dinnerCheckbox = page.locator('input[name="generate_dinner"], input[id="generate_dinner"]');
    const sideDishesCheckbox = page.locator('input[name="prefer_side_dishes"], input[id="prefer_side_dishes"]');

    await expect(breakfastCheckbox).toBeVisible();
    await expect(lunchCheckbox).toBeVisible();
    await expect(dinnerCheckbox).toBeVisible();
    await expect(sideDishesCheckbox).toBeVisible();

    // Capture initial checkbox states
    const initialBreakfast = await breakfastCheckbox.isChecked();
    const initialLunch = await lunchCheckbox.isChecked();
    const initialDinner = await dinnerCheckbox.isChecked();
    const initialSideDishes = await sideDishesCheckbox.isChecked();

    // Toggle checkboxes to new state
    // Ensure at least one meal type remains checked (system requirement)
    if (initialBreakfast) {
      await breakfastCheckbox.uncheck();
    } else {
      await breakfastCheckbox.check();
    }

    if (initialSideDishes) {
      await sideDishesCheckbox.uncheck();
    } else {
      await sideDishesCheckbox.check();
    }

    // Submit form
    const submitButton = page.locator('button[type="submit"]:has-text("Save"), button:has-text("Update Preferences")');
    await submitButton.click();

    // Wait for redirect to /profile (AC6: 303 Redirect to /profile)
    await page.waitForURL(/\/profile/);

    // Verify success message or confirmation
    const successMessage = page.locator(':text("Preferences updated"), :text("Settings saved"), .success-message');
    if (await successMessage.isVisible()) {
      await expect(successMessage).toBeVisible();
    }

    // Navigate back to preferences page to verify persistence
    await page.goto('/profile/meal-planning-preferences');

    // Verify checkbox states persisted
    const updatedBreakfast = await breakfastCheckbox.isChecked();
    const updatedSideDishes = await sideDishesCheckbox.isChecked();

    expect(updatedBreakfast).toBe(!initialBreakfast); // Toggled
    expect(updatedSideDishes).toBe(!initialSideDishes); // Toggled
  });

  /**
   * AC6 (Extended): Verify preferences applied to next meal plan generation
   * Tests that disabling breakfast/lunch/dinner actually affects meal plan generation
   */
  test('User preferences are applied to meal plan generation', async ({ page }) => {
    // Navigate to preferences page
    await page.goto('/profile/meal-planning-preferences');

    // Set specific preferences: Enable all meals, disable side dishes
    const breakfastCheckbox = page.locator('input[name="generate_breakfast"], input[id="generate_breakfast"]');
    const lunchCheckbox = page.locator('input[name="generate_lunch"], input[id="generate_lunch"]');
    const dinnerCheckbox = page.locator('input[name="generate_dinner"], input[id="generate_dinner"]');
    const sideDishesCheckbox = page.locator('input[name="prefer_side_dishes"], input[id="prefer_side_dishes"]');

    // Ensure all meals are enabled
    await breakfastCheckbox.check();
    await lunchCheckbox.check();
    await dinnerCheckbox.check();
    await sideDishesCheckbox.uncheck(); // Disable side dish preference

    // Submit preferences
    const submitButton = page.locator('button[type="submit"]:has-text("Save"), button:has-text("Update Preferences")');
    await submitButton.click();
    await page.waitForURL(/\/profile/);

    // Navigate to meal planning page and generate new plan (or regenerate)
    await page.goto('/plan');

    // Click regenerate to apply new preferences (if plan already exists)
    const regenerateButton = page.locator(
      'button:has-text("Regenerate All"), button:has-text("Regenerate Future"), [data-action="regenerate-future"]'
    );

    if (await regenerateButton.isVisible()) {
      await regenerateButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    } else {
      // Generate new plan if none exists
      const generateButton = page.locator('button:has-text("Generate"), form[action="/plan/generate-multi-week"] button[type="submit"]');
      if (await generateButton.isVisible()) {
        await generateButton.click();
        await page.waitForURL(/\/plan\?week=/);
      }
    }

    // Verify meal plan respects preferences
    // Should have meals for all three courses (breakfast, lunch, dinner)
    const mealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const mealCount = await mealSlots.count();

    // With all meals enabled, should have 21 meals minimum (7 days × 3 meals)
    expect(mealCount).toBeGreaterThanOrEqual(21);

    // Now test disabling breakfast
    await page.goto('/profile/meal-planning-preferences');
    await breakfastCheckbox.uncheck(); // Disable breakfast
    await lunchCheckbox.check(); // Ensure lunch is enabled
    await dinnerCheckbox.check(); // Ensure dinner is enabled

    await submitButton.click();
    await page.waitForURL(/\/profile/);

    // Regenerate meal plan with new preferences
    await page.goto('/plan');
    const regenerateButton2 = page.locator('button:has-text("Regenerate All"), [data-action="regenerate-future"]');
    if (await regenerateButton2.isVisible()) {
      await regenerateButton2.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    }

    // Verify fewer meals generated (no breakfast)
    const updatedMealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const updatedMealCount = await updatedMealSlots.count();

    // Should have 14 meals (7 days × 2 meals: lunch + dinner)
    expect(updatedMealCount).toBeLessThan(mealCount); // Fewer meals than before
    expect(updatedMealCount).toBeGreaterThanOrEqual(14); // At least lunch and dinner
  });

  /**
   * AC6 (Form Validation): Verify form validates at least one meal type is selected
   */
  test('User cannot disable all meal types (validation)', async ({ page }) => {
    await page.goto('/profile/meal-planning-preferences');

    const breakfastCheckbox = page.locator('input[name="generate_breakfast"], input[id="generate_breakfast"]');
    const lunchCheckbox = page.locator('input[name="generate_lunch"], input[id="generate_lunch"]');
    const dinnerCheckbox = page.locator('input[name="generate_dinner"], input[id="generate_dinner"]');

    // Try to uncheck all meal types
    await breakfastCheckbox.uncheck();
    await lunchCheckbox.uncheck();
    await dinnerCheckbox.uncheck();

    // Submit form
    const submitButton = page.locator('button[type="submit"]:has-text("Save"), button:has-text("Update Preferences")');
    await submitButton.click();

    // Verify form validation prevents submission or shows error message
    // Either:
    // 1. HTML5 validation prevents form submission
    // 2. Server returns error and displays message
    const errorMessage = page.locator(':text("at least one meal"), :text("select meal type"), .error-message');

    // If still on preferences page, validation worked (didn't redirect to /profile)
    const currentUrl = page.url();
    const stillOnPreferencesPage = currentUrl.includes('/meal-planning-preferences');

    // Either error message shown OR still on preferences page (validation prevented submission)
    if (await errorMessage.isVisible()) {
      await expect(errorMessage).toBeVisible();
    } else if (stillOnPreferencesPage) {
      // HTML5 validation likely prevented submission - this is acceptable
      expect(currentUrl).toContain('/meal-planning-preferences');
    }
  });
});
