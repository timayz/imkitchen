# Story 4.8: Day-of Cooking Reminders

Status: Approved

## Story

As a **user**,
I want **reminders for today's meals**,
so that **I remember to cook on schedule**.

## Acceptance Criteria

1. Cooking reminder sent 1 hour before typical meal time
2. Default meal times: Breakfast 8am, Lunch 12pm, Dinner 6pm
3. Reminder content: "Tonight's dinner: {recipe_name} - Ready in {total_time}"
4. Reminder displays recipe image and key info
5. Tapping opens recipe detail in cooking mode
6. User can dismiss or snooze (30 min, 1 hour)
7. Reminder respects user profile availability settings
8. No reminder sent if meal already marked as completed (out of MVP scope)

## Tasks / Subtasks

- [ ] Implement day-of cooking reminder scheduling logic (AC: 1, 2, 7)
  - [ ] Create daily scheduled job querying today's meal plan assignments
  - [ ] Calculate reminder time: meal_time - 1 hour (e.g., dinner at 6pm → reminder at 5pm)
  - [ ] Query user profile for availability settings and adjust if needed
  - [ ] Filter meals with scheduled_date = today
  - [ ] Schedule reminder with reminder_type="day_of_cooking"

- [ ] Implement notification message generation for cooking reminders (AC: 3, 4)
  - [ ] Create message template: "{meal_type_label}'s {meal_type}: {recipe_name} - Ready in {total_time}"
  - [ ] Calculate total_time from recipe (prep_time + cook_time)
  - [ ] Load recipe image URL for notification icon
  - [ ] Format notification title and body per AC #3

- [ ] Implement deep linking to recipe detail in cooking mode (AC: 5)
  - [ ] Add click_action URL to push notification payload
  - [ ] Format URL: /recipes/{recipe_id}?mode=cooking
  - [ ] Ensure recipe detail page handles mode=cooking parameter
  - [ ] Activate kitchen mode display automatically when parameter present

- [ ] Implement notification action buttons (AC: 6)
  - [ ] Add "Dismiss" action button to notification payload
  - [ ] Add "Snooze 30 min" action button
  - [ ] Add "Snooze 1 hour" action button
  - [ ] Implement snooze handler: reschedule notification with new time
  - [ ] Implement dismiss handler: mark notification as dismissed in read model

- [ ] Add integration tests (AC: all)
  - [ ] Test: Cooking reminder scheduled 1 hour before meal time
  - [ ] Test: Reminder message format correct for breakfast/lunch/dinner (AC #3)
  - [ ] Test: Recipe image included in notification payload (AC #4)
  - [ ] Test: Deep link URL formatted correctly with mode=cooking (AC #5)
  - [ ] Test: Snooze 30min reschedules notification correctly (AC #6)
  - [ ] Test: Snooze 1hour reschedules notification correctly (AC #6)
  - [ ] Test: Dismiss removes notification from pending queue (AC #6)
  - [ ] Test: No reminder sent if no meal plan for today

- [ ] Update notification UI for cooking reminders (AC: 4, 5)
  - [ ] Display recipe image in notification card
  - [ ] Show total cooking time estimate
  - [ ] Add "Open Recipe" button with cooking mode deep link
  - [ ] Show snooze/dismiss options in notification center

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento**:
- ReminderScheduled event contains reminder_type="day_of_cooking" to distinguish from advance_prep and morning reminders
- Cooking reminders scheduled dynamically: meal_time - 1 hour
- Snooze functionality implemented via ReminderRescheduled event with updated scheduled_time

**Scheduler Design**:
- Background worker runs every 15 minutes, queries meal_plan_slots for today's meals
- Query pattern: `SELECT * FROM meal_plan_slots WHERE meal_date = CURRENT_DATE AND reminder_sent = false`
- Calculate reminder_time based on meal_type and default times (breakfast 8am, lunch 12pm, dinner 6pm)
- Respect user availability settings if configured (future enhancement)

**Notification Message Format** (per AC #3):
```
Title: "Dinner Reminder"
Body: "Tonight's dinner: Chicken Tikka Masala - Ready in 50 minutes"
```

**Deep Linking with Cooking Mode** (per AC #5):
- Web Push notification click_action: `https://imkitchen.app/recipes/{recipe_id}?mode=cooking`
- Frontend: Recipe detail page activates kitchen mode when mode=cooking query param present
- Kitchen mode: high contrast, large text, step-by-step display

**Notification Actions** (per AC #6):
- Web Push supports action buttons (Dismiss, Snooze 30min, Snooze 1hour)
- Service worker handles notificationclick event with action detection
- Snooze: calculate new scheduled_time, emit ReminderRescheduled event
- Dismiss: emit ReminderDismissed event, update read model

### Source Tree Components to Touch

**Existing Files to Modify**:
```
crates/notifications/src/scheduler.rs
   Add day_of_cooking_reminder_scheduler() function
   Query today's meal assignments
   Calculate reminder_time: meal_time - 1 hour
   Schedule reminders with reminder_type="day_of_cooking"

crates/notifications/src/commands.rs
   Reuse existing ScheduleReminderCommand with reminder_type="day_of_cooking"
   Add RescheduleReminderCommand for snooze functionality

crates/notifications/src/events.rs
   Add ReminderRescheduled event (reminder_id, new_scheduled_time)

crates/notifications/src/push.rs
   Update create_push_payload() to include recipe image
   Add action buttons: [{action: "snooze_30", title: "Snooze 30 min"}, {action: "snooze_60", title: "Snooze 1 hour"}, {action: "dismiss", title: "Dismiss"}]
   Format deep link: /recipes/{recipe_id}?mode=cooking
```

**New Files to Create**:
```
tests/day_of_cooking_reminder_tests.rs
   Integration tests for Story 4.8 acceptance criteria
```

**Routes/UI**:
```
templates/pages/notifications.html
   Update to display cooking reminders with recipe images and action buttons

src/routes/recipes.rs
   Ensure /recipes/:id handles ?mode=cooking query parameter
   Activate kitchen mode display when parameter present

static/js/sw.js (service worker)
   Add notificationclick handler for snooze and dismiss actions
```

### Testing Standards Summary

**TDD Approach**:
1. Write failing test for cooking reminder scheduled 1 hour before meal
2. Implement day_of_cooking_reminder_scheduler() in scheduler.rs
3. Write failing test for message format (AC #3, #4)
4. Implement message generation with recipe image
5. Write failing test for cooking mode deep linking (AC #5)
6. Implement click_action URL with mode parameter
7. Write failing test for snooze functionality (AC #6)
8. Implement RescheduleReminderCommand and handler
9. Write failing test for dismiss functionality (AC #6)
10. Implement dismiss handler

**Test Coverage Targets**:
- scheduler.rs cooking reminder logic: 85%
- Snooze/dismiss handlers: 90%
- Integration tests covering all 8 acceptance criteria

**Integration Test Examples**:
```rust
#[tokio::test]
async fn test_cooking_reminder_scheduled_1_hour_before_dinner() {
    // Setup: Meal plan with dinner at 6pm today
    // Action: Run day_of_cooking_reminder_scheduler()
    // Assert: ReminderScheduled event with scheduled_time=5:00pm, reminder_type="day_of_cooking"
}

#[tokio::test]
async fn test_cooking_reminder_message_format() {
    // Setup: Recipe "Chicken Tikka Masala" with prep_time=20, cook_time=30
    // Action: Generate notification message
    // Assert: Body="Tonight's dinner: Chicken Tikka Masala - Ready in 50 minutes"
}

#[tokio::test]
async fn test_snooze_30min_reschedules_notification() {
    // Setup: Cooking reminder scheduled for 5pm
    // Action: User clicks "Snooze 30 min" at 5:00pm
    // Assert: ReminderRescheduled event with new_scheduled_time=5:30pm
}

#[tokio::test]
async fn test_dismiss_removes_notification() {
    // Setup: Cooking reminder in pending queue
    // Action: User clicks "Dismiss"
    // Assert: ReminderDismissed event emitted, notification status updated to dismissed
}
```

### Project Structure Notes

**Alignment with solution-architecture.md**:

This story extends the existing notifications domain crate established in Stories 4.6 and 4.7. All components follow the event-sourced pattern with evento aggregates, commands, events, and read model projections.

**Naming Conventions**:
- Scheduler functions: snake_case (e.g., `day_of_cooking_reminder_scheduler`)
- Event structs: PascalCase past tense (e.g., `ReminderScheduled`, `ReminderRescheduled`, `ReminderDismissed`)
- Command structs: PascalCase imperative (e.g., `ScheduleReminderCommand`, `RescheduleReminderCommand`)
- Background workers: snake_case with `_scheduler` suffix

**Detected Conflicts/Variances**:
- Story 4.6 implements advance prep reminders (24h+ before meal)
- Story 4.7 implements morning reminders (<24h before meal, sent at 9am)
- This story (4.8) implements day-of cooking reminders (1 hour before meal time)
- Need to ensure all three reminder types coexist without conflicts
- Resolution: Use reminder_type field ("advance_prep", "morning", "day_of_cooking") to distinguish

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-4.md] - Notifications domain architecture, scheduler design, push notification integration, day-of cooking reminders
- [Source: docs/solution-architecture.md#Notifications Domain] - Event sourcing patterns, CQRS read models

**Epic Context**:
- [Source: docs/epics.md#Story 4.8] - User story, acceptance criteria, technical notes
- [Source: docs/epics.md#Epic 4: Shopping and Preparation Orchestration] - Epic overview, preparation reminder system goals

**Related Stories**:
- [Source: docs/stories/story-4.6.md] - Advance Preparation Reminder System (prerequisite, establishes notifications crate)
- [Source: docs/stories/story-4.7.md] - Morning Preparation Reminders (related, different reminder type)

**Existing Implementation**:
- [Source: crates/notifications/src/scheduler.rs#calculate_reminder_time] - Existing reminder time calculation logic
- [Source: crates/notifications/src/scheduler.rs#generate_notification_body] - Existing message generation
- [Source: crates/notifications/src/events.rs#ReminderScheduled] - ReminderScheduled event schema with reminder_type field
- [Source: crates/notifications/src/commands.rs#ScheduleReminderCommand] - Command structure with reminder_type support

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.8.xml) - Generated 2025-10-18T23:27:28Z

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
| 2025-10-18 | Jonathan | Status updated to Approved |
