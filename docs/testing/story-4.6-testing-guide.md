# Story 4.6: Advance Preparation Reminder System - Testing Guide

## Overview

This guide provides step-by-step instructions for testing the Advance Preparation Reminder System, including both manual testing procedures and automated test scenarios.

---

## Prerequisites

1. **Database Setup**: Run migrations to create notifications and push_subscriptions tables
2. **VAPID Keys**: Configure VAPID keys in `config/default.toml` (or they'll be auto-generated)
3. **User Account**: Create a test user account
4. **Recipes**: Have at least 7 favorite recipes with varying `advance_prep_hours` values
5. **Browser**: Chrome, Firefox, or Safari (for push notification testing)

---

## Quick Start Testing

### 1. Start the Application

```bash
# From project root
cargo run
```

Application should start on `http://localhost:3000`

### 2. Generate VAPID Keys (if needed)

```bash
# Install web-push CLI
npm install -g web-push

# Generate VAPID keys
web-push generate-vapid-keys

# Copy output to config/default.toml under [vapid] section
```

---

## Test Scenarios

## Scenario 1: Generate Meal Plan with Advance Prep Recipes

**Objective**: Verify that notifications are scheduled when a meal plan is generated.

### Steps:

1. **Login** to the application at `http://localhost:3000/login`

2. **Add recipes with advance prep** (if not already done):
   - Go to `/recipes/new`
   - Create recipe with `advance_prep_hours = 24` (e.g., "Marinated Chicken")
   - Create recipe with `advance_prep_hours = 8` (e.g., "Overnight Oats")
   - Create recipe with `advance_prep_hours = 2` (e.g., "Chilled Salad")
   - Favorite all recipes (click heart icon)

3. **Generate a meal plan**:
   - Navigate to `/plan`
   - Click "Generate Meal Plan" button
   - Wait for meal plan to generate

4. **Verify notifications were created**:

   **Option A: Check database directly**
   ```bash
   sqlite3 imkitchen.db "SELECT id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, status FROM notifications ORDER BY scheduled_time;"
   ```

   **Expected**: You should see notifications scheduled for meals with advance prep

   **Option B: Check via API**
   ```bash
   # Get your auth token first (login and check browser DevTools > Application > Cookies)
   curl -H "Cookie: session=YOUR_TOKEN" http://localhost:3000/api/notifications
   ```

   **Expected**: JSON array of pending notifications

5. **Verify notification page**:
   - Navigate to `/notifications`
   - Should see list of pending prep reminders
   - Each notification should show:
     - Reminder type badge (⏰ Advance Prep, 🌅 Morning Prep, or 👨‍🍳 Day-of Prep)
     - Prep hours required
     - Scheduled time
     - Dismiss and Snooze buttons

---

## Scenario 2: Test Notification Scheduling Logic

**Objective**: Verify reminder times are calculated correctly.

### Test Cases:

#### Test Case 2.1: 24h+ Advance Prep (Day Before, 9am)

**Setup**: Recipe with `advance_prep_hours = 24`, meal scheduled for Thursday 6pm

**Expected Reminder Time**: Wednesday 9am

**Verification**:
```sql
SELECT
    recipe_id,
    meal_date,
    scheduled_time,
    reminder_type,
    prep_hours
FROM notifications
WHERE prep_hours >= 24;
```

**Expected**:
- `reminder_type = "advance_prep"`
- `scheduled_time` is 9am the day before `meal_date`

---

#### Test Case 2.2: 4-23h Advance Prep (Same Day, X hours before)

**Setup**: Recipe with `advance_prep_hours = 4`, meal scheduled for Wednesday 6pm

**Expected Reminder Time**: Wednesday 2pm (6pm - 4h)

**Verification**:
```sql
SELECT
    recipe_id,
    meal_date,
    scheduled_time,
    reminder_type,
    prep_hours
FROM notifications
WHERE prep_hours >= 4 AND prep_hours < 24;
```

**Expected**:
- `reminder_type = "advance_prep"`
- `scheduled_time` is 4 hours before meal time

---

#### Test Case 2.3: <4h Prep (Morning Reminder, 9am)

**Setup**: Recipe with `advance_prep_hours = 2`, meal scheduled for Wednesday 6pm

**Expected Reminder Time**: Wednesday 9am

**Verification**:
```sql
SELECT
    recipe_id,
    meal_date,
    scheduled_time,
    reminder_type,
    prep_hours
FROM notifications
WHERE prep_hours < 4 AND prep_hours > 0;
```

**Expected**:
- `reminder_type = "morning"`
- `scheduled_time` is 9am on `meal_date`

---

## Scenario 3: Test Dismiss Functionality

**Objective**: Verify users can dismiss notifications.

### Steps:

1. Navigate to `/notifications`
2. Find a pending notification
3. Click "✓ Done" button
4. **Expected**: Notification disappears with animation
5. **Verify in database**:
   ```sql
   SELECT id, status, dismissed_at
   FROM notifications
   WHERE status = 'dismissed';
   ```

### Security Test: Unauthorized Dismiss

**Test**: Try to dismiss another user's notification

```bash
# Get notification ID from User A
USER_A_NOTIF_ID="<notification-id>"

# Login as User B, try to dismiss User A's notification
curl -X POST \
  -H "Cookie: session=USER_B_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  http://localhost:3000/api/notifications/${USER_A_NOTIF_ID}/dismiss
```

**Expected**: `403 Permission Denied` (not `404 Not Found` - security fix!)

---

## Scenario 4: Test Snooze Functionality

**Objective**: Verify users can snooze notifications for 1h, 2h, or 4h.

### Steps:

1. Navigate to `/notifications`
2. Find a pending notification
3. Select "2 hours" from snooze dropdown
4. **Expected**: Form auto-submits, notification updates

5. **Verify in database**:
   ```sql
   SELECT id, scheduled_time, status
   FROM notifications
   WHERE status = 'pending'
   ORDER BY scheduled_time;
   ```

   **Expected**: `scheduled_time` should be ~2 hours in the future

### Test Invalid Snooze Duration

```bash
# Try invalid snooze duration (not 1, 2, or 4)
curl -X POST \
  -H "Cookie: session=YOUR_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "duration_hours=10" \
  http://localhost:3000/api/notifications/NOTIF_ID/snooze
```

**Expected**: Error response (command validation rejects invalid durations)

---

## Scenario 5: Test Push Notification Subscription

**Objective**: Verify users can subscribe to browser push notifications.

### Steps:

1. Navigate to `/notifications`
2. If push not enabled, you should see blue banner "Enable Push Notifications"
3. Click "Enable Notifications" button

4. **Expected Browser Behavior**:
   - Browser shows native permission prompt
   - "localhost:3000 wants to show notifications"
   - Click "Allow"

5. **Verify in browser DevTools**:
   - Open DevTools > Application > Service Workers
   - Should see `/static/sw.js` registered and activated

6. **Verify subscription in database**:
   ```sql
   SELECT user_id, endpoint, created_at
   FROM push_subscriptions;
   ```

   **Expected**: One row with your user_id and an HTTPS endpoint

### Security Test: Invalid Endpoint

**Test**: Try to subscribe with non-HTTPS endpoint

```bash
curl -X POST \
  -H "Cookie: session=YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint": "http://insecure.com/push",
    "p256dh_key": "BEl62iUYgUivxIkv69yViEuiBIa",
    "auth_key": "test123"
  }' \
  http://localhost:3000/api/notifications/subscribe
```

**Expected**: `400 Bad Request` - "Push endpoint must use HTTPS protocol"

### Security Test: Invalid Base64 Keys

```bash
curl -X POST \
  -H "Cookie: session=YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint": "https://fcm.googleapis.com/fcm/send/xyz",
    "p256dh_key": "NOT_VALID_BASE64!!!",
    "auth_key": "test123"
  }' \
  http://localhost:3000/api/notifications/subscribe
```

**Expected**: `400 Bad Request` - "Invalid p256dh_key: must be valid base64"

---

## Scenario 6: Test Background Worker

**Objective**: Verify background worker sends notifications at scheduled time.

### Manual Test (Fast-Forward Time):

1. **Create a notification due in 2 minutes**:
   ```sql
   INSERT INTO notifications (
     id, user_id, recipe_id, meal_date, scheduled_time,
     reminder_type, prep_hours, status
   ) VALUES (
     'test-notification-123',
     'your-user-id',
     'some-recipe-id',
     '2025-10-20',
     datetime('now', '+2 minutes'),  -- Due in 2 minutes
     'morning',
     2,
     'pending'
   );
   ```

2. **Subscribe to push notifications** (see Scenario 5)

3. **Watch server logs**:
   ```bash
   # In terminal where server is running, watch for:
   # "Processing X pending notifications..."
   # "Web Push notification sent successfully..."
   ```

4. **Expected after 2 minutes**:
   - Server logs show notification processed
   - Browser shows push notification (if subscribed)
   - Database shows `status = 'sent'`

### Verify Worker Logs:

```bash
# Check logs for background worker activity
tail -f logs/imkitchen.log | grep -E "notification|worker"
```

**Expected Log Entries**:
```
INFO  Starting notification background worker...
INFO  Notification worker started
INFO  Processing 3 pending notifications...
INFO  Web Push notification sent successfully to user_id=user123
```

---

## Scenario 7: Test Deep Linking from Notification

**Objective**: Verify clicking "View Recipe" opens recipe with prep highlighted.

### Steps:

1. Navigate to `/notifications`
2. Find a notification
3. Click "View Recipe" link

4. **Expected Behavior**:
   - Browser navigates to `/recipes/{recipe_id}?notification_id={notification_id}`
   - Page auto-scrolls to "Instructions" section
   - Instructions section has yellow highlight with pulse animation
   - Prep section border is orange (#f59e0b)

5. **Verify in browser DevTools**:
   - Inspect Instructions section
   - Should have class `prep-highlighted`
   - CSS animation should be running

### Test Auto-Scroll:

- Open DevTools > Console
- Navigate to recipe via notification link
- Check that `scrollIntoView()` was called:
  ```javascript
  // Should see smooth scroll animation to #prep-instructions
  ```

---

## Scenario 8: Test Notification Page UI/UX

**Objective**: Verify notification page displays correctly and is accessible.

### Visual Checks:

1. **Empty State**:
   - Dismiss all notifications
   - Reload `/notifications`
   - Should see: 🔔 icon, "All caught up!", "View Meal Plan →" link

2. **Notification Cards**:
   - Color-coded by type:
     - 🟠 Orange border: Advance Prep
     - 🟡 Yellow border: Morning Prep
     - 🔵 Blue border: Day-of Prep
   - Each card shows: reminder type badge, prep hours, scheduled time
   - Hover effect: card slides right slightly

3. **Dismiss Animation**:
   - Click "✓ Done"
   - Card fades out and slides right
   - Card disappears after 300ms

### Accessibility Checks:

1. **Keyboard Navigation**:
   - Tab through page
   - Can focus on Dismiss button (Enter to activate)
   - Can focus on Snooze dropdown (Arrow keys to select, Enter to submit)

2. **Screen Reader** (if available):
   - Each notification has `role="article"`
   - Notification has descriptive `aria-label`

---

## Automated Testing

### Unit Tests (Already Included)

Run existing unit tests:

```bash
cargo test --package notifications

# Expected: 8 passed
# - test_calculate_reminder_time_24h_prep
# - test_calculate_reminder_time_4h_prep
# - test_calculate_reminder_time_1h_prep
# - test_calculate_reminder_time_default_meal_time
# - test_determine_reminder_type
# - test_generate_notification_body_24h_prep
# - test_generate_notification_body_8h_prep
# - test_generate_notification_body_1h_prep
```

### Integration Test Example

Add to `crates/notifications/tests/integration_test.rs`:

```rust
#[tokio::test]
async fn test_schedule_and_dismiss_notification() {
    // Setup test database
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Schedule a notification
    let cmd = ScheduleReminderCommand {
        user_id: "test-user".to_string(),
        recipe_id: "recipe-1".to_string(),
        meal_date: "2025-10-20".to_string(),
        scheduled_time: "2025-10-19T09:00:00Z".to_string(),
        reminder_type: "advance_prep".to_string(),
        prep_hours: 24,
        prep_task: Some("Marinate chicken".to_string()),
    };

    let notif_id = schedule_reminder(cmd, &executor).await.unwrap();

    // Verify notification exists
    let notifications = get_user_pending_notifications(&pool, "test-user").await.unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].id, notif_id);

    // Dismiss notification
    let dismiss_cmd = DismissReminderCommand {
        notification_id: notif_id.clone(),
    };
    dismiss_reminder(dismiss_cmd, &executor).await.unwrap();

    // Verify notification dismissed
    let notifications = get_user_pending_notifications(&pool, "test-user").await.unwrap();
    assert_eq!(notifications.len(), 0);
}
```

---

## End-to-End Test with Playwright (Optional)

Create `tests/e2e/notifications.spec.ts`:

```typescript
import { test, expect } from '@playwright/test';

test('generate meal plan and view notifications', async ({ page }) => {
  // Login
  await page.goto('http://localhost:3000/login');
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'password123');
  await page.click('button[type="submit"]');

  // Generate meal plan
  await page.goto('http://localhost:3000/plan');
  await page.click('text=Generate Meal Plan');
  await page.waitForSelector('text=Meal plan generated');

  // View notifications
  await page.goto('http://localhost:3000/notifications');

  // Check notification card appears
  const notificationCard = page.locator('.notification-card').first();
  await expect(notificationCard).toBeVisible();

  // Dismiss notification
  await notificationCard.locator('button:has-text("Done")').click();

  // Verify notification dismissed
  await page.waitForTimeout(500); // Wait for animation
  await expect(notificationCard).not.toBeVisible();
});

test('subscribe to push notifications', async ({ page, context }) => {
  // Grant notification permission
  await context.grantPermissions(['notifications']);

  await page.goto('http://localhost:3000/notifications');

  // Click enable notifications
  await page.click('button:has-text("Enable Notifications")');

  // Wait for success message
  await expect(page.locator('text=Notifications Enabled!')).toBeVisible();
});
```

---

## Performance Testing

### Load Test: Multiple Notifications

```bash
# Create 100 notifications for stress testing
for i in {1..100}; do
  sqlite3 imkitchen.db "INSERT INTO notifications (id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, status) VALUES ('notif-${i}', 'user-123', 'recipe-1', '2025-10-20', datetime('now', '+${i} minutes'), 'morning', 2, 'pending');"
done

# Check page load time
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000/notifications
```

**Expected**: Page loads in <500ms even with 100 notifications

---

## Troubleshooting

### Issue: No notifications created after generating meal plan

**Check**:
1. Ensure recipes have `advance_prep_hours > 0`
2. Check server logs for evento subscription errors
3. Verify `meal_plan_subscriptions` is registered in main.rs

**Debug Query**:
```sql
SELECT * FROM evento_events
WHERE aggregator_type = 'meal_planning/MealPlanAggregate'
ORDER BY timestamp DESC LIMIT 5;
```

### Issue: Push notifications not appearing

**Check**:
1. Browser permissions granted?
2. Service worker registered? (DevTools > Application > Service Workers)
3. VAPID keys configured correctly?
4. Check push subscription in database

**Debug**:
```javascript
// In browser console
navigator.serviceWorker.ready.then(reg => {
  reg.pushManager.getSubscription().then(sub => {
    console.log('Subscription:', sub);
  });
});
```

### Issue: Notifications showing "Recipe Prep Reminder" instead of recipe name

**Known Issue**: Recipe title not yet fetched in read model (H2 in code review)

**Workaround**: Will be fixed in next iteration

---

## Test Checklist

Use this checklist to ensure complete testing:

- [ ] ✅ Notifications created when meal plan generated
- [ ] ✅ Correct reminder times for 24h+ prep (day before, 9am)
- [ ] ✅ Correct reminder times for 4-23h prep (X hours before)
- [ ] ✅ Correct reminder times for <4h prep (morning, 9am)
- [ ] ✅ Dismiss button removes notification
- [ ] ✅ Snooze updates scheduled time correctly
- [ ] ✅ Push subscription saves to database
- [ ] ✅ Background worker processes pending notifications
- [ ] ✅ Deep linking highlights prep instructions
- [ ] ✅ Auto-scroll works on recipe page
- [ ] ✅ Empty state displays when no notifications
- [ ] ✅ Notification cards color-coded correctly
- [ ] ✅ Security: Cannot dismiss other user's notifications
- [ ] ✅ Security: Invalid push endpoints rejected
- [ ] ✅ Security: Invalid base64 keys rejected
- [ ] ✅ Accessibility: Keyboard navigation works
- [ ] ✅ Performance: Page loads quickly with many notifications

---

## Next Steps

After completing manual testing:

1. **Add integration tests** for all routes
2. **Add E2E tests** with Playwright
3. **Set up monitoring** for notification delivery rates
4. **Configure production VAPID keys**
5. **Test on mobile browsers** (especially Safari iOS)

---

**Testing Guide Version**: 1.0
**Story**: 4.6 - Advance Preparation Reminder System
**Last Updated**: 2025-10-18
