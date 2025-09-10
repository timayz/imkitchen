import { test, expect } from '@playwright/test';

// End-to-end test for the complete meal plan → shopping list workflow
test.describe('Meal Plan to Shopping List Workflow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app and set up test environment
    await page.goto('http://localhost:3000');
    
    // Mock authentication for testing
    await page.evaluate(() => {
      localStorage.setItem('authToken', 'test-auth-token');
    });
  });

  test('should generate shopping list from meal plan creation', async ({ page }) => {
    // Step 1: Navigate to meal plan creation
    await page.click('[data-testid="create-meal-plan-button"]');
    await expect(page).toHaveURL(/.*meal-plans\/create/);

    // Step 2: Set up meal plan details
    await page.fill('[data-testid="week-start-date"]', '2024-01-01');
    await page.click('[data-testid="generate-meal-plan-button"]');

    // Step 3: Wait for meal plan generation to complete
    await expect(page.locator('[data-testid="meal-plan-grid"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 4: Verify automatic shopping list generation notification
    await expect(page.locator('[data-testid="shopping-list-notification"]')).toBeVisible({
      timeout: 5000,
    });
    
    const notificationText = await page.textContent('[data-testid="shopping-list-notification"]');
    expect(notificationText).toContain('Shopping list generated');

    // Step 5: Navigate to shopping list view
    await page.click('[data-testid="view-shopping-list-button"]');
    await expect(page).toHaveURL(/.*shopping-lists/);

    // Step 6: Verify shopping list content
    await expect(page.locator('[data-testid="shopping-list-screen"]')).toBeVisible();
    await expect(page.locator('[data-testid="shopping-progress-bar"]')).toBeVisible();
    
    // Verify grocery categories are present
    await expect(page.locator('[data-testid="grocery-section-produce"]')).toBeVisible();
    await expect(page.locator('[data-testid="grocery-section-dairy"]')).toBeVisible();
    await expect(page.locator('[data-testid="grocery-section-protein"]')).toBeVisible();
    
    // Verify shopping items are present
    const shoppingItems = page.locator('[data-testid^="shopping-item-"]');
    const itemCount = await shoppingItems.count();
    expect(itemCount).toBeGreaterThan(0);
  });

  test('should update shopping list when meal plan is modified', async ({ page }) => {
    // Step 1: Start with existing meal plan
    await page.goto('/meal-plans/test-meal-plan-1');
    await expect(page.locator('[data-testid="meal-plan-grid"]')).toBeVisible();

    // Step 2: Modify a meal slot
    await page.click('[data-testid="meal-slot-monday-breakfast"]');
    await page.click('[data-testid="change-recipe-button"]');
    await page.click('[data-testid="recipe-option-1"]');
    await page.click('[data-testid="save-meal-slot-button"]');

    // Step 3: Verify shopping list update notification
    await expect(page.locator('[data-testid="shopping-list-notification"]')).toBeVisible({
      timeout: 5000,
    });
    
    const notificationText = await page.textContent('[data-testid="shopping-list-notification"]');
    expect(notificationText).toContain('Shopping list updated');

    // Step 4: Navigate to shopping list to verify changes
    await page.click('[data-testid="view-shopping-list-button"]');
    
    // Wait for any diff modal to appear
    const diffModal = page.locator('[data-testid="shopping-list-diff-modal"]');
    if (await diffModal.isVisible()) {
      // Review changes and accept them
      await expect(page.locator('[data-testid="diff-summary"]')).toBeVisible();
      await page.click('[data-testid="accept-changes-button"]');
    }

    // Step 5: Verify updated shopping list
    await expect(page.locator('[data-testid="shopping-list-screen"]')).toBeVisible();
    
    // Shopping list should reflect the new recipe's ingredients
    const updatedItems = page.locator('[data-testid^="shopping-item-"]');
    const updatedItemCount = await updatedItems.count();
    expect(updatedItemCount).toBeGreaterThan(0);
  });

  test('should complete shopping list item workflow', async ({ page }) => {
    // Step 1: Navigate to existing shopping list
    await page.goto('/shopping-lists/test-shopping-list-1');
    await expect(page.locator('[data-testid="shopping-list-screen"]')).toBeVisible();

    // Step 2: Verify initial progress
    const initialProgress = await page.textContent('[data-testid="progress-text"]');
    expect(initialProgress).toMatch(/\d+ of \d+ completed/);

    // Step 3: Complete a shopping item
    const firstItem = page.locator('[data-testid^="shopping-item-"]').first();
    await firstItem.click();

    // Step 4: Verify item is marked as completed
    await expect(firstItem).toHaveClass(/completed/);
    
    // Step 5: Verify progress bar updated
    const updatedProgress = await page.textContent('[data-testid="progress-text"]');
    expect(updatedProgress).not.toBe(initialProgress);

    // Step 6: Add notes to an item
    const secondItem = page.locator('[data-testid^="shopping-item-"]').nth(1);
    await secondItem.locator('[data-testid="notes-button"]').click();
    
    await expect(page.locator('[data-testid="notes-modal"]')).toBeVisible();
    await page.fill('[data-testid="notes-input"]', 'Get organic if available');
    await page.click('[data-testid="save-notes-button"]');
    
    // Verify notes are saved
    await expect(secondItem.locator('[data-testid="item-notes"]')).toContainText('Get organic if available');

    // Step 7: View recipe sources
    const itemWithSources = page.locator('[data-testid^="shopping-item-"]').filter({
      has: page.locator('[data-testid="recipe-sources-button"]')
    }).first();
    
    await itemWithSources.locator('[data-testid="recipe-sources-button"]').click();
    await expect(page.locator('[data-testid="recipe-sources-modal"]')).toBeVisible();
    
    // Verify recipe sources are displayed
    const recipeSources = page.locator('[data-testid^="recipe-source-"]');
    const sourceCount = await recipeSources.count();
    expect(sourceCount).toBeGreaterThan(0);
    
    await page.click('[data-testid="close-modal-button"]');
  });

  test('should export shopping list in different formats', async ({ page }) => {
    // Step 1: Navigate to shopping list
    await page.goto('/shopping-lists/test-shopping-list-1');
    await expect(page.locator('[data-testid="shopping-list-screen"]')).toBeVisible();

    // Step 2: Open export modal
    await page.click('[data-testid="export-shopping-list-button"]');
    await expect(page.locator('[data-testid="export-modal"]')).toBeVisible();

    // Step 3: Test text export
    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-txt-button"]');
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/shopping-list.*\.txt$/);

    // Step 4: Test CSV export with recipe sources
    await page.click('[data-testid="export-shopping-list-button"]');
    const csvDownloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-csv-with-sources-button"]');
    const csvDownload = await csvDownloadPromise;
    expect(csvDownload.suggestedFilename()).toMatch(/shopping-list.*\.csv$/);

    // Step 5: Test JSON export
    await page.click('[data-testid="export-shopping-list-button"]');
    const jsonDownloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-json-button"]');
    const jsonDownload = await jsonDownloadPromise;
    expect(jsonDownload.suggestedFilename()).toMatch(/shopping-list.*\.json$/);
  });

  test('should handle performance requirements for large meal plans', async ({ page }) => {
    // Step 1: Create a large meal plan (3 weeks)
    await page.goto('/meal-plans/create');
    await page.fill('[data-testid="week-start-date"]', '2024-01-01');
    await page.check('[data-testid="generate-multiple-weeks"]');
    await page.fill('[data-testid="weeks-count"]', '3');
    
    // Step 2: Generate meal plan and measure time
    const startTime = Date.now();
    await page.click('[data-testid="generate-meal-plan-button"]');
    
    // Wait for generation to complete
    await expect(page.locator('[data-testid="meal-plan-grid"]')).toBeVisible({
      timeout: 15000,
    });
    
    const mealPlanGenerationTime = Date.now() - startTime;
    console.log(`Meal plan generation took: ${mealPlanGenerationTime}ms`);

    // Step 3: Generate shopping list and verify performance
    const shoppingListStartTime = Date.now();
    await page.click('[data-testid="generate-shopping-list-button"]');
    
    await expect(page.locator('[data-testid="shopping-list-notification"]')).toBeVisible({
      timeout: 5000,
    });
    
    const shoppingListGenerationTime = Date.now() - shoppingListStartTime;
    console.log(`Shopping list generation took: ${shoppingListGenerationTime}ms`);
    
    // Verify performance requirement (<3 seconds)
    expect(shoppingListGenerationTime).toBeLessThan(3000);

    // Step 4: Verify large shopping list functionality
    await page.click('[data-testid="view-shopping-list-button"]');
    await expect(page.locator('[data-testid="shopping-list-screen"]')).toBeVisible();
    
    // Verify all grocery categories are loaded
    const categorySections = page.locator('[data-testid^="grocery-section-"]');
    const categoryCount = await categorySections.count();
    expect(categoryCount).toBeGreaterThanOrEqual(4); // produce, dairy, protein, pantry
    
    // Verify total items count is reasonable for 3 weeks
    const progressText = await page.textContent('[data-testid="progress-text"]');
    const totalItems = parseInt(progressText!.match(/of (\d+) completed/)![1]);
    expect(totalItems).toBeGreaterThan(20); // Should have many items for 3 weeks
  });

  test('should handle shopping list caching correctly', async ({ page }) => {
    // Step 1: Generate shopping list for the first time
    await page.goto('/meal-plans/test-meal-plan-1');
    const firstGenerationStart = Date.now();
    
    await page.click('[data-testid="generate-shopping-list-button"]');
    await expect(page.locator('[data-testid="shopping-list-notification"]')).toBeVisible();
    
    const firstGenerationTime = Date.now() - firstGenerationStart;
    console.log(`First generation took: ${firstGenerationTime}ms`);

    // Step 2: Generate the same shopping list again (should be cached)
    await page.reload();
    await page.goto('/meal-plans/test-meal-plan-1');
    
    const cachedGenerationStart = Date.now();
    await page.click('[data-testid="generate-shopping-list-button"]');
    await expect(page.locator('[data-testid="shopping-list-notification"]')).toBeVisible();
    
    const cachedGenerationTime = Date.now() - cachedGenerationStart;
    console.log(`Cached generation took: ${cachedGenerationTime}ms`);
    
    // Cached generation should be significantly faster
    expect(cachedGenerationTime).toBeLessThan(firstGenerationTime * 0.5);

    // Step 3: Verify cache invalidation on meal plan update
    await page.click('[data-testid="meal-slot-monday-lunch"]');
    await page.click('[data-testid="change-recipe-button"]');
    await page.click('[data-testid="recipe-option-2"]');
    await page.click('[data-testid="save-meal-slot-button"]');
    
    // New generation should take longer as cache is invalidated
    const postUpdateStart = Date.now();
    await expect(page.locator('[data-testid="shopping-list-notification"]')).toBeVisible({
      timeout: 5000,
    });
    const postUpdateTime = Date.now() - postUpdateStart;
    
    console.log(`Post-update generation took: ${postUpdateTime}ms`);
    expect(postUpdateTime).toBeGreaterThan(cachedGenerationTime);
  });

  test('should integrate shopping notifications correctly', async ({ page }) => {
    // Step 1: Ensure notifications are visible on meal plan actions
    await page.goto('/meal-plans/create');
    await page.fill('[data-testid="week-start-date"]', '2024-01-01');
    await page.click('[data-testid="generate-meal-plan-button"]');
    
    // Wait for meal plan generation
    await expect(page.locator('[data-testid="meal-plan-grid"]')).toBeVisible();
    
    // Step 2: Verify shopping list generation notification
    const notification = page.locator('[data-testid="shopping-list-notification"]');
    await expect(notification).toBeVisible();
    
    const notificationText = await notification.textContent();
    expect(notificationText).toContain('generated');
    
    // Step 3: Verify notification dismissal
    await page.click('[data-testid="dismiss-notification-button"]');
    await expect(notification).not.toBeVisible();
    
    // Step 4: Verify notification auto-dismissal
    await page.click('[data-testid="meal-slot-monday-breakfast"]');
    await page.click('[data-testid="change-recipe-button"]');
    await page.click('[data-testid="recipe-option-1"]');
    await page.click('[data-testid="save-meal-slot-button"]');
    
    // New notification should appear
    const updateNotification = page.locator('[data-testid="shopping-list-notification"]');
    await expect(updateNotification).toBeVisible();
    
    // Wait for auto-dismissal (5 seconds)
    await expect(updateNotification).not.toBeVisible({ timeout: 6000 });
  });
});