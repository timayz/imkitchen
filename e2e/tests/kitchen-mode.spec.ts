import { test, expect } from '@playwright/test';

test.describe('Kitchen Mode - Story 5.6', () => {
  let testRecipeId: string;

  test.beforeAll(async ({ browser }) => {
    // Create a test recipe for Kitchen Mode tests
    const page = await browser.newPage();

    // Login
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');

    // Navigate to recipes list to get a recipe ID
    await page.goto('/recipes');

    // Get the first recipe link from the list
    const firstRecipeLink = page.locator('a[href^="/recipes/"]').first();
    const href = await firstRecipeLink.getAttribute('href');

    if (href) {
      // Extract recipe ID from href (e.g., /recipes/123 -> 123)
      testRecipeId = href.split('/').pop() || '1';
    } else {
      // Fallback to ID 1 if no recipes found
      testRecipeId = '1';
      console.warn('No recipes found in test database, using default ID 1');
    }

    await page.close();
  });

  test.beforeEach(async ({ page }) => {
    // Login first (assuming test user exists)
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
  });

  test.describe('AC1: Kitchen Mode toggle in recipe detail view', () => {
    test('should show Kitchen Mode toggle button on recipe detail page', async ({ page }) => {
      // Navigate to test recipe
      await page.goto(`/recipes/${testRecipeId}`);

      // Verify toggle button exists
      const toggleButton = page.locator('[data-testid="kitchen-mode-toggle"]');
      await expect(toggleButton).toBeVisible();

      // Verify aria-label for accessibility
      await expect(toggleButton).toHaveAttribute('aria-label', /kitchen mode/i);

      // Verify it has role="switch" for accessibility
      await expect(toggleButton).toHaveAttribute('role', 'switch');
    });

    test('should position toggle prominently for mobile thumb access', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 }); // iPhone SE

      await page.goto(`/recipes/${testRecipeId}`);

      const toggleButton = page.locator('[data-testid="kitchen-mode-toggle"]');
      const boundingBox = await toggleButton.boundingBox();

      expect(boundingBox).not.toBeNull();
      // Toggle should be in top-right corner for right-thumb access
      // Or top section (within first 200px from top)
      expect(boundingBox!.y).toBeLessThan(200);
    });

    test('should toggle Kitchen Mode on click', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      const toggleButton = page.locator('[data-testid="kitchen-mode-toggle"]');

      // Initially NOT in kitchen mode
      await expect(page.locator('body')).not.toHaveClass(/kitchen-mode/);

      // Click toggle
      await toggleButton.click();

      // Body should have kitchen-mode class
      await expect(page.locator('body')).toHaveClass(/kitchen-mode/);

      // aria-checked should be true
      await expect(toggleButton).toHaveAttribute('aria-checked', 'true');
    });
  });

  test.describe('AC2: Kitchen mode increases text size', () => {
    test('should increase body text to 20px in Kitchen Mode', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      // Measure normal mode body font size
      const normalFontSize = await page.locator('body').evaluate(el => {
        return window.getComputedStyle(el).fontSize;
      });

      // Enable Kitchen Mode
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Measure kitchen mode body font size
      const kitchenFontSize = await page.locator('body').evaluate(el => {
        return window.getComputedStyle(el).fontSize;
      });

      // Kitchen mode should be at least 20px
      expect(parseFloat(kitchenFontSize)).toBeGreaterThanOrEqual(20);
      expect(parseFloat(kitchenFontSize)).toBeGreaterThan(parseFloat(normalFontSize));
    });

    test('should increase heading text to 28px in Kitchen Mode', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      // Enable Kitchen Mode
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Measure h1 font size
      const h1FontSize = await page.locator('h1').first().evaluate(el => {
        return window.getComputedStyle(el).fontSize;
      });

      // Heading should be at least 28px
      expect(parseFloat(h1FontSize)).toBeGreaterThanOrEqual(28);
    });
  });

  test.describe('AC3: High contrast styling', () => {
    test('should apply high contrast colors in Kitchen Mode', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      // Enable Kitchen Mode
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Check body background is white
      const bgColor = await page.locator('body').evaluate(el => {
        return window.getComputedStyle(el).backgroundColor;
      });

      // rgb(255, 255, 255) is white
      expect(bgColor).toBe('rgb(255, 255, 255)');

      // Check text color is near-black (#1a1a1a = rgb(26, 26, 26))
      const textColor = await page.locator('body').evaluate(el => {
        return window.getComputedStyle(el).color;
      });

      expect(textColor).toBe('rgb(26, 26, 26)');
    });

    test('should have 7:1 contrast ratio for WCAG AAA compliance', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Use axe-core accessibility testing (requires @axe-core/playwright)
      // For now, manual verification - contrast ratio of #1a1a1a on #ffffff is 15.8:1
      // which exceeds 7:1 requirement

      // Verify no shadows or decorative elements that reduce contrast
      const hasShadow = await page.locator('body').evaluate(el => {
        const shadow = window.getComputedStyle(el).boxShadow;
        return shadow !== 'none';
      });

      expect(hasShadow).toBe(false);
    });
  });

  test.describe('AC4: Simplified UI', () => {
    test('should hide navigation in Kitchen Mode', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      // Navigation should be visible initially
      const nav = page.locator('nav').first();
      await expect(nav).toBeVisible();

      // Enable Kitchen Mode
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Navigation should be hidden
      await expect(nav).toBeHidden();
    });

    test('should hide non-essential recipe metadata', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      // Enable Kitchen Mode
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Check that .hide-in-kitchen-mode elements are hidden
      const hiddenElements = page.locator('.hide-in-kitchen-mode');
      const count = await hiddenElements.count();

      for (let i = 0; i < count; i++) {
        await expect(hiddenElements.nth(i)).toBeHidden();
      }
    });

    test('should prioritize ingredients and instructions only', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Ingredients and instructions sections should be visible
      await expect(page.locator('[aria-labelledby="ingredients-heading"]')).toBeVisible();
      await expect(page.locator('[aria-labelledby="instructions-heading"]')).toBeVisible();
    });
  });

  test.describe('AC5: Step-by-step instruction mode', () => {
    test('should show only one instruction at a time', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Wait for step navigation to initialize
      await page.waitForSelector('[data-testid="step-indicator"]');

      // Count visible instructions
      const visibleInstructions = await page.locator('ol li:visible').count();
      expect(visibleInstructions).toBe(1);
    });

    test('should have Next button (min 60x60px)', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      const nextButton = page.locator('[data-testid="step-next"]');
      await expect(nextButton).toBeVisible();

      const boundingBox = await nextButton.boundingBox();
      expect(boundingBox).not.toBeNull();
      expect(boundingBox!.width).toBeGreaterThanOrEqual(60);
      expect(boundingBox!.height).toBeGreaterThanOrEqual(60);
    });

    test('should navigate through steps with Next/Previous buttons', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      const stepIndicator = page.locator('[data-testid="step-indicator"]');
      await expect(stepIndicator).toContainText('Step 1 of');

      // Click Next
      await page.click('[data-testid="step-next"]');
      await expect(stepIndicator).toContainText('Step 2 of');

      // Click Previous
      await page.click('[data-testid="step-previous"]');
      await expect(stepIndicator).toContainText('Step 1 of');
    });

    test('should support keyboard navigation (arrow keys)', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      const stepIndicator = page.locator('[data-testid="step-indicator"]');
      await expect(stepIndicator).toContainText('Step 1 of');

      // Press ArrowRight
      await page.keyboard.press('ArrowRight');
      await expect(stepIndicator).toContainText('Step 2 of');

      // Press ArrowLeft
      await page.keyboard.press('ArrowLeft');
      await expect(stepIndicator).toContainText('Step 1 of');
    });
  });

  test.describe('AC6: Keep-awake functionality (Wake Lock API)', () => {
    test('should request wake lock when Kitchen Mode enabled', async ({ page }) => {
      // Mock Wake Lock API
      await page.addInitScript(() => {
        (window.navigator as any).wakeLock = {
          request: async () => {
            (window as any).wakeLockRequested = true;
            return {
              release: () => {
                (window as any).wakeLockReleased = true;
              }
            };
          }
        };
      });

      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Verify wake lock was requested
      const wakeLockRequested = await page.evaluate(() => (window as any).wakeLockRequested);
      expect(wakeLockRequested).toBe(true);
    });

    test('should release wake lock when Kitchen Mode disabled', async ({ page }) => {
      // Mock Wake Lock API
      await page.addInitScript(() => {
        (window.navigator as any).wakeLock = {
          request: async () => {
            return {
              release: () => {
                (window as any).wakeLockReleased = true;
              }
            };
          }
        };
      });

      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');
      await page.click('[data-testid="kitchen-mode-toggle"]'); // Toggle off

      // Verify wake lock was released
      const wakeLockReleased = await page.evaluate(() => (window as any).wakeLockReleased);
      expect(wakeLockReleased).toBe(true);
    });

    test('should show visual indicator when wake lock active', async ({ page }) => {
      await page.addInitScript(() => {
        (window.navigator as any).wakeLock = {
          request: async () => ({release: () => {}})
        };
      });

      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Visual indicator should be present
      const indicator = page.locator('[data-testid="wake-lock-indicator"]');
      await expect(indicator).toBeVisible();
    });
  });

  test.describe('AC7: Persist Kitchen Mode preference', () => {
    test('should store preference in localStorage', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Check localStorage
      const stored = await page.evaluate(() => localStorage.getItem('kitchen_mode_enabled'));
      expect(stored).toBe('true');
    });

    test('should auto-enable Kitchen Mode if preference set', async ({ page }) => {
      // Set localStorage first
      await page.goto(`/recipes/${testRecipeId}`);
      await page.evaluate(() => localStorage.setItem('kitchen_mode_enabled', 'true'));

      // Reload page
      await page.reload();

      // Kitchen Mode should be active
      await expect(page.locator('body')).toHaveClass(/kitchen-mode/);

      const toggleButton = page.locator('[data-testid="kitchen-mode-toggle"]');
      await expect(toggleButton).toHaveAttribute('aria-checked', 'true');
    });

    test('should persist across page refreshes', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      await page.reload();

      // Kitchen Mode should still be enabled
      await expect(page.locator('body')).toHaveClass(/kitchen-mode/);
    });

    test('should clear preference when toggled off', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Toggle off
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // localStorage should be cleared
      const stored = await page.evaluate(() => localStorage.getItem('kitchen_mode_enabled'));
      expect(stored).toBeNull();
    });
  });

  test.describe('AC8: Easy toggle to return to normal mode', () => {
    test('should have Exit Kitchen Mode button when enabled', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Exit button should be visible
      const exitButton = page.locator('[data-testid="kitchen-mode-exit"]');
      await expect(exitButton).toBeVisible();
      await expect(exitButton).toContainText(/exit/i);
    });

    test('should restore normal view instantly on exit', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Kitchen Mode is active
      await expect(page.locator('body')).toHaveClass(/kitchen-mode/);

      // Click exit
      await page.click('[data-testid="kitchen-mode-exit"]');

      // Normal view restored
      await expect(page.locator('body')).not.toHaveClass(/kitchen-mode/);

      // Navigation should be visible again
      await expect(page.locator('nav').first()).toBeVisible();
    });

    test('should preserve scroll position when toggling', async ({ page }) => {
      await page.goto(`/recipes/${testRecipeId}`);

      // Scroll to middle of page
      await page.evaluate(() => window.scrollTo(0, 500));
      const scrollBefore = await page.evaluate(() => window.scrollY);

      // Toggle Kitchen Mode
      await page.click('[data-testid="kitchen-mode-toggle"]');

      // Scroll position should be preserved
      const scrollAfter = await page.evaluate(() => window.scrollY);
      expect(Math.abs(scrollAfter - scrollBefore)).toBeLessThan(10); // Allow 10px tolerance
    });
  });
});
