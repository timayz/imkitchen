# Story 4.6: Advance Preparation Reminder System

Status: Approved

## Story

As a user with advance-prep recipes in my meal plan,
I want timely reminders for preparation tasks,
so that I successfully execute complex recipes.

## Acceptance Criteria

1. System scans meal plan for recipes with advance prep (marinade, rising, chilling, etc.)
2. Reminders scheduled automatically when meal plan generated
3. Reminder timing calculated from advance prep requirement and meal schedule
4. Example: "Marinate chicken 4 hours before" for Wednesday dinner → reminder sent Tuesday evening or Wednesday morning
5. Reminders delivered via push notification (if enabled)
6. Reminder displays: recipe name, specific prep task, timing guidance
7. Tapping reminder opens recipe detail with prep instructions highlighted
8. User can snooze reminder (1 hour, 2 hours, 4 hours)

## Tasks / Subtasks

- [ ] Task 1: Create notifications domain crate scaffold (AC: All)
  - [ ] Subtask 1.1: Create `crates/notifications/` directory with standard crate structure
  - [ ] Subtask 1.2: Create `Cargo.toml` with dependencies: evento, sqlx, tokio, chrono, serde, web-push
  - [ ] Subtask 1.3: Create `src/lib.rs` with module exports
  - [ ] Subtask 1.4: Create empty module files: `events.rs`, `commands.rs`, `aggregate.rs`, `read_model.rs`, `scheduler.rs`, `push.rs`
  - [ ] Subtask 1.5: Add notifications crate to workspace Cargo.toml members list
  - [ ] Subtask 1.6: Create `tests/` directory for unit tests

- [ ] Task 2: Define notification domain events (AC: #1, #2, #3)
  - [ ] Subtask 2.1: Create `ReminderScheduled` event in `events.rs` with fields: `notification_id`, `user_id`, `recipe_id`, `meal_date`, `scheduled_time`, `reminder_type` (advance_prep, morning, day_of), `prep_hours`
  - [ ] Subtask 2.2: Create `ReminderSent` event with fields: `notification_id`, `sent_at`, `delivery_status`
  - [ ] Subtask 2.3: Create `ReminderDismissed` event with fields: `notification_id`, `dismissed_at`
  - [ ] Subtask 2.4: Create `ReminderSnoozed` event with fields: `notification_id`, `snoozed_until`
  - [ ] Subtask 2.5: Create `PushSubscriptionCreated` event with fields: `subscription_id`, `user_id`, `endpoint`, `p256dh_key`, `auth_key`
  - [ ] Subtask 2.6: Add bincode Encode/Decode derives to all events
  - [ ] Subtask 2.7: Add evento::AggregatorName derives to all events

- [ ] Task 3: Implement notification commands (AC: #2, #5, #8)
  - [ ] Subtask 3.1: Create `ScheduleReminderCommand` in `commands.rs` with validation
  - [ ] Subtask 3.2: Create `SendReminderCommand` with retry logic support
  - [ ] Subtask 3.3: Create `DismissReminderCommand` for task completion
  - [ ] Subtask 3.4: Create `SnoozeReminderCommand` with duration validation (1h, 2h, 4h only)
  - [ ] Subtask 3.5: Create `SubscribeToPushCommand` for Web Push subscription
  - [ ] Subtask 3.6: Implement command handlers that create aggregates and emit events
  - [ ] Subtask 3.7: Write unit tests for each command handler (valid inputs, validation failures)

- [ ] Task 4: Build notification aggregate with evento (AC: All)
  - [ ] Subtask 4.1: Define `NotificationAggregate` struct in `aggregate.rs` with state fields
  - [ ] Subtask 4.2: Implement evento aggregator trait with event handlers
  - [ ] Subtask 4.3: Add `reminder_scheduled` event handler to set initial state
  - [ ] Subtask 4.4: Add `reminder_sent` event handler to update delivery status
  - [ ] Subtask 4.5: Add `reminder_dismissed` event handler to mark as complete
  - [ ] Subtask 4.6: Add `reminder_snoozed` event handler to update scheduled_time
  - [ ] Subtask 4.7: Write unit tests for aggregate event handling

- [ ] Task 5: Create notification read models and projections (AC: All)
  - [ ] Subtask 5.1: Define database schema in migration: `notifications` table with columns: `id`, `user_id`, `recipe_id`, `meal_date`, `scheduled_time`, `status` (pending/sent/dismissed/failed), `reminder_type`, `prep_hours`, `sent_at`, `dismissed_at`
  - [ ] Subtask 5.2: Define `push_subscriptions` table schema: `id`, `user_id`, `endpoint`, `p256dh_key`, `auth_key`, `created_at`
  - [ ] Subtask 5.3: Create projection handler `project_reminder_scheduled` in `read_model.rs`
  - [ ] Subtask 5.4: Create projection handler `project_reminder_sent`
  - [ ] Subtask 5.5: Create projection handler `project_reminder_dismissed`
  - [ ] Subtask 5.6: Create projection handler `project_push_subscription_created`
  - [ ] Subtask 5.7: Write integration tests for projections with in-memory SQLite

- [ ] Task 6: Implement reminder scheduling logic (AC: #2, #3, #4)
  - [ ] Subtask 6.1: Create `calculate_reminder_time()` function in `scheduler.rs`
  - [ ] Subtask 6.2: Logic: For advance_prep_hours >= 24h → schedule for morning (9am) of day before meal
  - [ ] Subtask 6.3: Logic: For advance_prep_hours 4-23h → schedule for (meal_time - prep_hours)
  - [ ] Subtask 6.4: Logic: For advance_prep_hours < 4h → schedule for day-of reminder (1 hour before meal)
  - [ ] Subtask 6.5: Handle edge cases: reminder time in past (schedule immediately), meal time unknown (default 6pm)
  - [ ] Subtask 6.6: Write comprehensive unit tests for `calculate_reminder_time()` with various scenarios
  - [ ] Subtask 6.7: Create evento subscription: `MealPlanGenerated` event → scan meals for advance_prep → schedule reminders

- [ ] Task 7: Integrate Web Push API (AC: #5, #6)
  - [ ] Subtask 7.1: Create `push.rs` module with VAPID key configuration (load from env vars)
  - [ ] Subtask 7.2: Implement `create_push_payload()` function to generate notification JSON
  - [ ] Subtask 7.3: Payload structure: `{ "title": "Prep Reminder", "body": "Marinate chicken tonight...", "icon": "/icon.png", "actions": [{"action": "view", "title": "View Recipe"}] }`
  - [ ] Subtask 7.4: Implement `send_push_notification()` using web-push crate
  - [ ] Subtask 7.5: Handle Web Push errors: endpoint invalid (410 Gone → delete subscription), rate limit (retry), server error (retry)
  - [ ] Subtask 7.6: Write integration tests with mocked Web Push endpoint

- [ ] Task 8: Build background notification scheduler worker (AC: #5)
  - [ ] Subtask 8.1: Create `NotificationWorker` struct in `scheduler.rs`
  - [ ] Subtask 8.2: Implement tokio interval task that polls every 1 minute
  - [ ] Subtask 8.3: Query `notifications` table for pending reminders with `scheduled_time <= now()`
  - [ ] Subtask 8.4: For each due notification, fetch user's push subscription
  - [ ] Subtask 8.5: Send push notification via Web Push API
  - [ ] Subtask 8.6: Emit `ReminderSent` or `ReminderFailed` event
  - [ ] Subtask 8.7: Implement exponential backoff retry (3 attempts: 1s, 2s, 4s)
  - [ ] Subtask 8.8: Add observability: log notification delivery attempts with tracing

- [ ] Task 9: Create HTTP routes for notifications (AC: #7, #8)
  - [ ] Subtask 9.1: Create `src/routes/notifications.rs` in root binary
  - [ ] Subtask 9.2: Implement `GET /notifications` route (list user's pending notifications)
  - [ ] Subtask 9.3: Implement `POST /notifications/:id/dismiss` route (mark reminder complete)
  - [ ] Subtask 9.4: Implement `POST /notifications/:id/snooze` route with duration parameter
  - [ ] Subtask 9.5: Implement `POST /notifications/subscribe` route to save push subscription
  - [ ] Subtask 9.6: Add authentication middleware (JWT validation)
  - [ ] Subtask 9.7: Write integration tests for all routes

- [ ] Task 10: Build notification UI templates (AC: #6, #7, #8)
  - [ ] Subtask 10.1: Create `templates/pages/notifications.html` to display pending reminders
  - [ ] Subtask 10.2: Show notification list with recipe name, prep task, scheduled time
  - [ ] Subtask 10.3: Add "Dismiss" button with TwinSpark AJAX (POST to dismiss endpoint)
  - [ ] Subtask 10.4: Add "Snooze" dropdown with options: 1h, 2h, 4h (TwinSpark form submission)
  - [ ] Subtask 10.5: Update recipe detail template to show prep task highlight when opened via notification
  - [ ] Subtask 10.6: Add push notification permission request UI in settings/profile page
  - [ ] Subtask 10.7: Implement JavaScript to request browser notification permission and POST subscription to backend

- [ ] Task 11: Generate notification message text (AC: #4, #6)
  - [ ] Subtask 11.1: Create `generate_notification_body()` function in `scheduler.rs`
  - [ ] Subtask 11.2: For 24h+ prep: "Marinate chicken tonight for {day} dinner: {recipe_title}"
  - [ ] Subtask 11.3: For 4-23h prep: "Start prep in {hours} hours for {meal}: {recipe_title}"
  - [ ] Subtask 11.4: For <4h prep: "Start cooking in 1 hour: {recipe_title}"
  - [ ] Subtask 11.5: Include recipe title, meal day/time, specific prep action (marinate, rise, chill)
  - [ ] Subtask 11.6: Write unit tests for message generation with various inputs

- [ ] Task 12: Handle notification deep linking (AC: #7)
  - [ ] Subtask 12.1: Add `notification_id` query parameter to recipe detail route: `GET /recipes/:id?notification_id=:notif_id`
  - [ ] Subtask 12.2: When notification_id present, query notification details
  - [ ] Subtask 12.3: Highlight prep instructions section in recipe template
  - [ ] Subtask 12.4: Add CSS class `.highlighted` to prep section
  - [ ] Subtask 12.5: Auto-scroll to prep section on page load (JavaScript)
  - [ ] Subtask 12.6: Test deep link flow: click notification → opens recipe → prep highlighted

- [ ] Task 13: End-to-end testing (All ACs)
  - [ ] Subtask 13.1: Write E2E test: Generate meal plan with advance prep recipe → verify reminder scheduled
  - [ ] Subtask 13.2: Test reminder timing calculation (24h prep → scheduled 9am day before)
  - [ ] Subtask 13.3: Test background worker delivery (mock current time to trigger scheduled notification)
  - [ ] Subtask 13.4: Test notification dismiss flow (click dismiss → status updated)
  - [ ] Subtask 13.5: Test notification snooze flow (snooze 1h → scheduled_time updated)
  - [ ] Subtask 13.6: Test push subscription flow (enable notifications → subscription saved)
  - [ ] Subtask 13.7: Test deep link from notification to recipe detail
  - [ ] Subtask 13.8: Verify 80%+ code coverage for notifications domain

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- NotificationAggregate manages reminder lifecycle via evento
- Events: `ReminderScheduled`, `ReminderSent`, `ReminderDismissed`, `ReminderSnoozed`, `PushSubscriptionCreated`
- Read model projections update `notifications` and `push_subscriptions` tables
- Full audit trail of all notification scheduling and delivery attempts

**CQRS Pattern:**
- Commands: `ScheduleReminderCommand`, `SendReminderCommand`, `DismissReminderCommand`, `SnoozeReminderCommand`
- Queries: Read from `notifications` table filtered by user_id and status
- Projections: Update notification status (pending → sent → dismissed)

**Background Worker Pattern:**
- Tokio interval task polls for due notifications every 1 minute
- Sends notifications via Web Push API
- Emits `ReminderSent` event on successful delivery
- Exponential backoff retry on failures (1s, 2s, 4s)

**Web Push API Integration:**
- VAPID-based push notifications (browser standard, no vendor lock-in)
- Push subscriptions stored in database (endpoint, p256dh_key, auth_key)
- Notification payload includes title, body, icon, action buttons
- Handles endpoint expiration (410 Gone → delete subscription)

**Domain Event Integration:**
- evento subscription: `MealPlanGenerated` → scan meals for advance_prep → schedule reminders
- evento subscription: `MealReplaced` → reschedule reminders if new meal has different prep time
- evento subscription: `MealPlanRegenerated` → cancel old reminders, schedule new ones

### Source Tree Components

**Notifications Domain Crate (`crates/notifications/`):**
- `src/events.rs` - Define all notification-related events
- `src/commands.rs` - Define command handlers for scheduling, sending, dismissing, snoozing
- `src/aggregate.rs` - NotificationAggregate with evento event handlers
- `src/read_model.rs` - Projections for notifications and push_subscriptions tables
- `src/scheduler.rs` - Background worker, reminder scheduling logic, notification message generation
- `src/push.rs` - Web Push API integration (VAPID, send_push_notification)
- `tests/` - Unit tests for commands, aggregates, scheduling logic

**HTTP Routes (`src/routes/notifications.rs`):**
- `GET /notifications` - List user's pending notifications
- `POST /notifications/:id/dismiss` - Mark reminder as complete
- `POST /notifications/:id/snooze` - Snooze reminder (1h, 2h, 4h)
- `POST /notifications/subscribe` - Save Web Push subscription

**Askama Templates:**
- `templates/pages/notifications.html` - Notification list page with dismiss/snooze actions
- `templates/pages/recipe-detail.html` - Updated to highlight prep section when opened via notification
- `templates/pages/profile.html` - Add push notification permission request UI

**Database Migrations:**
- `migrations/XXX_create_notifications_table.sql` - notifications table schema
- `migrations/XXX_create_push_subscriptions_table.sql` - push_subscriptions table schema

**JavaScript:**
- Push notification permission request and subscription handling
- Service worker to handle notification clicks (deep link to recipe)

### Testing Standards

**Unit Tests (`crates/notifications/tests/`):**
- Test `ScheduleReminderCommand` validation and event emission
- Test `calculate_reminder_time()` with various prep_hours and meal_times
- Test `generate_notification_body()` message generation
- Test aggregate event handlers (state transitions)
- Coverage target: 90%+ for notifications domain

**Integration Tests:**
- Test evento projections update database correctly
- Test HTTP routes (schedule, dismiss, snooze, subscribe)
- Test background worker with mocked clock (simulate time passing)
- Test Web Push API integration with mocked endpoint

**E2E Tests (`e2e/tests/notifications.spec.ts`):**
- Test full flow: Generate meal plan → verify reminder scheduled in database
- Test notification delivery (mock current time to trigger worker)
- Test dismiss and snooze actions
- Test push subscription flow (enable notifications → subscription saved)
- Test deep link from notification to recipe detail with prep highlighted

**TDD Approach:**
- Write failing test for `calculate_reminder_time()` function
- Implement function to pass test
- Write failing test for `ScheduleReminderCommand` handler
- Implement command to pass test
- Write failing E2E test for notification flow
- Implement background worker to pass test

### Project Structure Notes

**Alignment with Unified Project Structure:**
- Notifications domain follows established crate pattern (events, commands, aggregates, read_models)
- Background worker integrated into main binary startup (tokio task spawned in main.rs)
- Web Push API integration consistent with external service pattern (SMTP, Stripe, MinIO)
- Server-side rendering with Askama for notification UI

**Detected Conflicts or Variances:**
- None - notifications domain is new and integrates cleanly
- Web Push requires VAPID keys in configuration (add to config.toml and env vars)
- Background worker requires tokio runtime (already available in main.rs)

### References

**Source Documentation:**
- [Epic 4 Story 4.6 Requirements - docs/epics.md:998-1020]
- [Notification Domain Architecture - docs/tech-spec-epic-4.md:127-137]
- [Web Push API Pattern - docs/solution-architecture.md:1110-1153]
- [Background Worker Pattern - docs/tech-spec-epic-4.md:2004-2020]
- [Event Sourcing Pattern - docs/solution-architecture.md:54-73]
- [Notification Scheduling Logic - docs/tech-spec-epic-4.md:1987-2001]

**Technical Specifications:**
- [Epic 4 Tech Spec - docs/tech-spec-epic-4.md] (notification scheduling, Web Push integration)
- [Solution Architecture - docs/solution-architecture.md] (evento, CQRS, background workers, PWA notifications)

**Existing Codebase:**
- [Recipe Aggregate - crates/recipe/src/aggregate.rs] (advance_prep_hours field already exists)
- [Meal Planning Events - crates/meal_planning/src/events.rs] (MealPlanGenerated event to subscribe to)
- [Shopping Crate - crates/shopping/] (similar domain crate pattern to follow)

**Web Push API References:**
- [Web Push Protocol - RFC 8030](https://datatracker.ietf.org/doc/html/rfc8030)
- [VAPID - RFC 8292](https://datatracker.ietf.org/doc/html/rfc8292)
- [web-push Rust crate](https://docs.rs/web-push/)

## Dev Agent Record

### Context Reference

**Context File:** `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.6.xml`

**Generated:** 2025-10-18

**Contents:**
- Complete architecture patterns (Event Sourcing, CQRS, Background Worker, Web Push API)
- Existing codebase analysis (meal_planning events, recipe aggregate, shopping crate patterns)
- Technology stack and dependencies (evento, sqlx, tokio, chrono, web-push)
- Database schema (notifications, push_subscriptions tables)
- Testing standards (unit, integration, E2E with Playwright)
- Implementation tasks breakdown (13 tasks, 81+ subtasks)
- Technical specifications (evento subscriptions, VAPID keys, notification timing logic)
- Reference documentation (PRD, architecture, tech specs, event patterns)
- Interface contracts (MealPlanGenerated event, get_recipe_by_id query)
- Development constraints and patterns

### Agent Model Used

Claude 3.5 Sonnet (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List

**Expected Files to Create:**

**Domain Layer (New Crate):**
- `crates/notifications/Cargo.toml` - Notifications crate manifest
- `crates/notifications/src/lib.rs` - Module exports
- `crates/notifications/src/events.rs` - Notification events
- `crates/notifications/src/commands.rs` - Command handlers
- `crates/notifications/src/aggregate.rs` - NotificationAggregate
- `crates/notifications/src/read_model.rs` - Projections
- `crates/notifications/src/scheduler.rs` - Background worker, scheduling logic
- `crates/notifications/src/push.rs` - Web Push API integration
- `crates/notifications/tests/notification_tests.rs` - Unit tests

**HTTP Layer:**
- `src/routes/notifications.rs` - Notification routes (new file)

**Templates:**
- `templates/pages/notifications.html` - Notification list page
- `templates/pages/profile.html` - Updated with push permission UI
- Updated: `templates/pages/recipe-detail.html` - Add prep highlight when opened via notification

**Migrations:**
- `migrations/XXX_create_notifications_table.sql` - notifications table
- `migrations/XXX_create_push_subscriptions_table.sql` - push_subscriptions table

**Configuration:**
- `config/default.toml` - Add VAPID keys (public_key, private_key)
- Environment variables: `IMKITCHEN__VAPID__PUBLIC_KEY`, `IMKITCHEN__VAPID__PRIVATE_KEY`

**JavaScript:**
- `static/js/push-notifications.js` - Push subscription handling (new file)
- Updated: `static/js/sw.js` - Service worker notification click handler

**Tests:**
- `crates/notifications/tests/notification_tests.rs` - Unit tests
- `tests/notification_integration_tests.rs` - Integration tests
- `e2e/tests/notifications.spec.ts` - E2E tests (Playwright)
