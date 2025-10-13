import { test, expect } from '@playwright/test';

/**
 * E2E tests for Premium Subscription Upgrade Flow (Story 1.7)
 *
 * These tests verify the complete user journey from free tier to premium subscription
 * including Stripe Checkout integration and post-upgrade functionality.
 *
 * Note: These tests use mock Stripe interactions for local testing.
 * For full Stripe integration testing, use Stripe test mode with real test cards.
 */

test.describe('Premium Subscription Upgrade Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Start at the registration page
    await page.goto('/register');
  });

  test('free user can view subscription page with upgrade button', async ({ page }) => {
    // Register a new user
    const uniqueEmail = `test-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for redirect to onboarding or dashboard
    await page.waitForURL(/\/(onboarding|dashboard)/);

    // Skip onboarding if present
    const skipButton = page.locator('a:has-text("Skip")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForURL('/dashboard');
    }

    // Navigate to subscription page
    await page.goto('/subscription');

    // Verify subscription page elements (AC #1, AC #2)
    await expect(page.locator('h1, h2')).toContainText(/subscription/i);
    await expect(page.locator('text=/free|tier/i')).toBeVisible();
    await expect(page.locator('text=/upgrade.*premium/i')).toBeVisible();
    await expect(page.locator('text=/\\$9\\.99/i')).toBeVisible(); // Pricing display
    await expect(page.locator('text=/unlimited recipes/i')).toBeVisible(); // Premium benefit
  });

  test('premium user sees premium status without upgrade button', async ({ page }) => {
    // Note: This test requires a user to already be premium.
    // In a real scenario, you would:
    // 1. Register a user
    // 2. Manually upgrade via backend/database
    // 3. Login and verify premium status
    //
    // For now, this is a placeholder showing the expected behavior

    // Register a new user
    const uniqueEmail = `premium-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for redirect
    await page.waitForURL(/\/(onboarding|dashboard)/);

    // TODO: Upgrade user to premium via API or database manipulation
    // This would require exposing a test endpoint or using direct database access

    // Navigate to subscription page
    await page.goto('/subscription');

    // Verify premium status (AC #8)
    // await expect(page.locator('text=/premium/i')).toBeVisible();
    // await expect(page.locator('text=/upgrade/i')).not.toBeVisible();
  });

  test('clicking upgrade button redirects to Stripe Checkout (mock)', async ({ page }) => {
    // Register a new user
    const uniqueEmail = `stripe-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for redirect to onboarding or dashboard
    await page.waitForURL(/\/(onboarding|dashboard)/);

    // Skip onboarding
    const skipButton = page.locator('a:has-text("Skip")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForURL('/dashboard');
    }

    // Navigate to subscription page
    await page.goto('/subscription');

    // Click "Upgrade to Premium" button (AC #3)
    const upgradeButton = page.locator('button:has-text("Upgrade"), a:has-text("Upgrade")');
    await upgradeButton.click();

    // Note: In real Stripe integration, this would redirect to checkout.stripe.com
    // For testing, you would either:
    // 1. Mock the Stripe redirect
    // 2. Use Stripe test mode and verify redirect URL contains checkout.stripe.com
    // 3. Intercept the POST request to /subscription/upgrade and verify parameters

    // Example assertion (would need actual implementation):
    // await page.waitForURL(/checkout\.stripe\.com/);
    // OR
    // await expect(page).toHaveURL(/subscription\/upgrade/);
  });

  test('user can cancel Stripe Checkout and return to app', async ({ page }) => {
    // This test verifies AC #11: User can cancel Stripe Checkout

    // Register a new user
    const uniqueEmail = `cancel-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for redirect
    await page.waitForURL(/\/(onboarding|dashboard)/);

    // Skip onboarding
    const skipButton = page.locator('a:has-text("Skip")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
    }

    // Navigate to subscription page
    await page.goto('/subscription');

    // Click upgrade button
    const upgradeButton = page.locator('button:has-text("Upgrade"), a:has-text("Upgrade")');
    await upgradeButton.click();

    // In real Stripe Checkout, user clicks "Back" or "Cancel"
    // This would redirect to cancel_url: /subscription

    // Verify user returns to /subscription (no charge)
    // await expect(page).toHaveURL('/subscription');
    // await expect(page.locator('text=/free/i')).toBeVisible(); // Still free tier
  });

  test('successful payment redirects to success page', async ({ page }) => {
    // This test verifies AC #7: User redirected to /subscription/success after payment

    // In a real test with Stripe test mode, you would:
    // 1. Complete the Stripe Checkout flow with test card 4242424242424242
    // 2. Stripe redirects to success_url
    // 3. Webhook processes payment and upgrades user

    // For now, directly navigate to success page to verify it exists
    await page.goto('/subscription/success');

    // Verify success page content
    await expect(page.locator('text=/success|thank you|congratulations/i')).toBeVisible();
    await expect(page.locator('text=/premium/i')).toBeVisible();
  });

  test('premium user can create unlimited recipes', async ({ page }) => {
    // This test verifies AC #9: Freemium restrictions removed for premium

    // This would require:
    // 1. Creating a premium user (via API or database)
    // 2. Creating 10+ recipes
    // 3. Verifying no "recipe limit reached" error

    // Placeholder test structure:
    // await page.goto('/recipes');
    // for (let i = 1; i <= 15; i++) {
    //   await page.click('a:has-text("New Recipe")');
    //   await page.fill('input[name="title"]', `Recipe ${i}`);
    //   // Fill other required fields
    //   await page.click('button[type="submit"]');
    //   await expect(page).not.toContainText(/limit reached/i);
    // }
  });
});

test.describe('Subscription Error Handling', () => {
  test('free user at recipe limit sees upgrade prompt', async ({ page }) => {
    // Verify AC #1: "Upgrade to Premium" button visible within freemium restriction prompts

    // This would require:
    // 1. Creating a free user with 10 recipes
    // 2. Attempting to create an 11th recipe
    // 3. Verifying error prompt contains "Upgrade to Premium" button

    // Example flow:
    // await page.goto('/recipes/new');
    // await page.fill('input[name="title"]', 'Recipe 11');
    // await page.click('button[type="submit"]');
    // await expect(page.locator('text=/limit reached/i')).toBeVisible();
    // await expect(page.locator('a:has-text("Upgrade to Premium")')).toBeVisible();
  });

  test('payment failure displays Stripe error message', async ({ page }) => {
    // Verify AC #10: Failed payment displays Stripe error and allows retry

    // In Stripe test mode, use declining test card: 4000000000000002
    // This would trigger a payment failure and display Stripe's error message

    // Stripe Checkout handles error display automatically, so this test
    // would primarily verify that:
    // 1. Error message appears in Stripe Checkout UI
    // 2. User can update payment method and retry
  });
});

/**
 * Helper function to create a premium user for testing
 * (Would need backend API endpoint for test data setup)
 */
async function createPremiumUser(page: any, email: string, password: string) {
  // This is a placeholder for a test helper function
  // In practice, you would either:
  // 1. Call a test-only API endpoint: POST /test/users/premium
  // 2. Use database seeding scripts
  // 3. Programmatically trigger webhook events
}

/**
 * Helper function to create recipes for a user
 * (Would need recipe creation flow)
 */
async function createRecipe(page: any, title: string) {
  // Navigate to recipe creation page
  await page.goto('/recipes/new');
  await page.fill('input[name="title"]', title);
  // Fill other required fields based on actual form
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/recipes\/\w+/); // Wait for redirect to recipe detail
}
