# Story 4.10: Push Notification Permission Flow

Status: ContextReadyDraft

## Story

As a **user**,
I want **to enable push notifications for reminders**,
so that **I receive timely preparation alerts**.

## Acceptance Criteria

1. Onboarding includes notification permission prompt
2. Prompt explains benefits: "Get reminders for advance prep and cooking times"
3. User can allow, deny, or skip
4. If allowed, register service worker and subscription
5. If denied, fall back to in-app notifications only
6. User can change permission in browser settings
7. Settings page shows current notification status
8. Grace period: don't re-prompt if user denied within last 30 days

## Tasks / Subtasks

- [ ] Implement notification permission request flow (AC: 1, 2, 3)
  - [ ] Add notification permission prompt to onboarding flow
  - [ ] Create clear permission request UI with benefit explanation
  - [ ] Add "Allow", "Deny", and "Skip" action buttons
  - [ ] Track user's permission decision in user preferences
  - [ ] Handle browser permission API (Notification.requestPermission())

- [ ] Implement service worker registration (AC: 4)
  - [ ] Register service worker when permission granted
  - [ ] Create push subscription using PushManager API
  - [ ] Extract subscription endpoint, p256dh key, and auth key
  - [ ] POST subscription data to /api/push/subscribe endpoint
  - [ ] Store subscription in push_subscriptions table
  - [ ] Handle subscription creation failures gracefully

- [ ] Implement fallback for denied permissions (AC: 5, 6)
  - [ ] Store "denied" status in user preferences
  - [ ] Show in-app notification banner instead of push notifications
  - [ ] Display "Enable Notifications" link in settings
  - [ ] Provide instructions for changing permission in browser settings
  - [ ] Browser-specific instructions (Chrome, Firefox, Safari)

- [ ] Add notification status to settings page (AC: 7)
  - [ ] Query current push subscription status from database
  - [ ] Display "Notifications Enabled" or "Notifications Disabled" status
  - [ ] Show subscription count (number of devices with active subscriptions)
  - [ ] Add "Manage Notifications" button linking to browser settings
  - [ ] Display last notification sent timestamp (if any)

- [ ] Implement grace period for re-prompting (AC: 8)
  - [ ] Track denial timestamp in user preferences
  - [ ] Check if 30 days elapsed since last denial
  - [ ] Only show permission prompt if grace period passed
  - [ ] Add override option in settings ("Try Again")
  - [ ] Reset grace period after user manually re-enables

- [ ] Create push subscription endpoints (AC: 4)
  - [ ] POST /api/push/subscribe - Store new subscription
  - [ ] DELETE /api/push/unsubscribe - Remove subscription
  - [ ] GET /api/push/status - Check current subscription status
  - [ ] Validate subscription payload (endpoint URL, keys)
  - [ ] Security: Verify user owns subscription before deletion

- [ ] Add push subscription database schema (AC: 4)
  - [ ] Migration: Create push_subscriptions table
  - [ ] Columns: id, user_id, endpoint, p256dh_key, auth_key, created_at, last_used_at
  - [ ] Unique constraint on (user_id, endpoint) - one subscription per browser
  - [ ] Index on user_id for fast subscription lookups

- [ ] Update onboarding flow UI (AC: 1, 2, 3)
  - [ ] Add notification permission step to onboarding wizard
  - [ ] Design permission prompt with benefit explanation
  - [ ] Add icon/illustration for notification feature
  - [ ] Use TwinSpark for inline permission request without page reload
  - [ ] Track onboarding completion with/without notification permission

- [ ] Add integration tests (AC: all)
  - [ ] Test: Permission request creates push subscription (AC #4)
  - [ ] Test: Denied permission stores "denied" status (AC #5)
  - [ ] Test: Settings page shows correct subscription status (AC #7)
  - [ ] Test: Grace period prevents re-prompting before 30 days (AC #8)
  - [ ] Test: Manual retry bypasses grace period (AC #8)
  - [ ] Test: Multiple devices can subscribe (unique endpoint constraint)
  - [ ] Test: Security - user cannot access another user's subscriptions

- [ ] Service worker notification handling (AC: 4)
  - [ ] Update static/js/sw.js with push event handler
  - [ ] Display notification when push event received
  - [ ] Add notification actions: "View Recipe", "Dismiss"
  - [ ] Handle notificationclick event for deep links
  - [ ] Cache notification icons for offline display

## Dev Notes

### Architecture Patterns and Constraints

**Web Push API Integration**:
- Use browser-native Push API (no vendor lock-in, works on Chrome, Firefox, Edge, Safari 16+)
- VAPID keys required for push subscription (generate once, store in config)
- Subscription endpoint URL unique per browser/device
- Encryption keys (p256dh, auth) required for secure push delivery

**Push Subscription Flow**:
```
1. User clicks "Allow Notifications" in onboarding
   ↓
2. Browser displays native permission prompt
   ↓
3. If granted:
   - Register service worker (/sw.js)
   - Create push subscription via navigator.serviceWorker.ready.pushManager.subscribe()
   - Extract endpoint, p256dh key, auth key from subscription
   ↓
4. POST to /api/push/subscribe with subscription data
   ↓
5. Server stores subscription in push_subscriptions table
   ↓
6. User preferences updated: notification_permission = 'granted'
```

**Fallback Strategy**:
- If permission denied: Show in-app notification banner on dashboard
- Banner content: "Enable notifications to get cooking reminders"
- Link to settings page with browser-specific instructions
- No push notifications sent, but in-app notifications visible when logged in

**Event Sourcing**:
- `PushSubscriptionCreated` event when subscription stored
- `PushSubscriptionDeleted` event when subscription removed
- `NotificationPermissionChanged` event when user changes permission
- Aggregate: `NotificationPreferencesAggregate` (extends notifications crate)

**Read Model**:
- `push_subscriptions` table stores active subscriptions per user
- Query: `get_push_subscriptions_for_user(user_id)` returns all active subscriptions
- Status query: Check if user has any active subscriptions (notification_enabled = subscriptions.count() > 0)

**Grace Period Tracking**:
- Store `last_permission_denial_at` timestamp in user preferences
- Check: `now() - last_permission_denial_at < 30 days` before showing prompt
- Reset timestamp when user manually re-enables in settings
- Override: "Try Again" button in settings bypasses grace period

### Source Tree Components to Touch

**Existing Files to Modify**:
```
crates/notifications/src/commands.rs
   Add SubscribeToPushCommand struct
   Add UnsubscribeFromPushCommand struct
   Implement subscribe_to_push() handler
   Implement unsubscribe_from_push() handler

crates/notifications/src/events.rs
   Add PushSubscriptionCreated event (user_id, endpoint, keys, created_at)
   Add PushSubscriptionDeleted event (user_id, endpoint, deleted_at)
   Add NotificationPermissionChanged event (user_id, permission_status, changed_at)

crates/notifications/src/aggregate.rs
   Add push_subscription_created handler
   Add push_subscription_deleted handler
   Track subscription count in aggregate state

crates/notifications/src/read_model.rs
   Add get_push_subscriptions_for_user() query
   Add get_push_subscription_status() query (returns enabled/disabled + count)
   Add project_push_subscription_created() projection handler
   Add project_push_subscription_deleted() projection handler

crates/user/src/aggregate.rs
   Add notification_permission_status field to UserProfile
   Add last_permission_denial_at field for grace period tracking
   Add notification_permission_changed event handler

src/routes/push.rs (NEW FILE)
   Create POST /api/push/subscribe endpoint
   Create DELETE /api/push/unsubscribe endpoint
   Create GET /api/push/status endpoint
   Validate subscription payload (endpoint, p256dh_key, auth_key)
   Security: Verify user owns subscription

src/routes/profile.rs
   Add notification preferences section to settings page
   Display current push subscription status
   Add "Enable Notifications" / "Manage Notifications" buttons
   Link to browser-specific permission instructions

templates/pages/onboarding.html
   Add notification permission step (step 5 after availability)
   Benefit explanation: "Get reminders for advance prep and cooking times"
   "Allow", "Skip" buttons (browser handles "Deny")
   Track permission decision

templates/pages/profile.html
   Add "Notifications" section to settings
   Display status: "Enabled (2 devices)" or "Disabled"
   Show "Enable Notifications" button if disabled
   Show last notification timestamp if available
   Link to browser settings for permission management

static/js/sw.js (service worker)
   Add push event handler: self.addEventListener('push', event => ...)
   Parse push payload (notification title, body, action_url)
   Display notification: self.registration.showNotification(...)
   Add notificationclick handler for deep links
   Cache notification icons

static/js/push-subscription.js (NEW FILE)
   requestNotificationPermission() function
   subscribeToPush() function (calls PushManager API)
   sendSubscriptionToServer() function (POST to /api/push/subscribe)
   unsubscribeToPush() function
   getSubscriptionStatus() function
```

**New Files to Create**:
```
src/routes/push.rs
   Push subscription API endpoints

static/js/push-subscription.js
   Client-side push subscription logic

tests/push_notification_permission_tests.rs
   Integration tests for Story 4.10
```

**Database Schema Changes**:
```sql
-- Create push_subscriptions table (if not exists from Epic 4 planning)
CREATE TABLE IF NOT EXISTS push_subscriptions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  endpoint TEXT NOT NULL,           -- Browser push endpoint URL
  p256dh_key TEXT NOT NULL,         -- Encryption key
  auth_key TEXT NOT NULL,           -- Authentication key
  created_at TEXT NOT NULL,
  last_used_at TEXT,                -- Track active subscriptions
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE(user_id, endpoint)         -- One subscription per browser/device
);

CREATE INDEX idx_push_subscriptions_user ON push_subscriptions(user_id);

-- Add to users table (or user_profiles if separate)
ALTER TABLE users ADD COLUMN notification_permission_status TEXT DEFAULT 'not_asked';
   -- Values: 'not_asked' | 'granted' | 'denied' | 'skipped'
ALTER TABLE users ADD COLUMN last_permission_denial_at TEXT;
```

### Testing Standards Summary

**TDD Approach**:
1. Write failing test for POST /api/push/subscribe endpoint
2. Implement SubscribeToPushCommand and handler
3. Write failing test for PushSubscriptionCreated event emission
4. Implement event handler and projection
5. Write failing test for permission denied tracking (AC #5, #8)
6. Implement grace period logic in onboarding/settings
7. Write failing test for settings page status display (AC #7)
8. Implement settings page query and UI
9. Write failing test for multi-device subscriptions
10. Verify unique constraint and subscription count

**Test Coverage Targets**:
- commands.rs subscribe_to_push(): 90%
- Grace period logic: 85%
- Settings status query: 85%
- Integration tests covering all 8 acceptance criteria

**Integration Test Examples**:
```rust
#[tokio::test]
async fn test_allow_permission_creates_push_subscription() {
    // Setup: User in onboarding flow
    // Action: POST /api/push/subscribe with valid subscription data
    // Assert: PushSubscriptionCreated event emitted, row in push_subscriptions table
}

#[tokio::test]
async fn test_denied_permission_stores_status() {
    // Setup: User denies permission in browser
    // Action: POST /api/push/permission-denied
    // Assert: User preferences updated with status='denied', timestamp stored
}

#[tokio::test]
async fn test_settings_shows_subscription_status() {
    // Setup: User has 2 active push subscriptions
    // Action: GET /profile (settings page)
    // Assert: Page displays "Enabled (2 devices)"
}

#[tokio::test]
async fn test_grace_period_prevents_reprompt() {
    // Setup: User denied permission 10 days ago
    // Action: Visit onboarding again
    // Assert: Permission prompt NOT shown (grace period active)
}

#[tokio::test]
async fn test_manual_retry_bypasses_grace_period() {
    // Setup: User denied permission 10 days ago
    // Action: Click "Try Again" in settings
    // Assert: Permission prompt shown, grace period reset
}

#[tokio::test]
async fn test_multiple_devices_can_subscribe() {
    // Setup: User has subscription from Chrome
    // Action: POST /api/push/subscribe from Firefox (different endpoint)
    // Assert: Two subscriptions exist, both active
}

#[tokio::test]
async fn test_user_cannot_delete_another_users_subscription() {
    // Setup: User A has subscription
    // Action: User B attempts DELETE /api/push/unsubscribe with User A's subscription ID
    // Assert: 403 Forbidden error returned
}
```

### Project Structure Notes

**Alignment with solution-architecture.md**:

This story extends the notifications domain crate (established in Stories 4.6-4.9) with push subscription management. The implementation follows Web Push API standards (RFC 8030) using VAPID authentication for vendor-neutral push notifications.

**Module Organization**:
- Notifications crate: Push subscription aggregate, commands, events, read model
- User crate: Notification permission preferences in user profile
- Routes: New `push.rs` module for subscription API endpoints
- Static assets: Service worker (`sw.js`) and client-side push subscription logic (`push-subscription.js`)

**Naming Conventions**:
- Command: `SubscribeToPushCommand`, `UnsubscribeFromPushCommand` (PascalCase imperative)
- Event: `PushSubscriptionCreated`, `PushSubscriptionDeleted` (PascalCase past tense)
- Functions: `subscribe_to_push()`, `get_push_subscriptions_for_user()` (snake_case)
- Table: `push_subscriptions` (snake_case)

**Detected Conflicts/Variances**:
- Onboarding flow currently has 4 steps (Story 1.4: dietary, household, skill, availability)
- This story adds notification permission as step 5 (or optional final step)
- Resolution: Make notification permission OPTIONAL in onboarding (can be skipped, enabled later in settings)

**Lessons Learned from Story 4.9**:
- Use evento ULID as subscription_id for consistency with aggregate lookup
- Implement comprehensive security checks (user ownership validation)
- Track status transitions carefully (not_asked → granted/denied/skipped)
- TwinSpark integration for inline permission request without full page reload

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-4.md#Story 11: Push Notification Permission Flow] - Web Push API integration, VAPID keys, subscription storage
- [Source: docs/solution-architecture.md#9.3 Push Notifications] - Web Push API architecture, notification payload format
- [Source: docs/solution-architecture.md#11.3 Key Integrations] - Web Push external integration

**Epic Context**:
- [Source: docs/epics.md#Story 4.10] - User story, acceptance criteria, technical notes
- [Source: docs/epics.md#Epic 4: Shopping and Preparation Orchestration] - Notification system goals

**Related Stories**:
- [Source: docs/stories/story-1.4.md] - User Profile Creation (Onboarding) - prerequisite, establishes onboarding wizard flow
- [Source: docs/stories/story-4.6.md] - Advance Preparation Reminder System - uses push subscriptions for notification delivery
- [Source: docs/stories/story-4.7.md] - Morning Preparation Reminders - uses push subscriptions
- [Source: docs/stories/story-4.8.md] - Day-of Cooking Reminders - uses push subscriptions

**Web Push API Standards**:
- [RFC 8030 - Generic Event Delivery Using HTTP Push](https://datatracker.ietf.org/doc/html/rfc8030)
- [VAPID (Voluntary Application Server Identification)](https://datatracker.ietf.org/doc/html/rfc8292)
- [MDN Web Push API](https://developer.mozilla.org/en-US/docs/Web/API/Push_API)
- [MDN Notifications API](https://developer.mozilla.org/en-US/docs/Web/API/Notifications_API)
- [web-push crate documentation](https://docs.rs/web-push/latest/web_push/)

**Browser Compatibility**:
- Chrome 50+ (full support)
- Firefox 44+ (full support)
- Edge 17+ (full support)
- Safari 16+ (iOS 16.4+, macOS 13+) - added push notification support
- Opera 37+ (full support)

## Dev Agent Record

### Context Reference

- /home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.10.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List

---

## Change Log

| Date | Author | Change Summary |
|------|--------|----------------|
| 2025-10-18 | Bob (SM) | Initial story creation from epics.md and tech-spec-epic-4.md |
