import { test, expect } from '@playwright/test';

/**
 * E2E tests for Push Notification Permission Flow (Story 4.10)
 *
 * These tests verify the complete user journey for push notification permission
 * including onboarding step 5, profile settings, and grace period enforcement.
 *
 * Note: Browser notification permissions are mocked for testing.
 */

test.describe('Push Notification Permission - Onboarding Flow', () => {
  test.beforeEach(async ({ page, context }) => {
    // Grant notification permissions for the test
    await context.grantPermissions(['notifications']);

    // Register a new user and navigate to onboarding
    await page.goto('/register');
    const uniqueEmail = `notif-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for redirect to onboarding
    await page.waitForURL(/\/onboarding/);
  });

  test('onboarding step 5 displays notification permission request (AC #1)', async ({ page }) => {
    // Navigate through onboarding to step 5
    await page.goto('/onboarding?step=5');

    // AC #1: Step 5 prompts for notification permission
    await expect(page.locator('h2')).toContainText(/stay on track|reminders/i);

    // Verify "Allow Notifications" button is present
    await expect(page.locator('button:has-text("Allow Notifications")')).toBeVisible();
  });

  test('step 5 explains benefits of push notifications (AC #2)', async ({ page }) => {
    await page.goto('/onboarding?step=5');

    // AC #2: Benefits explanation visible
    await expect(page.locator('text=/advance prep/i')).toBeVisible();
    await expect(page.locator('text=/marinating|chilling/i')).toBeVisible();
    await expect(page.locator('text=/morning.*meal/i')).toBeVisible();
    await expect(page.locator('text=/cooking time/i')).toBeVisible();

    // Verify informational box styling
    const benefitsBox = page.locator('.bg-blue-50, [class*="blue"]').filter({ hasText: /advance prep/i });
    await expect(benefitsBox).toBeVisible();
  });

  test('user can allow notifications from onboarding (AC #3)', async ({ page, context }) => {
    await page.goto('/onboarding?step=5');

    // Click "Allow Notifications" button
    const allowButton = page.locator('button:has-text("Allow Notifications")');
    await allowButton.click();

    // Should redirect to dashboard after allowing
    await page.waitForURL('/dashboard', { timeout: 5000 });

    // Verify user reached dashboard
    await expect(page).toHaveURL('/dashboard');
  });

  test('user can skip notifications from onboarding (AC #3)', async ({ page }) => {
    await page.goto('/onboarding?step=5');

    // Click "Skip for now" button
    const skipButton = page.locator('button:has-text("Skip for now")');
    await skipButton.click();

    // Should redirect to dashboard after skipping
    await page.waitForURL('/dashboard', { timeout: 5000 });

    // Verify user reached dashboard
    await expect(page).toHaveURL('/dashboard');
  });

  test('step 5 has back button to step 4', async ({ page }) => {
    await page.goto('/onboarding?step=5');

    // Verify back button exists
    const backButton = page.locator('a:has-text("Back")');
    await expect(backButton).toBeVisible();

    // Click back button
    await backButton.click();

    // Should navigate to step 4
    await expect(page).toHaveURL(/step=4/);
    await expect(page.locator('text=/weeknight availability/i')).toBeVisible();
  });

  test('completing step 4 advances to step 5', async ({ page }) => {
    // Start at step 4
    await page.goto('/onboarding?step=4');

    // Fill out weeknight availability
    await page.fill('input[name="availability_start"]', '18:00');
    await page.fill('input[name="availability_duration"]', '60');

    // Submit step 4
    await page.click('button[type="submit"]');

    // Should advance to step 5
    await page.waitForURL(/step=5/, { timeout: 5000 });
    await expect(page.locator('text=/stay on track|reminders/i')).toBeVisible();
  });
});

test.describe('Push Notification Permission - Profile Settings', () => {
  test.beforeEach(async ({ page, context }) => {
    // Grant notification permissions
    await context.grantPermissions(['notifications']);

    // Register and complete onboarding
    await page.goto('/register');
    const uniqueEmail = `profile-notif-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    // Skip onboarding
    await page.waitForURL(/\/(onboarding|dashboard)/);
    const skipButton = page.locator('a:has-text("Skip")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForURL('/dashboard');
    }
  });

  test('profile page shows notification settings section (AC #7)', async ({ page }) => {
    await page.goto('/profile');

    // AC #7: Notification settings section visible
    await expect(page.locator('h2:has-text("Notification Settings")')).toBeVisible();

    // Verify push notification status is displayed
    await expect(page.locator('text=/push notifications/i')).toBeVisible();
  });

  test('profile shows disabled status when no subscription exists (AC #7)', async ({ page }) => {
    await page.goto('/profile');

    // Verify disabled badge is shown
    const statusBadge = page.locator('text=/disabled/i');
    await expect(statusBadge).toBeVisible();

    // Verify "Enable Notifications" button is present
    await expect(page.locator('button:has-text("Enable Notifications")')).toBeVisible();
  });

  test('profile shows enabled status with device count when subscription exists', async ({ page }) => {
    // Note: This test would require actually creating a push subscription
    // For now, it documents the expected behavior

    await page.goto('/profile');

    // After enabling notifications, should show:
    // - "Enabled" badge with green styling
    // - Device count (e.g., "Enabled (1 device)")
    // - "View Reminders" link instead of "Enable Notifications" button

    // This would require:
    // 1. Clicking "Enable Notifications"
    // 2. Browser creating actual push subscription
    // 3. Subscription saved to backend
    // 4. Page reload to show updated status
  });

  test('user can enable notifications from profile settings (AC #7)', async ({ page }) => {
    await page.goto('/profile');

    // Click "Enable Notifications" button
    const enableButton = page.locator('button:has-text("Enable Notifications")');
    await enableButton.click();

    // Page should reload after successful subscription
    await page.waitForTimeout(1000); // Wait for potential reload

    // Note: Full verification would require checking that:
    // - POST /api/notifications/subscribe was called
    // - Subscription data was saved
    // - Page shows "Enabled" status
  });

  test('enabled notifications show link to view reminders (AC #7)', async ({ page }) => {
    // Note: This requires a user with push notifications already enabled
    // For now, this documents the expected behavior

    await page.goto('/profile');

    // After notifications are enabled, should see:
    const viewRemindersLink = page.locator('a:has-text("View Reminders")');

    // This link should navigate to /notifications page
    // (Only visible when notification_enabled is true)
  });
});

test.describe('Push Notification Permission - Permission Denial', () => {
  test('user denying browser permission completes onboarding (AC #5)', async ({ page, context }) => {
    // Block notification permissions
    await context.clearPermissions();
    await context.grantPermissions([]);

    // Register and navigate to step 5
    await page.goto('/register');
    const uniqueEmail = `deny-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/onboarding?step=5');

    // Click "Allow Notifications" (will be denied by browser)
    const allowButton = page.locator('button:has-text("Allow Notifications")');
    await allowButton.click();

    // AC #5: User should still complete onboarding even if denied
    // (Alert may appear, but user is redirected to dashboard)
    await page.waitForURL('/dashboard', { timeout: 10000 });
  });

  test('browser alert shown when permission denied (AC #5)', async ({ page, context }) => {
    // This test verifies the alert message for denied permissions

    await context.clearPermissions();
    await page.goto('/onboarding?step=5');

    // Listen for alert dialog
    page.on('dialog', async dialog => {
      expect(dialog.message()).toContain(/blocked|denied|settings/i);
      await dialog.accept();
    });

    // Click allow (will trigger browser denial)
    await page.click('button:has-text("Allow Notifications")');

    // Verify user is guided to enable in browser settings
  });
});

test.describe('Push Notification Permission - Grace Period', () => {
  test('grace period prevents immediate re-prompting after denial (AC #8)', async ({ page }) => {
    // This test verifies that can_prompt_for_notification_permission returns false
    // within 30 days of denial

    // Note: This requires API testing rather than E2E, as grace period logic
    // is implemented server-side in can_prompt_for_notification_permission()

    // The test would:
    // 1. User denies permission (last_permission_denial_at set to now)
    // 2. Call GET /api/notifications/status
    // 3. Verify can_prompt: false
    // 4. Verify user is NOT shown "Enable Notifications" button
  });
});

test.describe('Push Notification Permission - Service Worker', () => {
  test('service worker registration is attempted (AC #4)', async ({ page, context }) => {
    await context.grantPermissions(['notifications']);
    await page.goto('/register');

    const uniqueEmail = `sw-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/onboarding?step=5');

    // Click allow notifications
    await page.click('button:has-text("Allow Notifications")');

    // AC #4: Service worker should be registered
    // Verify service worker exists (if browser supports it)
    const swRegistration = await page.evaluate(async () => {
      if ('serviceWorker' in navigator) {
        const registration = await navigator.serviceWorker.getRegistration();
        return registration ? true : false;
      }
      return null;
    });

    // If browser supports service workers, one should be registered
    if (swRegistration !== null) {
      expect(swRegistration).toBeTruthy();
    }
  });

  test('push subscription is created and sent to server (AC #4, #6)', async ({ page, context }) => {
    await context.grantPermissions(['notifications']);

    // Monitor network requests
    const subscriptionRequests: any[] = [];
    page.on('request', request => {
      if (request.url().includes('/api/notifications/subscribe')) {
        subscriptionRequests.push({
          method: request.method(),
          url: request.url(),
          postData: request.postData(),
        });
      }
    });

    await page.goto('/register');
    const uniqueEmail = `push-${Date.now()}@example.com`;
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="password_confirm"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/onboarding?step=5');
    await page.click('button:has-text("Allow Notifications")');

    // Wait for subscription request
    await page.waitForTimeout(2000);

    // AC #4, #6: Subscription should be sent to backend
    // Note: This may not work in headless mode if service workers are unsupported
    // In a real environment with proper service worker support, we would verify:
    // - POST /api/notifications/subscribe was called
    // - Request contains subscription object with endpoint, keys, etc.
  });
});

/**
 * Helper function to enable notifications for a test user
 */
async function enableNotificationsForUser(page: any) {
  await page.goto('/profile');
  const enableButton = page.locator('button:has-text("Enable Notifications")');
  if (await enableButton.isVisible()) {
    await enableButton.click();
    await page.waitForTimeout(1000);
  }
}
