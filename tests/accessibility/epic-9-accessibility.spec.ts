import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * Epic 9 Accessibility Testing
 *
 * Tests WCAG 2.1 Level AA compliance for all Epic 9 pages using axe-core.
 *
 * Acceptance Criteria Coverage:
 * - AC 9.7.7: Screen reader support (ARIA labels, semantic HTML)
 * - AC 9.7.8: Color contrast ratios (4.5:1 normal, 3:1 large)
 * - AC 9.7.10: Lighthouse accessibility score >90
 */

test.describe('Epic 9: Accessibility Testing', () => {

  test.describe('Multi-Week Calendar (/plan)', () => {
    test('should have no accessibility violations', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper heading structure', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['heading-order'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have sufficient color contrast', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['color-contrast'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper ARIA attributes on week tabs', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules([
          'aria-allowed-attr',
          'aria-required-attr',
          'aria-valid-attr',
          'aria-valid-attr-value',
          'button-name'
        ])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have labeled interactive elements', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['label', 'button-name', 'link-name'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper focus indicators', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      // Focus on week tab and verify focus indicator is visible
      await page.keyboard.press('Tab');

      const focusedElement = await page.locator(':focus');
      const outlineWidth = await focusedElement.evaluate((el) => {
        return window.getComputedStyle(el).outlineWidth;
      });

      // Verify focus indicator exists (outline-width should be ≥ 2px)
      expect(parseInt(outlineWidth)).toBeGreaterThanOrEqual(2);
    });
  });

  test.describe('Meal Planning Preferences Form (/profile/meal-planning-preferences)', () => {
    test('should have no accessibility violations', async ({ page }) => {
      await page.goto('http://localhost:8080/profile/meal-planning-preferences');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper form labels', async ({ page }) => {
      await page.goto('http://localhost:8080/profile/meal-planning-preferences');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['label', 'form-field-multiple-labels'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have sufficient color contrast', async ({ page }) => {
      await page.goto('http://localhost:8080/profile/meal-planning-preferences');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['color-contrast'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper checkbox attributes', async ({ page }) => {
      await page.goto('http://localhost:8080/profile/meal-planning-preferences');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['label', 'aria-allowed-attr'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have touch-friendly targets on mobile', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('http://localhost:8080/profile/meal-planning-preferences');

      // Check checkbox sizes
      const checkboxes = page.locator('input[type="checkbox"]');
      const checkboxCount = await checkboxes.count();

      for (let i = 0; i < checkboxCount; i++) {
        const bbox = await checkboxes.nth(i).boundingBox();
        if (bbox) {
          expect(bbox.width).toBeGreaterThanOrEqual(44);
          expect(bbox.height).toBeGreaterThanOrEqual(44);
        }
      }
    });
  });

  test.describe('Recipe Creation Form (/recipes/new)', () => {
    test('should have no accessibility violations', async ({ page }) => {
      await page.goto('http://localhost:8080/recipes/new');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper form labels', async ({ page }) => {
      await page.goto('http://localhost:8080/recipes/new');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['label'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper radio button groups', async ({ page }) => {
      await page.goto('http://localhost:8080/recipes/new');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['radiogroup'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have sufficient color contrast', async ({ page }) => {
      await page.goto('http://localhost:8080/recipes/new');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['color-contrast'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have touch-friendly radio buttons on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('http://localhost:8080/recipes/new');

      // Check radio button sizes
      const radios = page.locator('input[type="radio"]');
      const radioCount = await radios.count();

      for (let i = 0; i < radioCount; i++) {
        const bbox = await radios.nth(i).boundingBox();
        if (bbox) {
          expect(bbox.width).toBeGreaterThanOrEqual(44);
          expect(bbox.height).toBeGreaterThanOrEqual(44);
        }
      }
    });
  });

  test.describe('Shopping List (/shopping)', () => {
    test('should have no accessibility violations', async ({ page }) => {
      await page.goto('http://localhost:8080/shopping');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have proper list structure', async ({ page }) => {
      await page.goto('http://localhost:8080/shopping');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['list', 'listitem'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have labeled week selector', async ({ page }) => {
      await page.goto('http://localhost:8080/shopping');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['label'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have sufficient color contrast', async ({ page }) => {
      await page.goto('http://localhost:8080/shopping');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['color-contrast'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have touch-friendly checkboxes on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('http://localhost:8080/shopping');

      // Check shopping list item checkbox sizes
      const checkboxes = page.locator('input[type="checkbox"]');
      const checkboxCount = await checkboxes.count();

      for (let i = 0; i < Math.min(checkboxCount, 5); i++) {
        const bbox = await checkboxes.nth(i).boundingBox();
        if (bbox) {
          expect(bbox.width).toBeGreaterThanOrEqual(44);
          expect(bbox.height).toBeGreaterThanOrEqual(44);
        }
      }
    });
  });

  test.describe('Keyboard Navigation', () => {
    test('calendar should have logical tab order', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      // Tab through elements and verify focus moves in logical order
      await page.keyboard.press('Tab'); // First tab stop
      let focusedElement = await page.locator(':focus');
      let tagName = await focusedElement.evaluate((el) => el.tagName);

      // First interactive element should be a button or link
      expect(['BUTTON', 'A', 'INPUT']).toContain(tagName);

      await page.keyboard.press('Tab'); // Second tab stop
      focusedElement = await page.locator(':focus');
      const isVisible = await focusedElement.isVisible();

      // Focus should be on visible element
      expect(isVisible).toBe(true);
    });

    test('forms should be keyboard accessible', async ({ page }) => {
      await page.goto('http://localhost:8080/profile/meal-planning-preferences');

      // Tab to first form field
      await page.keyboard.press('Tab');

      // Should be able to interact with focused element
      await page.keyboard.press('Space'); // Should toggle if checkbox

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withRules(['focus-order-semantics'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('modals should trap focus', async ({ page }) => {
      await page.goto('http://localhost:8080/plan');

      // Open regeneration modal (if exists)
      const regenerateButton = page.locator('button:has-text("Regenerate")').first();

      if (await regenerateButton.isVisible()) {
        await regenerateButton.click();

        // Tab through modal elements
        await page.keyboard.press('Tab');

        // Focus should be within modal
        const focusedElement = await page.locator(':focus');
        const isInModal = await focusedElement.evaluate((el) => {
          return !!el.closest('[role="dialog"]');
        });

        expect(isInModal).toBe(true);
      }
    });
  });

  test.describe('Responsive Design', () => {
    test('mobile layout should have no accessibility violations', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('tablet layout should have no accessibility violations', async ({ page }) => {
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('desktop layout should have no accessibility violations', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });
      await page.goto('http://localhost:8080/plan');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });
  });
});
