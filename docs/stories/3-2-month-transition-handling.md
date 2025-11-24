# Story 3.2: Month Transition Handling

Status: ready

## Story

As a user,
I want generation at month-end to extend into next month,
So that I always have continuous meal plans without manual month switching.

## Acceptance Criteria

1. Generation logic detects when current month has <4 future weeks remaining
2. Extends generation into next month to ensure minimum 4 weeks generated
3. Explicit transition message displayed: "Generating weeks for rest of October + first 2 weeks of November"
4. Week metadata includes month and year for accurate calendar display
5. Tests verify month transition calculation and messaging

## Tasks / Subtasks

- [ ] Enhance week calculation logic (AC: #1, #2)
  - [ ] Modify `calculate_weeks_to_generate()` function to count remaining weeks in current month
  - [ ] If remaining weeks < 4, extend calculation into next month
  - [ ] Continue adding Monday dates until minimum 4 weeks reached
  - [ ] Return weeks with month/year metadata included

- [ ] Add month/year metadata to week structures (AC: #4)
  - [ ] Update `WeekData` struct in `event.rs` to include month and year fields
  - [ ] Populate month/year from week_start_date during generation
  - [ ] Update `meal_plans` table migration to include month and year columns
  - [ ] Update query handler to store month/year in projections

- [ ] Generate transition message (AC: #3)
  - [ ] Create helper function to format transition message
  - [ ] Detect when weeks span multiple months
  - [ ] Format message: "Generating weeks for rest of {month1} + first {n} weeks of {month2}"
  - [ ] Return message with generation result

- [ ] Update generation command to use enhanced logic (AC: #1, #2, #3)
  - [ ] Call enhanced week calculation function
  - [ ] Include transition message in command response
  - [ ] Store transition message in event metadata if applicable
  - [ ] Pass message to route handler for display

- [ ] Update route handler to display transition message (AC: #3)
  - [ ] Modify generation route to extract transition message
  - [ ] Pass message to template context
  - [ ] Display message prominently in UI after generation
  - [ ] Update Askama template with transition message section

- [ ] Write unit tests for month transition (AC: #1, #2, #5)
  - [ ] Test week calculation when current month has 5+ weeks remaining
  - [ ] Test week calculation when current month has 3 weeks remaining (should extend)
  - [ ] Test week calculation when current month has 1 week remaining (should extend)
  - [ ] Test week calculation on last day of month
  - [ ] Verify minimum 4 weeks always generated

- [ ] Write integration tests for transition messaging (AC: #3, #5)
  - [ ] Execute generation at month-end
  - [ ] Verify transition message returned in response
  - [ ] Verify generated weeks span two months
  - [ ] Verify month/year metadata correctly stored in projections

## Dev Notes

### Architecture Patterns

- **Week Calculation Enhancement**: Extend existing algorithm from Story 3.1 to handle month boundaries
- **Event Structure**: Add optional transition_message field to EventMetadata or command response
- **Query Enhancement**: Include month/year in projections for calendar display filtering

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/generator.rs` - Week calculation logic
- `crates/imkitchen-mealplan/src/event.rs` - Add month/year to WeekData
- `crates/imkitchen-mealplan/src/command.rs` - Return transition message
- `src/routes/mealplan/generate.rs` - Display transition message
- `migrations/queries/YYYYMMDDHHMMSS_meal_plans.sql` - Add month/year columns

### Technical Constraints

**Month Boundary Logic** [Source: epics.md Story 3.2 ACs]:
```rust
fn calculate_weeks_to_generate(today: NaiveDate) -> (Vec<NaiveDate>, Option<String>) {
    let next_monday = /* calculate next Monday */;
    let mut weeks = vec![];
    let mut current = next_monday;

    // Count weeks in current month
    while current.month() == next_monday.month() {
        weeks.push(current);
        current += Duration::days(7);
    }

    // If less than 4 weeks, extend into next month
    let mut transition_msg = None;
    if weeks.len() < 4 {
        let needed = 4 - weeks.len();
        transition_msg = Some(format!(
            "Generating weeks for rest of {} + first {} weeks of {}",
            month_name(next_monday.month()),
            needed,
            month_name(current.month())
        ));

        while weeks.len() < 4 {
            weeks.push(current);
            current += Duration::days(7);
        }
    }

    (weeks, transition_msg)
}
```

**Database Schema Update** [Source: architecture.md Data Architecture]:
```sql
ALTER TABLE meal_plans ADD COLUMN month INTEGER NOT NULL DEFAULT 0;
ALTER TABLE meal_plans ADD COLUMN year INTEGER NOT NULL DEFAULT 0;
CREATE INDEX idx_meal_plans_user_month_year ON meal_plans(user_id, year, month);
```

**UI Message Display** [Source: PRD.md FR019]:
- Display transition message prominently after generation
- Message format: "Generating weeks for rest of October + first 2 weeks of November"
- Include visual indicator (icon or badge) for multi-month plans
- Show in generation success template

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Focus on week calculation edge cases
  - Test on 1st of month (should generate ~4-5 weeks all in same month)
  - Test on 25th of month (likely needs extension into next month)
  - Test on last day of month (definitely needs extension)
  - Test February (short month) vs December (year boundary)
- **Integration Tests**: Verify end-to-end behavior
  - Mock "today" date to simulate month-end
  - Execute generate_meal_plan command
  - Verify weeks span two months in projections
  - Verify transition message returned
- **No direct SQL**: Use evento::load to validate state

### References

- [Source: epics.md#Epic 3 Story 3.2]
- [Source: PRD.md FR019 - Month transition with explicit messaging]
- [Source: architecture.md Data Architecture - meal_plans table]
- [Source: CLAUDE.md Command Guidelines - Command return values]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
