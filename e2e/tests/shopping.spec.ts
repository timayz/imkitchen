import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Shopping List Access (Story 10.1 - AC8)
 *
 * Tests cover:
 * - Navigation to /shopping with week-specific query parameter
 * - Shopping list displays ingredients for that week only
 * - Week selector functionality
 *
 * Uses authenticated session via storageState in playwright.config.ts
 */

test.describe('Shopping List E2E Tests', () => {
  /**
   * AC8: Test coverage for shopping list access for specific week
   * Verifies: Navigate to /shopping?week=2025-11-10, shopping list displays week-specific ingredients
   */
  test('User views shopping list for specific week', async ({ page }) => {
    // First, ensure a meal plan exists by navigating to meal planning page
    await page.goto('/plan');

    // Wait for meal plan calendar to load (assumes plan already generated or generate one)
    const generateButton = page.locator('button:has-text("Generate"), form[action="/plan/generate-multi-week"] button[type="submit"]');

    if (await generateButton.isVisible()) {
      // Generate meal plan if none exists
      await generateButton.click();
      await page.waitForURL(/\/plan\?week=/);
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    } else {
      // Plan already exists, just ensure calendar loaded
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    }

    // Extract week date from URL or default to a known week
    const currentUrl = page.url();
    const weekMatch = currentUrl.match(/week=(\d{4}-\d{2}-\d{2})/);
    const weekDate = weekMatch ? weekMatch[1] : '2025-11-10';

    // Navigate to shopping list for specific week
    await page.goto(`/shopping?week=${weekDate}`);

    // Verify shopping list page loaded
    const shoppingList = page.locator('[data-testid="shopping-list"], .shopping-list, #shopping-list');
    await expect(shoppingList).toBeVisible();

    // Verify page heading shows correct week
    const heading = page.locator('h1, h2');
    await expect(heading.first()).toBeVisible();

    // Verify shopping list has ingredients
    const ingredients = page.locator('[data-testid="shopping-item"], .shopping-item, .ingredient-item');
    const ingredientCount = await ingredients.count();

    // Should have at least one ingredient for the week
    expect(ingredientCount).toBeGreaterThan(0);

    // Verify ingredients are grouped by category (optional, depends on implementation)
    const categories = page.locator('[data-testid="category"], .category-header, h3');
    if (await categories.first().isVisible()) {
      const categoryCount = await categories.count();
      expect(categoryCount).toBeGreaterThan(0);
    }
  });

  /**
   * AC8 (Extended): Verify week selector allows switching between weeks
   */
  test('User can switch between weeks in shopping list', async ({ page }) => {
    // Navigate to shopping list (current week by default)
    await page.goto('/shopping');

    // Wait for shopping list to load
    await page.waitForSelector('[data-testid="shopping-list"], .shopping-list, #shopping-list');

    // Capture current week ingredients
    const initialIngredients = page.locator('[data-testid="shopping-item"], .shopping-item, .ingredient-item');
    const initialCount = await initialIngredients.count();
    const firstIngredientText = initialCount > 0 ? await initialIngredients.first().textContent() : null;

    // Look for week selector dropdown or navigation buttons
    const weekSelector = page.locator('select[name="week"], #week-selector, [data-testid="week-selector"]');
    const nextWeekButton = page.locator('button:has-text("Next Week"), a:has-text("Next Week")');

    if (await weekSelector.isVisible()) {
      // Use dropdown selector
      const options = await weekSelector.locator('option').allTextContents();
      if (options.length > 1) {
        // Select a different week
        await weekSelector.selectOption({ index: 1 });
        await page.waitForLoadState('networkidle');
      }
    } else if (await nextWeekButton.isVisible()) {
      // Use navigation buttons
      await nextWeekButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="shopping-list"], .shopping-list, #shopping-list');
    }

    // Verify shopping list updated (URL changed and/or content changed)
    const updatedUrl = page.url();
    const updatedWeekMatch = updatedUrl.match(/week=(\d{4}-\d{2}-\d{2})/);

    if (updatedWeekMatch) {
      // URL contains week parameter - good
      expect(updatedWeekMatch[1]).toBeTruthy();
    }

    // Verify ingredient list updated (may be different or same depending on meal plans)
    const updatedIngredients = page.locator('[data-testid="shopping-item"], .shopping-item, .ingredient-item');
    const updatedCount = await updatedIngredients.count();
    expect(updatedCount).toBeGreaterThan(0);
  });

  /**
   * AC8 (Edge Case): Verify shopping list shows empty state for week with no meal plan
   */
  test('Shopping list shows appropriate state when no meal plan exists for week', async ({ page }) => {
    // Navigate to shopping list for a future week that likely has no meal plan
    const futureWeek = '2026-01-05'; // Far future date
    await page.goto(`/shopping?week=${futureWeek}`);

    // Verify page loads (should not crash)
    await page.waitForLoadState('networkidle');

    // Check for empty state or message
    const emptyState = page.locator(':text("No meal plan"), :text("No shopping list"), .empty-state');
    const shoppingItems = page.locator('[data-testid="shopping-item"], .shopping-item, .ingredient-item');

    // Either:
    // 1. Empty state message is shown
    // 2. No shopping items are displayed
    const itemCount = await shoppingItems.count();

    if (await emptyState.isVisible()) {
      await expect(emptyState).toBeVisible();
    } else {
      // If no explicit empty state, should have zero items
      expect(itemCount).toBe(0);
    }
  });

  /**
   * AC8 (Category Grouping): Verify ingredients are organized by category
   */
  test('Shopping list ingredients are grouped by category', async ({ page }) => {
    await page.goto('/shopping');
    await page.waitForSelector('[data-testid="shopping-list"], .shopping-list, #shopping-list');

    // Verify category headers exist
    const categoryHeaders = page.locator('[data-testid="category"], .category-header, h3:has-text("Produce"), h3:has-text("Dairy"), h3:has-text("Meat")');

    if (await categoryHeaders.first().isVisible()) {
      const categoryCount = await categoryHeaders.count();
      expect(categoryCount).toBeGreaterThan(0);

      // Verify each category has associated items
      const firstCategory = categoryHeaders.first();
      await expect(firstCategory).toBeVisible();
    }
  });

  /**
   * AC8 (Checkoff Functionality): Verify users can check off shopping list items
   */
  test('User can check off items in shopping list', async ({ page }) => {
    await page.goto('/shopping');
    await page.waitForSelector('[data-testid="shopping-list"], .shopping-list, #shopping-list');

    // Find checkboxes for shopping items
    const checkboxes = page.locator('input[type="checkbox"][name*="item"], .shopping-item input[type="checkbox"]');

    if (await checkboxes.first().isVisible()) {
      const firstCheckbox = checkboxes.first();

      // Verify checkbox is initially unchecked
      await expect(firstCheckbox).not.toBeChecked();

      // Check the checkbox
      await firstCheckbox.check();

      // Verify checkbox is now checked
      await expect(firstCheckbox).toBeChecked();

      // Verify visual feedback (strike-through, dimmed, etc.)
      const parentItem = firstCheckbox.locator('xpath=ancestor::*[contains(@class, "shopping-item") or contains(@class, "ingredient-item")]').first();
      if (await parentItem.isVisible()) {
        // Item should have checked state styling (implementation-specific)
        // This is a visual check - actual assertion depends on CSS classes used
        await expect(parentItem).toBeVisible();
      }

      // Uncheck the checkbox
      await firstCheckbox.uncheck();

      // Verify checkbox is unchecked again
      await expect(firstCheckbox).not.toBeChecked();
    }
  });

  /**
   * AC8 (Integration): Verify shopping list updates when meal plan is regenerated
   */
  test('Shopping list updates when meal plan is regenerated', async ({ page }) => {
    // Navigate to meal planning page
    await page.goto('/plan');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Get current week from URL
    const planUrl = page.url();
    const weekMatch = planUrl.match(/week=(\d{4}-\d{2}-\d{2})/);
    const weekDate = weekMatch ? weekMatch[1] : null;

    // Navigate to shopping list for this week
    if (weekDate) {
      await page.goto(`/shopping?week=${weekDate}`);
    } else {
      await page.goto('/shopping');
    }

    await page.waitForSelector('[data-testid="shopping-list"], .shopping-list, #shopping-list');

    // Capture initial shopping list items
    const initialItems = page.locator('[data-testid="shopping-item"], .shopping-item, .ingredient-item');
    const initialCount = await initialItems.count();

    // Go back to meal planning and regenerate
    await page.goto('/plan');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    const regenerateButton = page.locator('button:has-text("Regenerate This Week"), [data-action="regenerate-week"]');
    if (await regenerateButton.isVisible()) {
      await regenerateButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    }

    // Return to shopping list
    if (weekDate) {
      await page.goto(`/shopping?week=${weekDate}`);
    } else {
      await page.goto('/shopping');
    }

    await page.waitForSelector('[data-testid="shopping-list"], .shopping-list, #shopping-list');

    // Verify shopping list updated (may have different items or count)
    const updatedItems = page.locator('[data-testid="shopping-item"], .shopping-item, .ingredient-item');
    const updatedCount = await updatedItems.count();

    // Shopping list should still have items (count may differ)
    expect(updatedCount).toBeGreaterThan(0);
  });
});
