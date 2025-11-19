# Story 3.5: Dietary Restriction Filtering

Status: drafted

## Story

As a user,
I want the algorithm to respect my dietary restrictions,
So that generated meal plans only include recipes I can actually eat.

## Acceptance Criteria

1. Algorithm filters favorited recipes by user's dietary restrictions from profile
2. Recipe excluded if ANY of its dietary_restrictions conflicts with user restrictions
3. 95% or higher of generated meals meet user's dietary restrictions
4. Empty slots left if insufficient restriction-compliant recipes available
5. Tests verify restriction filtering with various dietary combinations

## Tasks / Subtasks

- [ ] Load user dietary restrictions in generation command (AC: #1)
  - [ ] Query user_profiles table to fetch dietary_restrictions JSON array
  - [ ] Parse dietary restrictions into Rust Vec<String>
  - [ ] Pass restrictions to generation algorithm function
  - [ ] Handle case where user has no restrictions (empty array)

- [ ] Implement dietary restriction filtering logic (AC: #1, #2)
  - [ ] Update `generator.rs` to accept user dietary restrictions parameter
  - [ ] Create filter function `matches_dietary_restrictions(recipe: &Recipe, user_restrictions: &[String]) -> bool`
  - [ ] For each recipe dietary_restriction, check if it conflicts with user restrictions
  - [ ] Return false if ANY conflict found, true otherwise
  - [ ] Apply filter before recipe selection in generation algorithm

- [ ] Define restriction conflict logic (AC: #2)
  - [ ] Create mapping of restriction conflicts (e.g., "gluten-free" excludes recipes with "contains-gluten")
  - [ ] Common restrictions: gluten-free, dairy-free, vegan, vegetarian, nut-free
  - [ ] Recipe restrictions: contains-gluten, contains-dairy, contains-meat, contains-nuts
  - [ ] Conflict if user has "X-free" and recipe has "contains-X"

- [ ] Update recipe selection to use filtered pool (AC: #1, #2, #4)
  - [ ] Filter favorited recipes before selection begins
  - [ ] Separate filtered recipes by type (appetizer, main, dessert)
  - [ ] If filtered pool empty for any type, leave slots empty
  - [ ] Log warning when insufficient compliant recipes available

- [ ] Implement restriction compliance measurement (AC: #3)
  - [ ] After generation, calculate % of non-empty slots that meet restrictions
  - [ ] Target: 95% or higher compliance
  - [ ] If compliance < 95%, log warning but don't fail generation
  - [ ] Include compliance metric in generation response (optional)

- [ ] Update generation command to include dietary filtering (AC: #1, #2)
  - [ ] Load user dietary restrictions from user_profiles table
  - [ ] Pass restrictions to generator algorithm
  - [ ] Generate meal plan with filtered recipe pool
  - [ ] Emit event with generated weeks
  - [ ] Return meal_plan_id with optional compliance metric

- [ ] Write unit tests for filtering logic (AC: #2, #5)
  - [ ] Test with user restriction "gluten-free", recipe has "contains-gluten" (should exclude)
  - [ ] Test with user restriction "vegan", recipe has "contains-meat" (should exclude)
  - [ ] Test with user restriction "nut-free", recipe has "nut-free" (should include)
  - [ ] Test with multiple user restrictions (e.g., gluten-free + dairy-free)
  - [ ] Test with no user restrictions (should include all recipes)

- [ ] Write integration tests for dietary filtering (AC: #1, #3, #4, #5)
  - [ ] Create test user with dietary restrictions: ["gluten-free", "dairy-free"]
  - [ ] Create favorited recipes: some compliant, some non-compliant
  - [ ] Execute generate_meal_plan command
  - [ ] Use evento::load to verify generated meal plan
  - [ ] Verify only compliant recipes selected
  - [ ] Verify empty slots when insufficient compliant recipes
  - [ ] Calculate compliance % and verify >= 95%

- [ ] Write E2E test for dietary filtering end-to-end (AC: #1, #3, #5)
  - [ ] Use Playwright to register user with dietary restrictions
  - [ ] Favorite recipes with various restrictions
  - [ ] Generate meal plan
  - [ ] View calendar and verify meals displayed
  - [ ] Manually verify generated meals respect restrictions

## Dev Notes

### Architecture Patterns

- **Filtering at Algorithm Entry**: Apply dietary filter before random selection, not after
- **Graceful Degradation**: Empty slots when insufficient compliant recipes (don't fail generation)
- **Data Source**: User restrictions from user_profiles, recipe restrictions from recipes table
- **Compliance Measurement**: Post-generation metric, non-blocking

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/command.rs` - Load user dietary restrictions
- `crates/imkitchen-mealplan/src/generator.rs` - Implement filtering logic
- `src/queries/users.rs` - Query function for user dietary restrictions
- `tests/mealplan_test.rs` - Integration tests for dietary filtering

### Technical Constraints

**Dietary Restrictions Mapping** [Source: PRD.md FR026, epics.md Story 3.5]:

User Restriction → Recipe Exclusions:
- "gluten-free" → Exclude recipes with "contains-gluten"
- "dairy-free" → Exclude recipes with "contains-dairy"
- "vegan" → Exclude recipes with "contains-meat" OR "contains-dairy" OR "contains-eggs"
- "vegetarian" → Exclude recipes with "contains-meat" OR "contains-fish"
- "nut-free" → Exclude recipes with "contains-nuts"

**Filtering Algorithm** [Source: epics.md Story 3.5 AC#2]:
```rust
fn matches_dietary_restrictions(recipe: &Recipe, user_restrictions: &[String]) -> bool {
    for user_restriction in user_restrictions {
        match user_restriction.as_str() {
            "gluten-free" => {
                if recipe.dietary_restrictions.contains(&"contains-gluten".to_string()) {
                    return false;
                }
            }
            "dairy-free" => {
                if recipe.dietary_restrictions.contains(&"contains-dairy".to_string()) {
                    return false;
                }
            }
            "vegan" => {
                if recipe.dietary_restrictions.iter().any(|r|
                    r == "contains-meat" || r == "contains-dairy" || r == "contains-eggs"
                ) {
                    return false;
                }
            }
            "vegetarian" => {
                if recipe.dietary_restrictions.iter().any(|r|
                    r == "contains-meat" || r == "contains-fish"
                ) {
                    return false;
                }
            }
            "nut-free" => {
                if recipe.dietary_restrictions.contains(&"contains-nuts".to_string()) {
                    return false;
                }
            }
            _ => {} // Unknown restriction, ignore
        }
    }
    true // No conflicts found
}
```

**Compliance Calculation** [Source: epics.md Story 3.5 AC#3]:
```rust
fn calculate_compliance(weeks: &[WeekData], compliant_recipe_ids: &[String]) -> f32 {
    let mut total_slots = 0;
    let mut compliant_slots = 0;

    for week in weeks {
        for day in &week.days {
            if let Some(id) = &day.appetizer_id {
                total_slots += 1;
                if compliant_recipe_ids.contains(id) {
                    compliant_slots += 1;
                }
            }
            // Repeat for main_id and dessert_id
        }
    }

    if total_slots == 0 {
        return 100.0; // No slots = 100% compliance
    }

    (compliant_slots as f32 / total_slots as f32) * 100.0
}
```

**Database Query** [Source: architecture.md Data Architecture]:
```rust
// Query user dietary restrictions
pub async fn get_user_dietary_restrictions(
    pool: &SqlitePool,
    user_id: &str,
) -> anyhow::Result<Vec<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT dietary_restrictions FROM user_profiles WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((json_str,)) => {
            let restrictions: Vec<String> = serde_json::from_str(&json_str)?;
            Ok(restrictions)
        }
        None => Ok(vec![]), // No profile = no restrictions
    }
}
```

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Pure function tests for filtering logic
  - Test all restriction types individually
  - Test combinations of restrictions
  - Test edge cases (no restrictions, unknown restrictions)
  - No database needed
- **Integration Tests**: Full generation with dietary filtering
  - Setup user with restrictions
  - Create compliant and non-compliant recipes
  - Generate meal plan
  - Verify compliance via evento::load
  - NEVER use direct SQL
- **E2E Tests**: User journey validation
  - Register → Set restrictions → Favorite recipes → Generate → Verify calendar

### References

- [Source: epics.md#Epic 3 Story 3.5]
- [Source: PRD.md FR026 - Dietary restrictions consideration]
- [Source: architecture.md Data Architecture - user_profiles.dietary_restrictions]
- [Source: CLAUDE.md Command Guidelines - Load data for commands]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
