# Story 4.7: Shopping List Access (Current + Future Weeks)

Status: drafted

## Story

As a user,
I want to generate shopping lists for current week and all future weeks,
So that I can plan my grocery shopping flexibly.

## Acceptance Criteria

1. Shopping list route accepts week_id parameter
2. Users can generate shopping lists for any non-past week
3. Free tier users can generate shopping lists for accessible first week only
4. Premium tier users can generate shopping lists for any generated week
5. Shopping list UI shows week selector dropdown
6. Tests verify access control for free vs premium tiers

## Tasks / Subtasks

- [ ] Integrate Access Control in shopping list route (AC: #3, #4)
  - [ ] Modify `src/routes/shopping.rs` to use AccessControlService
  - [ ] Check access before generating shopping list: `access_control.can_view_week(user_id, week_number)`
  - [ ] If access denied → return access denied error or redirect to upgrade page
  - [ ] If access allowed → proceed with shopping list generation
  - [ ] Premium users: all weeks accessible
  - [ ] Free users: week 1 only accessible
- [ ] Add week validation logic (AC: #2)
  - [ ] Validate week_number parameter: must be > 0
  - [ ] Query meal_plans to verify week exists for user
  - [ ] Prevent access to past weeks (week start date < today - 7 days)
  - [ ] Return 404 if week doesn't exist
  - [ ] Return error if week is in the past
- [ ] Create week selector component (AC: #5)
  - [ ] Create `templates/components/week-selector.html`
  - [ ] Display dropdown with available weeks: "Week 1", "Week 2", etc.
  - [ ] Show current week indicator (highlighted or marked)
  - [ ] Lock weeks inaccessible to free tier (disabled options)
  - [ ] Use Twinspark `ts-req` to load shopping list on week change
  - [ ] Style with Tailwind for mobile-responsive design
- [ ] Modify shopping list template for week selection (AC: #5)
  - [ ] Add week selector component to `templates/pages/shopping.html`
  - [ ] Position selector at top of page for easy access
  - [ ] Update shopping list content when week changes
  - [ ] Display currently selected week prominently
  - [ ] Show accessibility status: free tier sees lock icons on weeks 2+
- [ ] Add upgrade prompt for restricted weeks (AC: #3)
  - [ ] When free tier user attempts to access week 2+, show upgrade prompt
  - [ ] Display message: "Upgrade to unlock shopping lists for all weeks"
  - [ ] Link to pricing page for conversion
  - [ ] Use upgrade modal component from Story 4.4
  - [ ] Show first week shopping list as teaser
- [ ] Write integration tests (AC: #6)
  - [ ] Extend `tests/shopping_list_test.rs` with access control tests
  - [ ] Test free tier: week 1 accessible, week 2+ restricted
  - [ ] Test premium tier: all weeks accessible
  - [ ] Test past week access: denied for all users
  - [ ] Test non-existent week: returns 404
  - [ ] Test global bypass: free user accesses all weeks with bypass=true
  - [ ] Test per-user bypass: free user with premium_bypass accesses all weeks

## Dev Notes

### Architecture Patterns and Constraints

**Access Control Integration:**
- Reuse AccessControlService from Stories 4.4 and 4.5
- Consistent freemium enforcement: calendar, dashboard, shopping lists
- Same week-level access check logic across all features
- Access denied → redirect to upgrade page or show modal

**Week Validation Logic:**
```rust
// src/routes/shopping.rs
pub async fn get_shopping_list(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(week_number): Path<usize>,
) -> impl IntoResponse {
    // 1. Validate week exists
    let week = queries::mealplans::get_week(&state.pool, &user.id, week_number).await?;
    if week.is_none() {
        return Err(anyhow!("Week not found")); // 404
    }

    // 2. Check if week is past
    if week.is_past() {
        return Err(anyhow!("Cannot generate shopping list for past weeks"));
    }

    // 3. Check access control
    let can_view = state.access_control.can_view_week(&user.id, week_number).await?;
    if !can_view {
        // Redirect to upgrade page or show restricted template
        return UpgradePromptTemplate { ... }.into_response();
    }

    // 4. Generate shopping list
    let list = state.shopping_list_service.generate_for_week(&user.id, week_number).await?;
    ShoppingListTemplate { list, week_number, ... }.into_response()
}
```

**Week Selector UI:**
- Dropdown shows all generated weeks (1-5)
- Free tier: week 1 enabled, weeks 2+ disabled with lock icon
- Premium tier: all weeks enabled
- Current week highlighted for clarity
- Twinspark updates shopping list without page reload

**Past Week Restriction:**
- Prevent shopping list generation for past weeks (more than 7 days ago)
- Rationale: shopping lists are forward-looking, not historical
- Users can still view past meal plans in calendar (read-only)
- Past week definition: week_start_date < today - 7 days

**Upgrade Conversion Flow:**
1. Free tier user clicks week 2 in selector
2. Access check fails → show upgrade prompt
3. Prompt displays: "Unlock shopping lists for all weeks"
4. User clicks "Upgrade Now" → pricing page
5. After subscription, user can access all weeks

**Week Selector State Management:**
- Active week stored in URL parameter: `/shopping/{week_number}`
- Dropdown updates URL on selection change
- Browser back/forward buttons work naturally
- Deep linking supported: share `/shopping/3` link

### Project Structure Notes

**Files to Create/Modify:**
- `src/routes/shopping.rs` - Integrate AccessControlService (MODIFY)
- `src/queries/mealplans.rs` - Add `get_week()` validation function (MODIFY)
- `templates/pages/shopping.html` - Add week selector component (MODIFY)
- `templates/components/week-selector.html` - Week selector dropdown (NEW)
- `tests/shopping_list_test.rs` - Add access control tests (MODIFY)

**Query Function Addition:**
```rust
// src/queries/mealplans.rs
pub struct Week {
    pub week_number: usize,
    pub week_start_date: String,  // ISO date
    pub is_current: bool,
}

impl Week {
    pub fn is_past(&self) -> bool {
        // Check if week_start_date < today - 7 days
        let week_start = NaiveDate::parse_from_str(&self.week_start_date, "%Y-%m-%d")?;
        let cutoff = Utc::now().naive_utc().date() - Duration::days(7);
        week_start < cutoff
    }
}

pub async fn get_week(pool: &SqlitePool, user_id: &str, week_number: usize) -> anyhow::Result<Option<Week>> {
    // Query meal_plans for specified week
}
```

**Template Structure:**
```html
<!-- templates/pages/shopping.html -->
<div class="shopping-list-container">
    {% include "components/week-selector.html" %}

    <h1>Shopping List for Week {{ week_number }}</h1>

    <!-- Shopping list content from Story 4.6 -->
    ...
</div>
```

**Week Selector Component:**
```html
<!-- templates/components/week-selector.html -->
<select ts-req="/shopping/{value}" ts-req-method="GET" ts-target="#shopping-list-content">
    {% for week in available_weeks %}
        <option value="{{ week.number }}" {% if week.is_locked %}disabled{% endif %}>
            Week {{ week.number }} {% if week.is_locked %}🔒{% endif %}
        </option>
    {% endfor %}
</select>
```

**Visual Mockup Alignment:**
- Implements `mockups/shopping-list.html` week selector functionality
- Locked weeks for free tier match calendar locked week styling
- Week selector positioned consistently with calendar navigation
- Upgrade prompt matches dashboard and calendar upgrade prompts

**Integration with Stories:**
- Story 4.4: Reuses AccessControlService for consistent access checks
- Story 4.6: Extends shopping list generation with access control
- Story 6.3: Links to pricing page for upgrade conversion

### References

- [Source: docs/epics.md#Story 4.7 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR037 - Shopping lists accessible for current and future weeks]
- [Source: docs/architecture.md#ADR-004 - Centralized Access Control Service]
- [Source: docs/architecture.md#HTTP Routes - /shopping/{week_number} route contract]
- [Source: CLAUDE.md#Server-Side Rendering Rules - Twinspark for dynamic updates]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

<!-- Debug logs will be added during implementation -->

### Completion Notes List

<!-- Completion notes will be added during implementation -->

### File List

<!-- Files created/modified will be listed during implementation -->
