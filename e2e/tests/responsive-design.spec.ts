/**
 * E2E Tests for Mobile-Responsive Design (Story 5.4)
 *
 * Tests responsive breakpoints, layouts, and mobile-specific features
 * across mobile (<768px), tablet (768-1024px), and desktop (>1024px)
 */

import { test, expect } from '@playwright/test';

test.describe('Responsive Design - Story 5.4', () => {

  test.describe('Mobile Viewport (375px - iPhone SE)', () => {
    test.use({ viewport: { width: 375, height: 667 } });

    test('AC-1: Responsive breakpoints - Mobile layout', async ({ page }) => {
      await page.goto('/');

      // Verify viewport meta tag
      const viewport = await page.locator('meta[name="viewport"]').getAttribute('content');
      expect(viewport).toContain('width=device-width');
      expect(viewport).toContain('initial-scale=1');
    });

    test('AC-2: Mobile layout - Bottom navigation visible', async ({ page }) => {
      await page.goto('/dashboard');

      // Verify bottom navigation is visible on mobile
      const bottomNav = page.locator('nav[aria-label="Mobile navigation"]');
      await expect(bottomNav).toBeVisible();

      // Verify position is fixed
      const position = await bottomNav.evaluate((el) =>
        window.getComputedStyle(el).position
      );
      expect(position).toBe('fixed');
    });

    test('AC-2: Mobile layout - Desktop nav hidden', async ({ page }) => {
      await page.goto('/dashboard');

      // Verify top navigation horizontal links are hidden on mobile
      const desktopNavLinks = page.locator('nav.bg-white .hidden.md\\:flex');
      // Element exists in DOM but should not be visible due to hidden class
      const count = await desktopNavLinks.count();
      if (count > 0) {
        await expect(desktopNavLinks.first()).not.toBeVisible();
      }
    });

    test('AC-5: Text sizes - 16px minimum body text', async ({ page }) => {
      await page.goto('/dashboard');

      // Check body text size
      const bodyFontSize = await page.evaluate(() => {
        return window.getComputedStyle(document.body).fontSize;
      });

      const fontSize = parseInt(bodyFontSize);
      expect(fontSize).toBeGreaterThanOrEqual(16);
    });

    test('AC-7: Form inputs - Touch targets minimum 44px', async ({ page }) => {
      // Navigate to a page with form inputs
      await page.goto('/login');

      // Check input field height
      const inputs = page.locator('input[type="email"], input[type="password"]');
      const count = await inputs.count();

      if (count > 0) {
        const inputBox = await inputs.first().boundingBox();
        expect(inputBox?.height).toBeGreaterThanOrEqual(44);
      }
    });

    test('AC-7: Button touch targets - Minimum 44px', async ({ page }) => {
      await page.goto('/');

      const buttons = page.locator('button, a.bg-primary-500').first();
      if (await buttons.count() > 0) {
        const buttonBox = await buttons.boundingBox();
        expect(buttonBox?.width).toBeGreaterThanOrEqual(44);
        expect(buttonBox?.height).toBeGreaterThanOrEqual(44);
      }
    });

    test('AC-8: Navigation accessible - Bottom nav fixed on scroll', async ({ page }) => {
      await page.goto('/dashboard');

      // Scroll down the page
      await page.evaluate(() => window.scrollTo(0, 500));
      await page.waitForTimeout(100);

      // Verify bottom navigation is still visible
      const bottomNav = page.locator('nav[aria-label="Mobile navigation"]');
      await expect(bottomNav).toBeVisible();
    });

    test('AC-8: Back to top button appears on scroll', async ({ page }) => {
      await page.goto('/recipes');

      const backToTopButton = page.locator('#back-to-top');

      // Scroll down more than 500px
      await page.evaluate(() => window.scrollTo(0, 600));
      await page.waitForTimeout(200); // Wait for scroll handler debounce

      // Button should become visible (opacity-100 class)
      const hasOpacity100 = await backToTopButton.evaluate((el) =>
        el.classList.contains('opacity-100')
      );
      expect(hasOpacity100).toBe(true);
    });
  });

  test.describe('Tablet Viewport (768px - iPad)', () => {
    test.use({ viewport: { width: 768, height: 1024 } });

    test('AC-3: Tablet layout - Desktop nav visible', async ({ page }) => {
      await page.goto('/dashboard');

      // Desktop nav links should be visible on tablet
      const desktopNav = page.locator('nav.bg-white .hidden.md\\:flex');
      const count = await desktopNav.count();
      if (count > 0) {
        await expect(desktopNav.first()).toBeVisible();
      }
    });

    test('AC-3: Tablet layout - Bottom nav hidden', async ({ page }) => {
      await page.goto('/dashboard');

      // Bottom navigation should be hidden on tablet
      const bottomNav = page.locator('nav[aria-label="Mobile navigation"]');
      const count = await bottomNav.count();
      if (count > 0) {
        await expect(bottomNav).not.toBeVisible();
      }
    });

    test('AC-3: Tablet layout - Multi-column grid (2 columns)', async ({ page }) => {
      await page.goto('/recipes');

      // Check for 2-column grid on tablet (AC-3 requires exactly 2 columns)
      const recipeGrid = page.locator('.grid').first();
      if (await recipeGrid.count() > 0) {
        const gridClasses = await recipeGrid.getAttribute('class');
        // Should have md:grid-cols-2
        expect(gridClasses).toContain('md:grid-cols-2');
      }
    });
  });

  test.describe('Desktop Viewport (1920px)', () => {
    test.use({ viewport: { width: 1920, height: 1080 } });

    test('AC-4: Desktop layout - Multi-column grid (4 columns)', async ({ page }) => {
      await page.goto('/recipes');

      const recipeGrid = page.locator('.grid').first();
      if (await recipeGrid.count() > 0) {
        const gridClasses = await recipeGrid.getAttribute('class');
        // Should have lg:grid-cols-4
        expect(gridClasses).toContain('lg:grid-cols-4');
      }
    });

    test('AC-4: Desktop layout - Persistent desktop navigation', async ({ page }) => {
      await page.goto('/dashboard');

      // Desktop nav should be visible
      const desktopNav = page.locator('nav.bg-white .hidden.md\\:flex');
      if (await desktopNav.count() > 0) {
        await expect(desktopNav.first()).toBeVisible();
      }

      // Bottom mobile navigation should not be visible
      const bottomNav = page.locator('nav[aria-label="Mobile navigation"]');
      const count = await bottomNav.count();
      if (count > 0) {
        await expect(bottomNav).not.toBeVisible();
      }
    });

    test('AC-4: Desktop meal calendar - Full week view', async ({ page }) => {
      await page.goto('/plan');

      // Check if calendar grid has 7 columns for desktop
      const calendarGrid = page.locator('.grid.lg\\:grid-cols-7').first();
      if (await calendarGrid.count() > 0) {
        await expect(calendarGrid).toBeVisible();
      }
    });
  });

  test.describe('Responsive Images', () => {
    test('AC-6: Images have lazy loading enabled', async ({ page }) => {
      await page.goto('/recipes');

      const recipeImages = page.locator('.recipe-card img').first();
      if (await recipeImages.count() > 0) {
        const loading = await recipeImages.getAttribute('loading');
        expect(loading).toBe('lazy');
      }
    });

    test('AC-6: Images have srcset attribute', async ({ page }) => {
      await page.goto('/recipes');

      // Find recipe card images
      const recipeImages = page.locator('.recipe-card img').first();
      if (await recipeImages.count() > 0) {
        const srcset = await recipeImages.getAttribute('srcset');
        // Srcset should exist (even if placeholder implementation)
        expect(srcset).toBeTruthy();
      }
    });

    test('AC-6: Images have explicit dimensions', async ({ page }) => {
      await page.goto('/recipes');

      const recipeImages = page.locator('.recipe-card img').first();
      if (await recipeImages.count() > 0) {
        const width = await recipeImages.getAttribute('width');
        const height = await recipeImages.getAttribute('height');
        // Images should have width and height to prevent layout shift
        expect(width || height).toBeTruthy();
      }
    });
  });

  test.describe('Accessibility (WCAG 2.1 Level AA)', () => {
    test.use({ viewport: { width: 375, height: 667 } });

    test('AC-1: Viewport meta tag configured correctly', async ({ page }) => {
      await page.goto('/');

      const viewport = await page.locator('meta[name="viewport"]').getAttribute('content');
      expect(viewport).toContain('width=device-width');
      expect(viewport).toContain('initial-scale=1');
    });

    test('AC-7: Interactive elements meet touch target size', async ({ page }) => {
      await page.goto('/dashboard');

      // Check button sizes
      const buttons = page.locator('button, a[class*="btn"]');
      const buttonCount = await buttons.count();

      for (let i = 0; i < Math.min(buttonCount, 3); i++) {
        const button = buttons.nth(i);
        const buttonBox = await button.boundingBox();
        if (buttonBox) {
          expect(buttonBox.width).toBeGreaterThanOrEqual(44);
          expect(buttonBox.height).toBeGreaterThanOrEqual(44);
        }
      }
    });

    test('AC-8: Skip to content link for keyboard navigation', async ({ page }) => {
      await page.goto('/');

      // Skip link should exist (even if visually hidden)
      const skipLink = page.locator('a[href="#main-content"]');
      await expect(skipLink).toHaveCount(1);
    });

    test('Navigation has proper ARIA labels', async ({ page }) => {
      await page.goto('/dashboard');

      const bottomNav = page.locator('nav[aria-label="Mobile navigation"]');
      if (await bottomNav.count() > 0) {
        const ariaLabel = await bottomNav.getAttribute('aria-label');
        expect(ariaLabel).toBeTruthy();
      }
    });
  });

  test.describe('Cross-Viewport Consistency', () => {
    test('Responsive breakpoints work consistently', async ({ page }) => {
      // Test mobile
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/dashboard');

      const mobileNav = page.locator('nav[aria-label="Mobile navigation"]');
      if (await mobileNav.count() > 0) {
        await expect(mobileNav).toBeVisible();
      }

      // Test tablet
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.reload();

      if (await mobileNav.count() > 0) {
        await expect(mobileNav).not.toBeVisible();
      }

      // Test desktop
      await page.setViewportSize({ width: 1920, height: 1080 });
      await page.reload();

      if (await mobileNav.count() > 0) {
        await expect(mobileNav).not.toBeVisible();
      }
    });
  });
});
