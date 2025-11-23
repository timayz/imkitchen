# Story 3.7: Accompaniment Pairing System

Status: drafted

## Story

As a user,
I want main courses automatically paired with compatible accompaniment recipes,
So that meals are complete with realistic composition (curry with rice, pasta with sauce).

## Acceptance Criteria

1. Main courses with accepts_accompaniment=true trigger pairing logic
2. Algorithm matches main course with compatible accompaniment from favorited accompaniment recipes
3. Accompaniment recipes can repeat freely (not subject to uniqueness constraint)
4. Paired accompaniments stored in meal slot with main course reference
5. 85% or higher success rate for pairing when user has compatible accompaniment recipes favorited
6. Empty accompaniment slot if no compatible accompaniment available
7. Tests verify pairing logic, compatibility matching, and success rate

## Tasks / Subtasks

- [ ] Update meal slot data structure for accompaniments (AC: #4)
  - [ ] Add `accompaniment_id` field to `DayData` struct in event.rs
  - [ ] Make accompaniment_id optional (None if no pairing or no accompaniment)
  - [ ] Update MealPlanGenerated event to include accompaniment data
  - [ ] Ensure accompaniment_id can reference same recipe multiple times

- [ ] Implement accompaniment compatibility logic (AC: #2)
  - [ ] Define compatibility rules: cuisine-based matching
  - [ ] Italian main → Italian accompaniment preferred
  - [ ] Asian main → Asian accompaniment preferred
  - [ ] If no exact cuisine match, select any accompaniment
  - [ ] Create `is_compatible(main: &Recipe, accompaniment: &Recipe) -> bool` function

- [ ] Implement accompaniment pairing algorithm (AC: #1, #2, #3, #6)
  - [ ] After selecting main course, check if main.accepts_accompaniment == true
  - [ ] Filter favorited recipes where recipe_type == "Accompaniment"
  - [ ] Filter accompaniments by compatibility with selected main
  - [ ] Select random accompaniment from compatible pool
  - [ ] If no compatible accompaniments, leave accompaniment_id as None
  - [ ] Update DayData with accompaniment_id

- [ ] Update generation algorithm to include pairing (AC: #1, #3, #4)
  - [ ] Load accompaniment recipes separately from other types
  - [ ] For each main course selected, attempt pairing
  - [ ] Store accompaniment_id in day data structure
  - [ ] Ensure accompaniment uniqueness NOT enforced (can repeat)
  - [ ] Track accompaniment usage for debugging/logging only

- [ ] Measure pairing success rate (AC: #5)
  - [ ] After generation, calculate % of main courses successfully paired
  - [ ] Only count main courses where accepts_accompaniment=true
  - [ ] Target: 85% success rate when user has accompaniments favorited
  - [ ] Log warning if success rate < 85%
  - [ ] Include success rate in generation response (optional)

- [ ] Update meal plan projections for accompaniments (AC: #4)
  - [ ] Add accompaniment_id column to meal_plans or separate table
  - [ ] Option 1: Add column to meal_plan_recipe_snapshots (preferred)
  - [ ] Store accompaniment_id alongside main course in same day row
  - [ ] Query handler extracts accompaniment_id from event and stores

- [ ] Write unit tests for compatibility matching (AC: #2, #7)
  - [ ] Test Italian main + Italian accompaniment (should be compatible)
  - [ ] Test Chinese main + Chinese accompaniment (should be compatible)
  - [ ] Test Italian main + Chinese accompaniment (fallback compatible)
  - [ ] Test main with accepts_accompaniment=false (should not pair)

- [ ] Write unit tests for pairing algorithm (AC: #1, #3, #6, #7)
  - [ ] Test pairing with 5 compatible accompaniments (should succeed)
  - [ ] Test pairing with 0 accompaniments (should leave None)
  - [ ] Test accompaniment repetition across multiple days (should allow)
  - [ ] Test main course uniqueness still enforced with pairing

- [ ] Write integration tests for pairing (AC: #1, #2, #4, #5, #7)
  - [ ] Create test user with 10 main courses (accepts_accompaniment=true)
  - [ ] Create 5 accompaniment recipes with matching cuisines
  - [ ] Execute generate_meal_plan for 4 weeks
  - [ ] Use evento::load to verify accompaniments paired
  - [ ] Calculate success rate and verify >= 85%
  - [ ] Verify accompaniment_id stored in projections

- [ ] Write integration test for low accompaniment count (AC: #5, #6, #7)
  - [ ] Create test user with 10 main courses (accepts_accompaniment=true)
  - [ ] Create only 1 accompaniment recipe
  - [ ] Execute generation
  - [ ] Verify some pairings succeed, some fail (accompaniment_id=None)
  - [ ] Verify generation does not fail due to low accompaniment count

## Dev Notes

### Architecture Patterns

- **Compatibility Matching**: Cuisine-based matching with fallback to any accompaniment
- **Repetition Allowed**: Accompaniments not subject to uniqueness constraint (unlike main courses)
- **Optional Pairing**: Empty accompaniment slot gracefully handled (no error)
- **Success Measurement**: Post-generation metric for quality monitoring

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/event.rs` - Add accompaniment_id to DayData
- `crates/imkitchen-mealplan/src/generator.rs` - Implement pairing logic
- `src/queries/mealplans.rs` - Store accompaniment data in projections
- `migrations/queries/YYYYMMDDHHMMSS_meal_plans.sql` - Add accompaniment support
- `tests/mealplan_test.rs` - Integration tests for pairing

### Technical Constraints

**Accompaniment Compatibility Rules** [Source: epics.md Story 3.7 ACs, PRD.md FR028]:
```rust
fn is_compatible(main: &Recipe, accompaniment: &Recipe) -> bool {
    // Priority 1: Exact cuisine match
    if main.cuisine_type == accompaniment.cuisine_type {
        return true;
    }

    // Priority 2: Related cuisine families
    let asian_cuisines = ["Chinese", "Japanese", "Korean", "Thai", "Vietnamese"];
    let mediterranean_cuisines = ["Italian", "Greek", "Spanish"];

    let main_is_asian = asian_cuisines.contains(&main.cuisine_type.as_str());
    let acc_is_asian = asian_cuisines.contains(&accompaniment.cuisine_type.as_str());

    let main_is_med = mediterranean_cuisines.contains(&main.cuisine_type.as_str());
    let acc_is_med = mediterranean_cuisines.contains(&accompaniment.cuisine_type.as_str());

    if (main_is_asian && acc_is_asian) || (main_is_med && acc_is_med) {
        return true;
    }

    // Priority 3: Fallback - all accompaniments compatible
    true
}
```

**Pairing Algorithm** [Source: epics.md Story 3.7 ACs]:
```rust
fn pair_accompaniment(
    main: &Recipe,
    accompaniments: &[Recipe],
) -> Option<String> {
    if !main.accepts_accompaniment {
        return None; // Main doesn't accept accompaniment
    }

    if accompaniments.is_empty() {
        return None; // No accompaniments available
    }

    // Filter by compatibility
    let compatible: Vec<&Recipe> = accompaniments
        .iter()
        .filter(|acc| is_compatible(main, acc))
        .collect();

    if compatible.is_empty() {
        // Fallback: select any accompaniment
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        return accompaniments.choose(&mut rng).map(|r| r.id.clone());
    }

    // Select random compatible accompaniment
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    compatible.choose(&mut rng).map(|r| r.id.clone())
}
```

**Success Rate Calculation** [Source: epics.md Story 3.7 AC#5]:
```rust
fn calculate_pairing_success_rate(weeks: &[WeekData]) -> f32 {
    let mut total_accepting_mains = 0;
    let mut successful_pairings = 0;

    for week in weeks {
        for day in &week.days {
            if let Some(main_id) = &day.main_id {
                // Check if this main accepts accompaniment
                // (would need to load recipe or store flag in day data)
                if main_accepts_accompaniment(main_id) {
                    total_accepting_mains += 1;
                    if day.accompaniment_id.is_some() {
                        successful_pairings += 1;
                    }
                }
            }
        }
    }

    if total_accepting_mains == 0 {
        return 100.0; // No mains accept accompaniment = 100% success
    }

    (successful_pairings as f32 / total_accepting_mains as f32) * 100.0
}
```

**Database Schema Update** [Source: architecture.md Data Architecture]:
```sql
-- Update meal_plan_recipe_snapshots to include accompaniment
ALTER TABLE meal_plan_recipe_snapshots ADD COLUMN accompaniment_recipe_id TEXT;

-- Or create separate accompaniment column if preferred
-- accompaniment_recipe_id references original recipe, but snapshot stores full data
```

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Test compatibility and pairing logic
  - Pure functions, no database needed
  - Test all cuisine combinations
  - Test edge cases: no accompaniments, all incompatible
- **Integration Tests**: Full generation with pairing
  - Setup user with mains and accompaniments
  - Generate meal plan
  - Verify pairing via evento::load
  - Measure success rate
  - NEVER use direct SQL
- **Success Rate Validation**: Ensure target met
  - Generate with adequate accompaniments (expect >85%)
  - Generate with few accompaniments (expect <85%)
  - Verify algorithm behavior aligns with availability

### References

- [Source: epics.md#Epic 3 Story 3.7]
- [Source: PRD.md FR028 - Accompaniment pairing logic]
- [Source: PRD.md FR006 - Main course accepts_accompaniment field]
- [Source: architecture.md Data Architecture - meal_plan_recipe_snapshots]
- [Source: CLAUDE.md Command Guidelines - Accompaniment repetition allowed]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
