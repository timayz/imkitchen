# Story 3.4: Regeneration with Confirmation

Status: ready 

## Story

As a user,
I want to regenerate all future weeks with a confirmation dialog,
So that I don't accidentally replace my meal plans.

## Acceptance Criteria

1. "Regenerate" button displays confirmation modal: "This will replace all future weeks. Continue?"
2. Confirmation required to proceed with regeneration
3. AllFutureWeeksRegenerated event replaces all non-locked weeks
4. Non-deterministic generation produces different recipe arrangements each time
5. Query handlers delete future week projections and insert new ones
6. Tests verify confirmation requirement and regeneration behavior

## Tasks / Subtasks

- [ ] Create regenerate command (AC: #3, #4)
  - [ ] Add `regenerate_future_weeks()` method to Command struct
  - [ ] Accept input struct with user_id
  - [ ] Load existing meal plans to identify current week
  - [ ] Calculate future weeks to regenerate (start_date > current Sunday)
  - [ ] Use non-deterministic random selection for different arrangements
  - [ ] Emit AllFutureWeeksRegenerated event with new week data
  - [ ] Return regenerated meal_plan_id

- [ ] Define AllFutureWeeksRegenerated event (AC: #3)
  - [ ] Add event to `event.rs` with same structure as MealPlanGenerated
  - [ ] Include current_week_id to preserve reference
  - [ ] Include list of regenerated weeks with new recipe assignments
  - [ ] Use bincode Encode/Decode for serialization

- [ ] Update aggregate to handle regeneration event (AC: #3)
  - [ ] Add `all_future_weeks_regenerated()` handler to aggregate
  - [ ] Replace future weeks in aggregate state
  - [ ] Preserve current week unchanged
  - [ ] Update last_regenerated timestamp

- [ ] Implement non-deterministic generation (AC: #4)
  - [ ] Ensure RNG seed is NOT fixed (use system entropy)
  - [ ] Shuffle recipe order before each generation
  - [ ] Vary random selection weights slightly
  - [ ] Verify repeated regenerations produce different results

- [ ] Create query handler for regeneration (AC: #5)
  - [ ] Add `on_all_future_weeks_regenerated()` handler
  - [ ] DELETE meal plans where week_start_date > current Sunday AND is_current_week = false
  - [ ] INSERT new meal plan rows for regenerated weeks
  - [ ] Update is_current_week flag for preserved week
  - [ ] Use event.timestamp for generated_at

- [ ] Create regenerate route handler (AC: #1, #2)
  - [ ] Add POST route `/mealplan/regenerate`
  - [ ] Show confirmation modal on button click (client-side via ts-action)
  - [ ] Modal displays: "This will replace all future weeks. Continue?"
  - [ ] Cancel button closes modal without action
  - [ ] Confirm button submits POST request to regenerate endpoint

- [ ] Implement confirmation modal UI (AC: #1, #2)
  - [ ] Create modal template component in `templates/components/confirmation-modal.html`
  - [ ] Use Twinspark ts-action to trigger modal display
  - [ ] Include warning message and two buttons (Cancel, Confirm)
  - [ ] Style with Tailwind for visual prominence
  - [ ] Mobile-responsive modal design

- [ ] Update dashboard/calendar with regenerate button (AC: #1)
  - [ ] Add "Regenerate" button next to "Generate" on dashboard
  - [ ] Only show if meal plans already exist
  - [ ] Attach ts-action to trigger confirmation modal
  - [ ] Disable button during regeneration (loading state)

- [ ] Write unit tests for non-deterministic generation (AC: #4)
  - [ ] Run generation 10 times with same input
  - [ ] Verify at least 8/10 produce different recipe arrangements
  - [ ] Ensure main course uniqueness still enforced
  - [ ] Verify dietary restrictions still respected

- [ ] Write integration tests for regeneration (AC: #3, #5, #6)
  - [ ] Generate initial meal plan with 4 weeks
  - [ ] Mock "today" to be in Week 2 (current week)
  - [ ] Execute regenerate command
  - [ ] Verify AllFutureWeeksRegenerated event emitted
  - [ ] Use evento::load to validate Week 2 unchanged, Weeks 3+ regenerated
  - [ ] Verify query projections reflect regenerated state

- [ ] Write E2E test for confirmation flow (AC: #1, #2, #6)
  - [ ] Use Playwright to navigate to dashboard with existing meal plans
  - [ ] Click "Regenerate" button
  - [ ] Verify confirmation modal appears with correct text
  - [ ] Click "Cancel" and verify modal closes, no regeneration
  - [ ] Click "Regenerate" again
  - [ ] Click "Confirm" and verify regeneration occurs
  - [ ] Verify success message displayed

## Dev Notes

### Architecture Patterns

- **Confirmation Pattern**: Use Twinspark ts-action to trigger modal (no custom JavaScript)
- **Non-Deterministic**: Ensure RNG uses system entropy, not fixed seed
- **Event-Driven**: Emit dedicated regeneration event (not reuse MealPlanGenerated)
- **Query Projection**: DELETE + INSERT pattern for idempotent regeneration handling

### Project Structure Notes

Files to modify/create:
- `crates/imkitchen-mealplan/src/command.rs` - Add regenerate_future_weeks() method
- `crates/imkitchen-mealplan/src/event.rs` - Add AllFutureWeeksRegenerated event
- `crates/imkitchen-mealplan/src/aggregate.rs` - Add regeneration handler
- `crates/imkitchen-mealplan/src/generator.rs` - Ensure non-deterministic RNG
- `src/routes/mealplan/regenerate.rs` - New route handler
- `src/queries/mealplans.rs` - Add regeneration query handler
- `templates/components/confirmation-modal.html` - Reusable modal component
- `templates/pages/dashboard.html` - Add regenerate button
- `tests/e2e/user_flows.spec.ts` - E2E confirmation test

### Technical Constraints

**Non-Deterministic Generation** [Source: PRD.md FR022]:
```rust
use rand::{thread_rng, seq::SliceRandom};

fn generate_weeks_non_deterministic(recipes: &mut Vec<Recipe>) {
    let mut rng = thread_rng(); // Uses system entropy
    recipes.shuffle(&mut rng);  // Different order each time

    // Selection with randomness, no fixed seed
    for week in &weeks {
        for day in &mut week.days {
            day.main_id = recipes.choose(&mut rng).map(|r| r.id.clone());
        }
    }
}
```

**Confirmation Modal with Twinspark** [Source: CLAUDE.md Twinspark API]:
```html
<!-- Button with ts-action to show modal -->
<button ts-action="class+ .confirmation-modal-open body">
  Regenerate
</button>

<!-- Modal template -->
<div class="confirmation-modal fixed inset-0 hidden confirmation-modal-open:block">
  <div class="modal-content">
    <p>This will replace all future weeks. Continue?</p>
    <button ts-action="class- .confirmation-modal-open body">Cancel</button>
    <form ts-req="/mealplan/regenerate" ts-req-method="POST">
      <button type="submit">Confirm</button>
    </form>
  </div>
</div>
```

**Query Handler DELETE + INSERT** [Source: CLAUDE.md Query Guidelines]:
```rust
#[evento::handler(MealPlan)]
async fn on_all_future_weeks_regenerated<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<AllFutureWeeksRegenerated, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    // Delete future weeks (idempotent - safe to replay event)
    sqlx::query("DELETE FROM meal_plans WHERE user_id = ? AND is_current_week = 0")
        .bind(&event.metadata.user_id)
        .execute(&pool)
        .await?;

    // Insert regenerated weeks
    for week in &event.data.weeks {
        sqlx::query("INSERT INTO meal_plans (id, user_id, week_start_date, week_number, is_current_week, generated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&week.id)
            .bind(&event.metadata.user_id)
            .bind(&week.week_start_date)
            .bind(week.week_number)
            .bind(week.is_current_week)
            .bind(event.timestamp)
            .execute(&pool)
            .await?;
    }

    Ok(())
}
```

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Verify non-deterministic behavior
  - Run generation 10 times, collect results
  - Ensure variety (not identical arrangements)
  - No need for database in these tests
- **Integration Tests**: Full regeneration flow
  - Setup initial meal plans via generate command
  - Execute regenerate command
  - Verify preservation + replacement via evento::load
  - NEVER use direct SQL
- **E2E Tests**: Confirmation UX validation
  - Critical user flow - must work correctly
  - Test both Cancel and Confirm paths
  - Playwright required for modal interaction

### References

- [Source: epics.md#Epic 3 Story 3.4]
- [Source: PRD.md FR022 - Non-deterministic generation]
- [Source: PRD.md FR023 - Confirmation dialog requirement]
- [Source: CLAUDE.md Twinspark API - ts-action for modal triggers]
- [Source: CLAUDE.md Query Guidelines - Idempotent handlers]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
