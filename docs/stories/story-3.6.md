# Story 3.6: Replace Individual Meal Slot

Status: Approved

## Story

As a **user**,
I want to **replace a single meal in my plan**,
so that **I can adjust for schedule changes or preferences**.

## Acceptance Criteria

1. "Replace This Meal" button visible on each calendar slot
2. System offers 3-5 alternative recipes matching constraints
3. Alternatives respect rotation (only unused recipes)
4. Selected recipe immediately replaces meal in calendar (AJAX update)
5. Replaced recipe returned to rotation pool (available again)
6. Shopping list automatically updates
7. Confirmation message: "Meal replaced successfully"

## Tasks / Subtasks

### Task 1: Implement POST /plan/meal/:id/replace Route (AC: 1, 4, 7)
- [ ] Create `ReplaceMealSlotForm` struct with `new_recipe_id` field
  - [ ] Add to `src/routes/meal_plan.rs` with serde derives
  - [ ] Add validation (recipe_id must be valid UUID)
- [ ] Implement `replace_meal_slot` route handler
  - [ ] Accept `Path(assignment_id)` and `Form(ReplaceMealSlotForm)`
  - [ ] Query `meal_assignment` by ID from read model
  - [ ] Validate assignment belongs to user's active meal plan (authorization)
  - [ ] Validate new recipe belongs to user and is favorited
- [ ] Invoke domain command `meal_planning::replace_meal_slot(cmd)`
  - [ ] Pass `meal_plan_id`, `date`, `meal_type`, `new_recipe_id`, `replacement_reason`
  - [ ] Handle domain errors (recipe not available, etc.)
- [ ] Return AJAX HTML fragment (MealSlotPartial template)
  - [ ] Render updated meal slot with new recipe
  - [ ] Include success confirmation in response
- [ ] Write integration test:
  - [ ] Test: Replace meal slot updates database
  - [ ] Test: Returns HTML fragment with new recipe
  - [ ] Test: Authorization check prevents cross-user replacement

### Task 2: Implement Domain Command - ReplaceMealSlot (AC: 3, 5)
- [ ] Create `ReplaceMealSlotCommand` struct in `crates/meal_planning/src/commands.rs`
  - [ ] Fields: meal_plan_id, date, meal_type, new_recipe_id, replacement_reason
- [ ] Implement `replace_meal_slot()` function in `crates/meal_planning/src/lib.rs`
  - [ ] Load MealPlan aggregate from evento event stream
  - [ ] Validate new recipe not already used in current rotation
  - [ ] Query rotation state to check recipe availability
  - [ ] Mark old recipe as available in rotation (return to pool)
  - [ ] Mark new recipe as used in rotation
  - [ ] Emit `MealSlotReplaced` event with old/new recipe IDs
- [ ] Update MealPlan aggregate handler for `MealSlotReplaced` event
  - [ ] Find assignment by date + meal_type
  - [ ] Update assignment.recipe_id to new_recipe_id
  - [ ] Update assignment.assignment_reasoning (new reasoning text)
- [ ] Write unit tests:
  - [ ] Test: Replace meal slot emits MealSlotReplaced event
  - [ ] Test: Rotation state updated correctly (old available, new used)
  - [ ] Test: Reject replacement if new recipe already used in rotation
  - [ ] Test: Aggregate state reflects new recipe assignment

### Task 3: Update Rotation Manager for Meal Replacement (AC: 3, 5)
- [ ] Add `unmark_recipe_used()` method to RotationManager
  - [ ] Remove recipe_id from used_recipe_ids HashSet
  - [ ] Validate recipe was actually used before unmarking
- [ ] Implement rotation update logic in replacement flow
  - [ ] Query current rotation state for user
  - [ ] Call `rotation.unmark_recipe_used(old_recipe_id)`
  - [ ] Call `rotation.mark_recipe_used(new_recipe_id)`
  - [ ] Emit `RecipeUsedInRotation` events for both changes
- [ ] Write unit tests:
  - [ ] Test: unmark_recipe_used() removes recipe from used set
  - [ ] Test: Recipe becomes available after unmarking
  - [ ] Test: Replacement maintains rotation cycle integrity

### Task 4: Create Alternative Recipe Selection UI (AC: 2, 3)
- [ ] Modify "Replace This Meal" button in meal-slot template
  - [ ] Change from direct POST to modal trigger
  - [ ] Add TwinSpark attributes to open modal
- [ ] Create `replace-meal-modal.html` template component
  - [ ] Display 3-5 alternative recipes in selectable list
  - [ ] Show recipe title, complexity, prep time for each option
  - [ ] Indicate which recipes unused in rotation (highlight)
  - [ ] Include "Cancel" button to close modal
- [ ] Implement `GET /plan/meal/:id/alternatives` route
  - [ ] Query meal assignment to get current recipe and context
  - [ ] Query user's favorite recipes filtered by:
    - Same meal type (breakfast/lunch/dinner)
    - Not used in current rotation
    - Match or improve constraints (complexity, timing)
  - [ ] Rank alternatives by suitability score
  - [ ] Return top 3-5 alternatives
  - [ ] Render modal template with alternatives list
- [ ] Add selection handler in modal
  - [ ] Each alternative has "Select" button
  - [ ] Button triggers POST /plan/meal/:id/replace with selected recipe_id
- [ ] Write integration test:
  - [ ] Test: GET /alternatives returns unused recipes only
  - [ ] Test: Alternatives respect meal type constraint
  - [ ] Test: Modal displays correct number of alternatives (3-5)

### Task 5: Update Read Model Projection (AC: 4, 6)
- [ ] Implement `project_meal_slot_replaced()` subscription handler
  - [ ] Listen for `MealSlotReplaced` events
  - [ ] Update `meal_assignments` table:
    - SET recipe_id = new_recipe_id
    - SET assignment_reasoning = new reasoning
    - WHERE meal_plan_id = ? AND date = ? AND meal_type = ?
  - [ ] Update `recipe_rotation_state` table:
    - DELETE old recipe usage record
    - INSERT new recipe usage record with current timestamp
- [ ] Trigger shopping list recalculation
  - [ ] Emit `ShoppingListUpdateRequested` event
  - [ ] Include meal_plan_id in event data
  - [ ] Shopping domain subscribes and regenerates list
- [ ] Write integration test:
  - [ ] Test: MealSlotReplaced updates meal_assignments table
  - [ ] Test: Rotation state updated in database
  - [ ] Test: ShoppingListUpdateRequested event emitted

### Task 6: Create MealSlotPartial Template (AC: 4, 7)
- [ ] Create `templates/partials/meal-slot-updated.html`
  - [ ] Render same structure as meal-slot component
  - [ ] Include success toast notification HTML
  - [ ] Set id="meal-slot-{{ assignment.id }}" for TwinSpark targeting
- [ ] Update meal-slot template to be reusable
  - [ ] Extract meal slot rendering to macro or include
  - [ ] Ensure both full calendar and partial use same template
- [ ] Add success toast component
  - [ ] Create `templates/components/toast.html`
  - [ ] Auto-dismiss after 3 seconds (JavaScript)
  - [ ] Success styling (green background, checkmark icon)
- [ ] Write integration test:
  - [ ] Test: Partial template renders with new recipe data
  - [ ] Test: Success toast HTML included in response
  - [ ] Test: meal-slot-{{ id }} id matches original slot

### Task 7: Wire TwinSpark AJAX Behavior (AC: 4)
- [ ] Update "Replace This Meal" button TwinSpark attributes
  - [ ] ts-req="/plan/meal/{{ assignment.id }}/alternatives"
  - [ ] ts-req-method="GET"
  - [ ] ts-target="#replace-modal-container"
  - [ ] ts-swap="innerHTML"
- [ ] Add modal container to recipe detail template
  - [ ] `<div id="replace-modal-container"></div>` in base layout
  - [ ] Hidden by default
- [ ] Implement modal selection POST
  - [ ] Each "Select" button in modal:
    - ts-req="/plan/meal/{{ assignment.id }}/replace"
    - ts-req-method="POST"
    - ts-target="#meal-slot-{{ assignment.id }}"
    - ts-swap="outerHTML"
  - [ ] Form data: new_recipe_id from button value
- [ ] Write E2E test (Playwright):
  - [ ] Test: Click "Replace This Meal" opens modal
  - [ ] Test: Select alternative recipe updates calendar
  - [ ] Test: Calendar updates without full page reload
  - [ ] Test: Toast notification appears

### Task 8: Write Comprehensive Test Suite (TDD)
- [ ] **Unit tests** (domain logic):
  - [ ] Test: ReplaceMealSlotCommand validates inputs
  - [ ] Test: RotationManager unmark/mark cycle
  - [ ] Test: MealSlotReplaced event handler updates aggregate
- [ ] **Integration tests** (full HTTP flow):
  - [ ] Test: POST /plan/meal/:id/replace with valid data succeeds
  - [ ] Test: Replace updates meal_assignments table
  - [ ] Test: Rotation state updated correctly
  - [ ] Test: ShoppingListUpdateRequested event emitted
  - [ ] Test: Unauthorized user cannot replace others' meals
  - [ ] Test: Invalid recipe_id returns 400 error
  - [ ] Test: Already-used recipe returns validation error
- [ ] **E2E tests** (Playwright):
  - [ ] Test: User clicks "Replace This Meal" from calendar
  - [ ] Test: Modal displays alternative recipes
  - [ ] Test: Select alternative updates calendar instantly
  - [ ] Test: Toast confirmation appears
  - [ ] Test: Shopping list page reflects ingredient changes
- [ ] Test coverage: Target 80%+ via cargo tarpaulin

## Dev Notes

### Architecture Patterns
- **Event Sourcing**: MealSlotReplaced event persisted to evento stream
- **CQRS**: Command updates aggregate, read model projection updates meal_assignments
- **Domain Events**: ShoppingListUpdateRequested triggers cross-domain update
- **Server-Side Rendering**: Askama templates for modal and partial responses
- **Progressive Enhancement**: TwinSpark for AJAX modal and slot replacement

### Key Components
- **Route**: `src/routes/meal_plan.rs::replace_meal_slot()` (NEW handler)
- **Domain Command**: `crates/meal_planning/src/lib.rs::replace_meal_slot()` (NEW)
- **Aggregate**: `crates/meal_planning/src/aggregate.rs::MealPlan` (UPDATE event handler)
- **Rotation Manager**: `crates/meal_planning/src/rotation.rs::RotationManager` (UPDATE with unmark method)
- **Read Model Projection**: `crates/meal_planning/src/read_model.rs::project_meal_slot_replaced()` (NEW)
- **Templates**:
  - `templates/partials/meal-slot-updated.html` (NEW partial)
  - `templates/components/replace-meal-modal.html` (NEW modal)
  - `templates/components/toast.html` (NEW toast notification)
  - `templates/components/meal-slot.html` (UPDATE with modal trigger)

### Data Flow
1. **User clicks "Replace This Meal"**:
   - GET /plan/meal/:id/alternatives
   - Route handler queries unused favorite recipes matching meal type
   - Render modal with 3-5 alternatives
   - TwinSpark injects modal HTML into DOM

2. **User selects alternative recipe**:
   - POST /plan/meal/:id/replace with new_recipe_id
   - Route handler validates authorization and recipe availability
   - Invoke domain command: meal_planning::replace_meal_slot()
   - Domain layer:
     - Load MealPlan aggregate from event stream
     - Validate new recipe not in rotation
     - Unmark old recipe, mark new recipe in RotationManager
     - Emit MealSlotReplaced event
   - Evento subscription:
     - Update meal_assignments table (recipe_id, reasoning)
     - Update recipe_rotation_state table
     - Emit ShoppingListUpdateRequested event
   - Route handler renders MealSlotPartial with success toast
   - TwinSpark swaps meal-slot HTML, displays toast

3. **Shopping list updates automatically**:
   - Shopping domain subscription listens for ShoppingListUpdateRequested
   - Regenerates shopping list with new recipe ingredients
   - User sees updated list on /shopping page

### Project Structure Notes

**Alignment with Solution Architecture**:
- **evento Aggregate Pattern**: MealPlan aggregate handles MealSlotReplaced event (docs/solution-architecture.md#Event Sourcing)
- **CQRS Read Models**: meal_assignments table updated via projection (docs/solution-architecture.md#CQRS Implementation)
- **TwinSpark AJAX**: Partial HTML response swapped without page reload (docs/solution-architecture.md#TwinSpark Progressive Enhancement)
- **Route Structure**: Follows /plan prefix for meal planning routes (docs/solution-architecture.md#Page Routing)

**Lessons from Story 3.5**:
- **Error Handling**: Use proper match statements instead of .unwrap() (Story 3.5 action item #1)
- **External JavaScript**: Extract inline JS to separate files for CSP compliance (Story 3.5 action item #2)
- **ARIA Landmarks**: Add role attributes for accessibility (Story 3.5 action item #3)
- **Keyboard Navigation**: Support keyboard shortcuts for modal (Story 3.5 action item #4)
- **Test Coverage**: Maintain 80%+ coverage with integration tests (Story 3.5 achieved 82 passing tests)

**New Components**:
- `src/routes/meal_plan.rs::replace_meal_slot()` - New route handler for meal replacement
- `crates/meal_planning/src/rotation.rs::unmark_recipe_used()` - New method for rotation pool management
- `templates/partials/meal-slot-updated.html` - New partial template for AJAX response
- `templates/components/replace-meal-modal.html` - New modal for alternative selection

### References

- [Source: docs/epics.md#Story 3.6] Replace Individual Meal Slot requirements (lines 679-702)
- [Source: docs/tech-spec-epic-3.md#Story 3.6] Implementation checklist and acceptance criteria (lines 1291-1299)
- [Source: docs/tech-spec-epic-3.md#Workflow 2] Replace meal slot workflow sequence (lines 952-999)
- [Source: docs/tech-spec-epic-3.md#RotationManager] Rotation state management (lines 233-275)
- [Source: docs/tech-spec-epic-3.md#MealSlotReplaced Event] Event definition and handling (lines 300-307, 385-398)
- [Source: docs/solution-architecture.md#TwinSpark] Progressive enhancement patterns (lines 536-558)
- [Source: docs/solution-architecture.md#Server-Side Rendering] Askama template patterns (lines 122-141)
- [Source: docs/solution-architecture.md#CQRS] Command/query segregation (lines 206-249)
- [Source: Story 3.5 Completion Notes] Lessons learned on error handling, accessibility, CSP compliance (lines 224-490)

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.6.xml` (Generated: 2025-10-17)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
