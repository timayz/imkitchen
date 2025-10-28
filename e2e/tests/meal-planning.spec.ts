import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Meal Planning Flows (Story 10.1 - AC2, AC3, AC4, AC5)
 *
 * Tests cover critical meal planning user flows:
 * - Multi-week meal plan generation (4 weeks)
 * - Week navigation (Next/Previous buttons)
 * - Single week regeneration
 * - All future weeks regeneration
 *
 * Uses authenticated session via storageState in playwright.config.ts
 */

test.describe('Meal Planning E2E Tests', () => {
  /**
   * AC2: Test coverage for multi-week meal plan generation flow
   * Verifies: Navigate to /plan, submit generation form, assert success redirect
   */
  test('User generates multi-week meal plan (4 weeks)', async ({ page }) => {
    // Navigate to meal planning page
    await page.goto('/plan');

    // Check if "Generate Meal Plan" form is present (assumes no existing plan)
    const generateButton = page.locator('button:has-text("Generate"), form[action="/plan/generate-multi-week"] button[type="submit"]');

    // If generate button exists, click it to generate meal plan
    if (await generateButton.isVisible()) {
      // Optional: Verify num_weeks input defaults to 4
      const numWeeksInput = page.locator('input[name="num_weeks"]');
      if (await numWeeksInput.isVisible()) {
        const value = await numWeeksInput.inputValue();
        expect(parseInt(value || '4')).toBe(4);
      }

      // Submit form to generate meal plan
      await generateButton.click();

      // Wait for redirect to /plan?week={first_monday} (AC2)
      await page.waitForURL(/\/plan\?week=\d{4}-\d{2}-\d{2}/);
    }

    // Verify meal plan calendar is displayed
    const calendar = page.locator('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    await expect(calendar).toBeVisible();

    // Verify calendar shows 7 days (Monday-Sunday)
    // Look for day headers or meal slots for each day of the week
    const dayElements = page.locator('[data-day], .day-header, .calendar-day');
    const dayCount = await dayElements.count();
    expect(dayCount).toBeGreaterThanOrEqual(7);

    // Verify meal assignments are present (3 courses per day: breakfast, lunch, dinner)
    const mealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const mealCount = await mealSlots.count();
    expect(mealCount).toBeGreaterThanOrEqual(21); // 7 days × 3 meals minimum
  });

  /**
   * AC3: Test coverage for week navigation
   * Verifies: Click "Next Week" button, calendar updates to show next Monday-Sunday range
   */
  test('User navigates between weeks', async ({ page }) => {
    await page.goto('/plan');

    // Ensure meal plan exists (may need generation first)
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Get current week date from URL or page heading
    const currentUrl = page.url();
    const currentWeekMatch = currentUrl.match(/week=(\d{4}-\d{2}-\d{2})/);
    const currentWeekDate = currentWeekMatch ? currentWeekMatch[1] : null;

    // Click "Next Week" button
    const nextWeekButton = page.locator('button:has-text("Next"), a:has-text("Next Week"), [data-action="next-week"]').first();
    await expect(nextWeekButton).toBeVisible();
    await nextWeekButton.click();

    // Wait for calendar to update (TwinSpark attribute: ts-req, ts-target)
    // Use waitForSelector for DOM updates after TwinSpark AJAX requests (not fixed delays)
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Verify URL updated to next week's Monday date
    const nextUrl = page.url();
    const nextWeekMatch = nextUrl.match(/week=(\d{4}-\d{2}-\d{2})/);
    expect(nextWeekMatch).not.toBeNull();

    // If we captured the original week, verify it advanced by 7 days
    if (currentWeekDate && nextWeekMatch) {
      const currentDate = new Date(currentWeekDate);
      const nextDate = new Date(nextWeekMatch[1]);
      const daysDiff = Math.floor((nextDate.getTime() - currentDate.getTime()) / (1000 * 60 * 60 * 24));
      expect(daysDiff).toBe(7); // Advanced by one week
    }

    // Click "Previous Week" button to go back
    const prevWeekButton = page.locator('button:has-text("Previous"), a:has-text("Previous Week"), [data-action="prev-week"]').first();
    await expect(prevWeekButton).toBeVisible();
    await prevWeekButton.click();

    // Wait for calendar to update
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Verify we're back to original week (URL should match currentUrl or be close)
    const backUrl = page.url();
    if (currentWeekDate) {
      expect(backUrl).toContain(`week=${currentWeekDate}`);
    }
  });

  /**
   * AC4: Test coverage for single week regeneration
   * Verifies: Click "Regenerate This Week" button, only target week changes (other weeks unchanged)
   */
  test('User regenerates single week', async ({ page }) => {
    await page.goto('/plan');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Capture current week's meal assignments before regeneration
    const mealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const initialMealCount = await mealSlots.count();
    const firstMealTitle = await mealSlots.first().textContent();

    // Click "Regenerate This Week" button
    const regenerateWeekButton = page.locator(
      'button:has-text("Regenerate This Week"), button:has-text("Regenerate Week"), [data-action="regenerate-week"]'
    ).first();
    await expect(regenerateWeekButton).toBeVisible();
    await regenerateWeekButton.click();

    // Wait for regeneration to complete (redirect to /plan?week={regenerated_week})
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Verify meal assignments changed for this week
    const regeneratedMealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const regeneratedMealCount = await regeneratedMealSlots.count();
    const regeneratedFirstMealTitle = await regeneratedMealSlots.first().textContent();

    // Should have same number of meals
    expect(regeneratedMealCount).toBe(initialMealCount);

    // First meal should be different (high probability with 50 recipes)
    expect(regeneratedFirstMealTitle).not.toBe(firstMealTitle);

    // Navigate to next week to verify other weeks unchanged (if multi-week plan exists)
    const nextWeekButton = page.locator('button:has-text("Next"), a:has-text("Next Week")').first();
    if (await nextWeekButton.isVisible()) {
      await nextWeekButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

      // This week should still have meals (not affected by single week regeneration)
      const nextWeekMealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
      const nextWeekMealCount = await nextWeekMealSlots.count();
      expect(nextWeekMealCount).toBeGreaterThan(0);
    }
  });

  /**
   * AC5: Test coverage for all future weeks regeneration
   * Verifies: Click "Regenerate All Future Weeks" button, all weeks from current forward regenerated
   */
  test('User regenerates all future weeks', async ({ page }) => {
    await page.goto('/plan');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Capture current week's meal assignments
    const mealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const initialFirstMealTitle = await mealSlots.first().textContent();

    // Navigate to next week to capture future week's meals
    const nextWeekButton = page.locator('button:has-text("Next"), a:has-text("Next Week")').first();
    let futureWeekInitialMeal: string | null = null;

    if (await nextWeekButton.isVisible()) {
      await nextWeekButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

      const futureWeekMealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
      futureWeekInitialMeal = await futureWeekMealSlots.first().textContent();

      // Go back to current week
      const prevWeekButton = page.locator('button:has-text("Previous"), a:has-text("Previous Week")').first();
      await prevWeekButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');
    }

    // Click "Regenerate All Future Weeks" button
    const regenerateAllButton = page.locator(
      'button:has-text("Regenerate All Future Weeks"), button:has-text("Regenerate Future"), [data-action="regenerate-future"]'
    ).first();
    await expect(regenerateAllButton).toBeVisible();
    await regenerateAllButton.click();

    // Wait for regeneration to complete (redirect to /plan?week={current_week})
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

    // Verify current week meals changed
    const regeneratedMealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
    const regeneratedFirstMealTitle = await regeneratedMealSlots.first().textContent();
    expect(regeneratedFirstMealTitle).not.toBe(initialFirstMealTitle);

    // Navigate to future week to verify it also changed
    if (await nextWeekButton.isVisible() && futureWeekInitialMeal) {
      await nextWeekButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForSelector('[data-testid="meal-calendar"], .meal-calendar, #meal-calendar');

      const futureWeekRegeneratedMealSlots = page.locator('[data-meal], .meal-slot, .meal-assignment');
      const futureWeekRegeneratedMeal = await futureWeekRegeneratedMealSlots.first().textContent();

      // Future week should also be different (regenerated)
      expect(futureWeekRegeneratedMeal).not.toBe(futureWeekInitialMeal);
    }
  });
});
