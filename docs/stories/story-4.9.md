# Story 4.9: Prep Task Completion Tracking

Status: Approved

## Story

As a **user**,
I want **to mark prep tasks as complete**,
so that **I track my preparation progress**.

## Acceptance Criteria

1. Advance prep reminders include "Mark Complete" button
2. Clicking marks task as completed
3. Completed tasks display checkmark on dashboard
4. Dashboard shows "Prep Tasks for Today" section with completion status
5. Completed tasks removed from active reminders
6. Completion tracked per recipe, per meal slot
7. Uncompleted tasks carried over to next reminder cycle
8. Recipe detail page shows prep completion status

## Tasks / Subtasks

- [ ] Implement prep task completion tracking in notification system (AC: 1, 2, 5, 6)
  - [ ] Add "Mark Complete" action button to advance prep reminder notifications
  - [ ] Create POST /api/notifications/:id/complete endpoint
  - [ ] Implement `CompletePrep TaskCommand` in notifications crate
  - [ ] Emit `PrepTaskCompleted` event when user marks task complete
  - [ ] Update read model to track completion status per notification
  - [ ] Remove completed tasks from active notification queries

- [ ] Add dashboard prep task display (AC: 3, 4)
  - [ ] Create "Prep Tasks for Today" section in dashboard template
  - [ ] Query pending prep tasks for today from notifications read model
  - [ ] Display tasks with recipe name, prep description, timing
  - [ ] Show checkmark icon for completed tasks
  - [ ] Add inline "Mark Complete" button for pending tasks
  - [ ] Update dashboard to refresh when task marked complete (TwinSpark)

- [ ] Add prep status to recipe detail page (AC: 8)
  - [ ] Query notification by recipe_id and meal_plan_slot_id
  - [ ] Display prep task checklist on recipe detail page
  - [ ] Show completion status (pending/completed with timestamp)
  - [ ] Support notification_id query param for direct notification linking
  - [ ] Add "Mark Complete" button that triggers POST /api/notifications/:id/complete

- [ ] Handle uncompleted tasks in reminder cycle (AC: 7)
  - [ ] Modify scheduler to check for pending prep tasks
  - [ ] Re-send reminder notifications for uncompleted tasks
  - [ ] Track reminder_count to prevent infinite reminders
  - [ ] Add max_reminder_count (default: 3) to prevent spam
  - [ ] Update notification status to 'expired' after max reminders

- [ ] Add integration tests (AC: all)
  - [ ] Test: Mark Complete button creates PrepTaskCompleted event
  - [ ] Test: Completed tasks show checkmark on dashboard (AC #3)
  - [ ] Test: Dashboard displays pending prep tasks correctly (AC #4)
  - [ ] Test: Completed tasks removed from active notifications (AC #5)
  - [ ] Test: Completion tracked per meal_plan_slot_id (AC #6)
  - [ ] Test: Uncompleted tasks carried over to next cycle (AC #7)
  - [ ] Test: Recipe detail shows prep completion status (AC #8)

- [ ] Update notification templates and UI (AC: 1, 3, 4)
  - [ ] Add "Mark Complete" action to notification payload
  - [ ] Update templates/pages/dashboard.html with prep tasks section
  - [ ] Update templates/pages/recipe-detail.html with prep checklist
  - [ ] Add checkmark icon styling for completed tasks
  - [ ] Ensure TwinSpark integration for real-time updates

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento**:
- `PrepTaskCompleted` event captures completion with timestamp and user_id
- Event payload includes notification_id, recipe_id, meal_plan_slot_id for traceability
- Read model updated via evento subscription to mark status='completed'
- Aggregate tracks completion_time and completed_by for audit trail

**Read Model Queries**:
- Dashboard query: `SELECT * FROM notifications WHERE user_id = ? AND notification_type IN ('advance_prep', 'morning') AND meal_date = CURRENT_DATE ORDER BY status ASC, scheduled_time ASC`
- Status ordering ensures pending tasks appear first
- Recipe detail query: `SELECT * FROM notifications WHERE recipe_id = ? AND meal_plan_slot_id = ?`

**Notification Actions** (Web Push API):
- Add "Mark Complete" action to notification payload (alongside Snooze/Dismiss)
- Service worker handles notificationclick event with action='complete'
- POSTs to /api/notifications/:id/complete via fetch API
- Optimistic UI update: remove notification immediately, rollback on error

**Reminder Cycle Carry-Over**:
- Uncompleted prep tasks (status='sent') re-scheduled for next reminder window
- Track reminder_count in aggregate to prevent infinite re-sending
- After max_reminder_count (3), mark notification status='expired'
- Expired notifications still show on dashboard but stop sending reminders

### Source Tree Components to Touch

**Existing Files to Modify**:
```
crates/notifications/src/commands.rs
   Add CompletePrepTaskCommand struct
   Implement complete_prep_task() handler

crates/notifications/src/events.rs
   Add PrepTaskCompleted event (notification_id, recipe_id, meal_plan_slot_id, completed_at)

crates/notifications/src/aggregate.rs
   Add prep_task_completed handler to update aggregate state
   Track completion_time and completed_by fields

crates/notifications/src/read_model.rs
   Add get_user_prep_tasks_for_today() query
   Add project_prep_task_completed() projection handler
   Update get_notification_by_id() to include completion status

crates/notifications/src/scheduler.rs
   Modify advance_prep_reminder_scheduler() to check for uncompleted tasks
   Add carry_over_uncompleted_tasks() function
   Track reminder_count and enforce max_reminder_count

src/routes/notifications.rs
   Add POST /api/notifications/:id/complete endpoint
   Validate user owns notification before allowing completion

templates/pages/dashboard.html
   Add "Prep Tasks for Today" section
   Display pending/completed tasks with checkmarks
   Add "Mark Complete" button with TwinSpark integration

templates/pages/recipe-detail.html
   Add prep task checklist section
   Display completion status
   Support ?notification_id= query param for deep linking

static/js/sw.js (service worker)
   Add "Mark Complete" action button to notification payload
   Handle notificationclick event for action='complete'
   Fetch POST /api/notifications/:id/complete
```

**New Files to Create**:
```
tests/prep_task_completion_tests.rs
   Integration tests for Story 4.9 acceptance criteria
```

**Database Schema Changes**:
```sql
-- Add to existing notifications table (if not present)
ALTER TABLE notifications ADD COLUMN completion_time TEXT;
ALTER TABLE notifications ADD COLUMN reminder_count INTEGER DEFAULT 0;
ALTER TABLE notifications ADD COLUMN max_reminder_count INTEGER DEFAULT 3;

CREATE INDEX idx_notifications_user_prep_tasks
ON notifications(user_id, notification_type, meal_date, status);
```

### Testing Standards Summary

**TDD Approach**:
1. Write failing test for POST /api/notifications/:id/complete endpoint
2. Implement CompletePrepTaskCommand and handler
3. Write failing test for PrepTaskCompleted event emission
4. Implement event handler and projection
5. Write failing test for dashboard prep tasks display (AC #3, #4)
6. Implement dashboard query and template
7. Write failing test for uncompleted task carry-over (AC #7)
8. Implement scheduler logic for re-sending reminders
9. Write failing test for recipe detail prep status (AC #8)
10. Implement recipe detail query and UI

**Test Coverage Targets**:
- commands.rs complete_prep_task(): 90%
- Dashboard prep tasks query: 85%
- Scheduler carry-over logic: 85%
- Integration tests covering all 8 acceptance criteria

**Integration Test Examples**:
```rust
#[tokio::test]
async fn test_mark_complete_creates_prep_task_completed_event() {
    // Setup: Create advance prep reminder for user
    // Action: POST /api/notifications/:id/complete
    // Assert: PrepTaskCompleted event emitted with correct notification_id
}

#[tokio::test]
async fn test_completed_tasks_show_checkmark_on_dashboard() {
    // Setup: User has completed prep task
    // Action: GET /dashboard
    // Assert: Dashboard displays task with checkmark icon, status='completed'
}

#[tokio::test]
async fn test_uncompleted_tasks_carried_over() {
    // Setup: Prep task sent yesterday, not completed
    // Action: Run scheduler today
    // Assert: New reminder scheduled, reminder_count incremented
}

#[tokio::test]
async fn test_max_reminder_count_prevents_infinite_reminders() {
    // Setup: Prep task with reminder_count=3
    // Action: Run scheduler
    // Assert: No new reminder sent, status updated to 'expired'
}
```

### Project Structure Notes

**Alignment with solution-architecture.md**:

This story extends the notifications domain crate established in Stories 4.6, 4.7, and 4.8. All components follow the event-sourced pattern with evento aggregates, commands, events, and read model projections.

**Module Organization**:
- Commands: `crates/notifications/src/commands.rs` - Command handlers
- Events: `crates/notifications/src/events.rs` - Domain events
- Aggregate: `crates/notifications/src/aggregate.rs` - NotificationAggregate with event handlers
- Read Model: `crates/notifications/src/read_model.rs` - Query functions and projections
- Routes: `src/routes/notifications.rs` - HTTP endpoints

**Naming Conventions**:
- Command: `CompletePrepTaskCommand` (PascalCase imperative)
- Event: `PrepTaskCompleted` (PascalCase past tense)
- Function: `complete_prep_task()` (snake_case)
- Query: `get_user_prep_tasks_for_today()` (snake_case)

**Detected Conflicts/Variances**:
- Story 4.8 implements dismiss functionality via ReminderDismissed event
- This story (4.9) adds completion tracking with PrepTaskCompleted event
- Need to distinguish: Dismiss = user ignores notification, Complete = user finished prep task
- Resolution: Two separate events and status values ('dismissed' vs 'completed')

**Lessons Learned from Story 4.8**:
- Use evento-generated ULID as notification_id for consistency with aggregate lookup
- Implement TwinSpark integration for real-time UI updates without full page reload
- Add comprehensive integration tests covering all acceptance criteria
- Track status transitions carefully (pending → sent → completed/dismissed/expired)

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-4.md#Story 10: Prep Task Completion Tracking] - Technical verification requirements, endpoint specs, event schema
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Notifications crate organization
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships] - Notifications table schema

**Epic Context**:
- [Source: docs/epics.md#Story 4.9] - User story, acceptance criteria, technical notes
- [Source: docs/epics.md#Epic 4: Shopping and Preparation Orchestration] - Epic overview, prep task tracking goals

**Related Stories**:
- [Source: docs/stories/story-4.6.md] - Advance Preparation Reminder System (prerequisite, establishes notification scheduling)
- [Source: docs/stories/story-4.7.md] - Morning Preparation Reminders (related, morning reminder type)
- [Source: docs/stories/story-4.8.md] - Day-of Cooking Reminders (predecessor, establishes dismiss functionality)

**Existing Implementation**:
- [Source: src/routes/notifications.rs:78-100] - Existing dismiss_notification endpoint pattern to follow
- [Source: crates/notifications/src/commands.rs#DismissReminderCommand] - Similar command structure for reference
- [Source: crates/notifications/src/events.rs#ReminderDismissed] - Event pattern to replicate for PrepTaskCompleted
- [Source: crates/notifications/src/read_model.rs#get_user_pending_notifications] - Query pattern for dashboard prep tasks

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.9.xml) - Generated 2025-10-18

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
| 2025-10-18 | Bob (SM) | Story context generated - story-context-4.9.xml created |
| 2025-10-18 | Jonathan | Status updated to Approved |
