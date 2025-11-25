# Story 4.8: Empty Slot Community Recipe Links

Status: drafted

## Story

As a user,
I want empty meal slots to suggest browsing community recipes,
So that I can discover new recipes to fill gaps in my meal plan.

## Acceptance Criteria

1. Empty meal slots display "Browse community recipes" link
2. Link routes to community recipe page with appropriate filter (appetizer/main/dessert)
3. Community page shows relevant recipes based on slot type
4. Users can favorite recipes directly from community page
5. Tests verify link routing and filter application

## Tasks / Subtasks

- [ ] Update meal card component for empty slots (AC: #1)
  - [ ] Modify `templates/components/meal-card.html` to detect empty slots
  - [ ] Empty slot condition: snapshot_id is None/null
  - [ ] Display placeholder UI: "Empty slot" message
  - [ ] Show "Browse community recipes" link with appropriate styling
  - [ ] Link includes slot type parameter: `/recipes/community?type={slot_type}`
  - [ ] Style with Tailwind: dashed border, muted colors, clear CTA
- [ ] Update calendar template with empty slot handling (AC: #1, #2)
  - [ ] Ensure calendar template passes slot_type to meal card component
  - [ ] Slot types: appetizer, main, dessert (matches RecipeType enum)
  - [ ] Empty slot links route correctly based on slot type
  - [ ] Maintain consistent empty slot UI across all calendar views
- [ ] Create community recipe route with filters (AC: #2, #3)
  - [ ] Implement GET `/recipes/community` route in `src/routes/recipes/community.rs`
  - [ ] Extract query parameter: `?type={appetizer|main|dessert|accompaniment}`
  - [ ] Query shared recipes filtered by recipe_type if parameter present
  - [ ] Render community recipe page with filtered results
  - [ ] Preserve filter state in URL for deep linking
- [ ] Implement community recipe query function (AC: #3)
  - [ ] Create `queries::recipes::get_community_recipes()` in `src/queries/recipes.rs`
  - [ ] Query recipes table: WHERE is_shared = true
  - [ ] Apply recipe_type filter if provided
  - [ ] Sort by rating (highest first), then favorite count
  - [ ] Paginate results (20 recipes per page)
  - [ ] Return recipe cards with: name, type, rating, favorite count, owner name
- [ ] Create community recipe page template (AC: #3, #4)
  - [ ] Create `templates/pages/recipes/community.html`
  - [ ] Display filtered recipe type header if filter applied (e.g., "Community Appetizers")
  - [ ] Render recipe cards using reusable recipe-card component
  - [ ] Include quick-favorite button on each card
  - [ ] Show filter badges: active filter highlighted, others clickable
  - [ ] Add search bar and additional filters (cuisine, dietary restrictions)
  - [ ] Style with Tailwind for mobile-responsive grid layout
- [ ] Add favorite button integration (AC: #4)
  - [ ] Include favorite toggle button on each recipe card
  - [ ] Use Twinspark `ts-req` POST to `/recipes/{id}/favorite`
  - [ ] Update favorite button state without page reload
  - [ ] Display favorite count next to button
  - [ ] Show upgrade modal if free tier user hits 10 favorite limit (Story 5.4 integration)
- [ ] Write integration tests (AC: #5)
  - [ ] Extend `tests/calendar_test.rs` to verify empty slot links
  - [ ] Test empty slot displays "Browse community recipes" link
  - [ ] Test link includes correct type parameter (appetizer/main/dessert)
  - [ ] Create `tests/community_test.rs` for community page tests
  - [ ] Test community page with type filter: appetizer, main, dessert, accompaniment
  - [ ] Test community page without filter: shows all shared recipes
  - [ ] Test favorite button integration on community page

## Dev Notes

### Architecture Patterns and Constraints

**Empty Slot Detection:**
- Empty slot = meal_plan_recipe_snapshots row with NULL/missing snapshot_id
- Empty slots created by meal plan generation algorithm (Story 3.9) when insufficient favorited recipes
- No minimum recipe count enforced - graceful degradation per FR021

**Community Recipe Filtering:**
- URL parameter approach: `/recipes/community?type=appetizer`
- Type parameter matches RecipeType enum: Appetizer, MainCourse, Dessert, Accompaniment
- Filter applied in SQL query: WHERE is_shared = true AND recipe_type = ?
- Preserves filter state for back/forward navigation

**Recipe Discovery Flow:**
1. User generates meal plan with insufficient recipes → empty slots appear
2. User clicks "Browse community recipes" link on empty appetizer slot
3. Community page loads with appetizer filter applied
4. User browses community appetizers, favorites 3 recipes
5. User regenerates meal plan → new appetizers fill previously empty slots

**Favorite Button UX:**
- Quick-favorite button on each recipe card (no need to open recipe detail)
- Twinspark async POST without page reload
- Toggle state: favorited (filled heart) vs not favorited (outline heart)
- Free tier limit check (Story 5.4): show upgrade modal if limit reached

**Query Optimization:**
- Community recipes query: WHERE is_shared = true (index required)
- Recipe type filter: indexed on recipe_type column
- Sort by rating DESC, favorite_count DESC for quality filtering
- Pagination: LIMIT 20 OFFSET {page * 20}

**Empty Slot UI Consistency:**
- Same empty slot styling in calendar, dashboard, week carousel
- Dashed border, muted background color, clear CTA
- Icon or illustration for visual distinction from filled slots
- Encourages discovery without feeling like an error state

### Project Structure Notes

**Files to Create/Modify:**
- `templates/components/meal-card.html` - Add empty slot UI (MODIFY)
- `templates/pages/mealplan/calendar.html` - Pass slot_type to meal cards (MODIFY)
- `src/routes/recipes/community.rs` - Community recipe route with filters (CREATE if not exists, MODIFY if exists from Epic 5)
- `src/queries/recipes.rs` - Add community recipe query function (MODIFY)
- `templates/pages/recipes/community.html` - Community recipe page template (CREATE if not exists, MODIFY if exists)
- `tests/calendar_test.rs` - Add empty slot link tests (MODIFY)
- `tests/community_test.rs` - Community page tests (NEW)

**Meal Card Component Update:**
```html
<!-- templates/components/meal-card.html -->
{% if meal_slot.is_empty %}
    <div class="meal-card meal-card-empty">
        <div class="empty-icon">🍽️</div>
        <p class="text-muted">Empty {{ slot_type }}</p>
        <a href="/recipes/community?type={{ slot_type }}" class="cta-link">
            Browse community recipes
        </a>
    </div>
{% else %}
    <!-- Normal meal card display -->
    <div class="meal-card">
        <h3>{{ meal_slot.name }}</h3>
        ...
    </div>
{% endif %}
```

**Community Route Handler:**
```rust
// src/routes/recipes/community.rs
pub async fn get_community_recipes(
    State(state): State<AppState>,
    Query(params): Query<CommunityFilters>,
) -> impl IntoResponse {
    let recipes = queries::recipes::get_community_recipes(
        &state.pool,
        params.recipe_type,
        params.page.unwrap_or(0),
    ).await?;

    CommunityRecipesTemplate {
        recipes,
        active_filter: params.recipe_type,
        ...
    }.into_response()
}
```

**Visual Mockup Alignment:**
- Empty slot styling matches `mockups/calendar-premium.html` empty state placeholders
- "Browse community recipes" link matches mockup CTA design
- Community page filtered view matches `mockups/community.html` filter badges
- Favorite button integration matches mockup quick-action buttons

**Integration with Epic 5:**
- Story 5.1 implements recipe sharing (is_shared flag)
- Story 5.2 implements full community browse page
- This story provides the entry point from empty meal slots to community discovery
- Favorite button integration connects to Story 2.3 (favorites system)

### References

- [Source: docs/epics.md#Story 4.8 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR034 - Empty meal slots display "Browse community recipes" link]
- [Source: docs/PRD.md#FR021 - Graceful handling of insufficient recipes]
- [Source: docs/architecture.md#HTTP Routes - /recipes/community route contract]
- [Source: docs/architecture.md#Data Architecture - recipes table with is_shared flag]
- [Source: CLAUDE.md#Server-Side Rendering Rules - Twinspark for async actions]

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
