# Story 9.4: Update Recipe Creation Form with Accompaniment Fields

Status: Approved

## Story

As a frontend developer,
I want to add accompaniment settings to recipe creation form,
so that users can configure main courses to accept sides.

## Acceptance Criteria

1. Recipe form updated at `templates/recipes/create_recipe.html` to include new fields
2. Recipe type selection includes "Accompaniment" option (radio buttons: Appetizer, Main Course, Dessert, Accompaniment)
3. For Main Course: checkbox "This dish accepts an accompaniment" (name="accepts_accompaniment")
4. If checked: show preferred categories checkboxes (Pasta, Rice, Fries, Salad, Bread, Vegetable, Other)
5. For Accompaniment type: show category radio buttons (same options as preferred categories)
6. Cuisine selection: dropdown with variants (Italian, Indian, Mexican, Chinese, etc.) + Custom text input
7. Dietary tags: checkbox list (Vegetarian, Vegan, Gluten-Free, Dairy-Free, Nut-Free, Halal, Kosher)
8. Form submission includes all new fields in POST body to `/recipes`
9. Validation: If recipe_type="accompaniment", category required (server-side validation)
10. Playwright test verifies form submission with new fields creates recipe with accompaniment data

## Tasks / Subtasks

- [ ] Update recipe form template structure (AC: #1)
  - [ ] Open `templates/recipes/create_recipe.html` for modification
  - [ ] Add new fieldsets for accompaniment settings
  - [ ] Maintain existing fields (title, ingredients, instructions, prep time, etc.)
  - [ ] Ensure form action points to `POST /recipes`

- [ ] Implement recipe type selection (AC: #2)
  - [ ] Add radio button group: Appetizer, Main Course, Dessert, Accompaniment
  - [ ] Name attribute: `name="recipe_type"`
  - [ ] Values: "appetizer", "main_course", "dessert", "accompaniment"
  - [ ] Default selection: Main Course
  - [ ] Style with Tailwind CSS for clear visual grouping

- [ ] Add accompaniment acceptance checkbox (AC: #3)
  - [ ] Add checkbox: `name="accepts_accompaniment"`, conditional display for Main Course
  - [ ] Label: "This dish accepts an accompaniment"
  - [ ] Show/hide using JavaScript or TwinSpark based on recipe_type selection
  - [ ] Initially hidden, shown when "Main Course" selected

- [ ] Add preferred accompaniment categories (AC: #4)
  - [ ] Create checkbox list: Pasta, Rice, Fries, Salad, Bread, Vegetable, Other
  - [ ] Name: `name="preferred_accompaniments[]"`, values match categories
  - [ ] Conditional display: shown when "accepts_accompaniment" checked
  - [ ] Group in fieldset with legend "Preferred Accompaniment Categories"
  - [ ] Progressive disclosure pattern (show/hide based on parent checkbox)

- [ ] Add accompaniment category radio buttons (AC: #5)
  - [ ] Create radio button group with same categories as preferred list
  - [ ] Name: `name="accompaniment_category"`
  - [ ] Conditional display: shown when recipe_type="accompaniment"
  - [ ] Required validation when Accompaniment type selected
  - [ ] Style consistently with other radio groups

- [ ] Implement cuisine selection (AC: #6)
  - [ ] Add dropdown (select element) with cuisine options
  - [ ] Options: Italian, Indian, Mexican, Chinese, French, Thai, Japanese, American, Mediterranean, etc.
  - [ ] Add "Custom" option that reveals text input field
  - [ ] Text input: `name="custom_cuisine"`, shown when "Custom" selected
  - [ ] Pre-populate common cuisine types from backend enum

- [ ] Add dietary tags checkboxes (AC: #7)
  - [ ] Create checkbox list: Vegetarian, Vegan, Gluten-Free, Dairy-Free, Nut-Free, Halal, Kosher
  - [ ] Name: `name="dietary_tags[]"`
  - [ ] Allow multiple selections
  - [ ] Group in fieldset with legend "Dietary Tags"
  - [ ] Consistent styling with other checkbox groups

- [ ] Configure form submission (AC: #8)
  - [ ] Verify form method: `method="POST"`
  - [ ] Verify form action: `action="/recipes"`
  - [ ] Include CSRF token
  - [ ] Ensure all new fields included in form submission
  - [ ] Test: Submit form → Verify all fields received by backend

- [ ] Implement validation (AC: #9)
  - [ ] Add HTML5 validation: required attributes where applicable
  - [ ] Server-side validation: If recipe_type="accompaniment", category required
  - [ ] Backend returns 422 with inline errors for validation failures
  - [ ] Display error messages next to invalid fields (text-red-500)
  - [ ] Test validation: Submit accompaniment without category → Verify error shown

- [ ] Progressive disclosure with JavaScript/TwinSpark
  - [ ] Show/hide "accepts_accompaniment" when Main Course selected
  - [ ] Show/hide preferred categories when "accepts_accompaniment" checked
  - [ ] Show/hide accompaniment category when Accompaniment type selected
  - [ ] Show/hide custom cuisine input when "Custom" selected
  - [ ] Implement with minimal JavaScript or TwinSpark server-driven approach

- [ ] Responsive design and styling
  - [ ] Full-width form on mobile (w-full)
  - [ ] Constrained width on desktop (max-w-2xl)
  - [ ] Clear visual grouping with fieldsets and legends
  - [ ] Consistent spacing (space-y-4, space-y-6)
  - [ ] Test on mobile (375px), tablet (768px), desktop (1920px)

- [ ] Integration testing (AC: #10)
  - [ ] Write Playwright test for recipe creation with accompaniment fields
  - [ ] Test: Create main course with accepts_accompaniment=true, preferred=[Rice, Pasta]
  - [ ] Test: Create accompaniment with type=Rice
  - [ ] Test: Verify recipes saved with correct accompaniment data
  - [ ] Test: Validation error when accompaniment type but no category selected
  - [ ] Test: Cuisine selection with custom cuisine input

## Dev Notes

### Architecture Patterns and Constraints

- **Progressive Disclosure:** Show/hide fields based on user selections (JavaScript or TwinSpark)
- **Form Validation:** HTML5 client-side + server-side validation for security
- **Type-Safe Form Handling:** Backend deserializes into `CreateRecipeForm` DTO
- **CSRF Protection:** Include CSRF token in all mutating forms

### Source Tree Components

**Templates:**
- `templates/recipes/create_recipe.html` - Recipe creation form (modify existing)
- `templates/recipes/recipe_detail.html` - Recipe detail page (no change, already displays accompaniment data)

**Backend Routes (Epic 8/existing):**
- `GET /recipes/new` - Serve recipe creation form
- `POST /recipes` - Process form submission, create recipe

**Domain Models (Epic 6):**
- `Recipe` aggregate with accompaniment fields: accepts_accompaniment, preferred_accompaniments, accompaniment_category, cuisine, dietary_tags
- `CreateRecipe` command with all fields

**Form DTO:**
```rust
pub struct CreateRecipeForm {
    pub title: String,
    pub ingredients: Vec<IngredientInput>,
    pub instructions: Vec<String>,
    pub prep_time_min: u32,
    pub cook_time_min: u32,
    pub advance_prep_text: Option<String>,
    pub recipe_type: String,                   // "appetizer" | "main_course" | "dessert" | "accompaniment"
    pub accepts_accompaniment: bool,           // checkbox
    pub preferred_accompaniments: Vec<String>, // checkboxes: ["Rice", "Pasta"]
    pub accompaniment_category: Option<String>, // radio button
    pub cuisine: Option<String>,               // dropdown
    pub custom_cuisine: Option<String>,        // text input when Custom selected
    pub dietary_tags: Vec<String>,             // checkboxes
}
```

### Testing Standards

- **Integration Tests:** Playwright verifies form submission and validation
- **Accessibility:** Proper labels, fieldsets, radio/checkbox groups
- **Progressive Disclosure:** Show/hide logic tested across browser types
- **Validation:** Server-side validation cannot be bypassed

### Progressive Disclosure Pattern (JavaScript)

```html
<script>
  // Show/hide accompaniment acceptance checkbox
  document.querySelectorAll('input[name="recipe_type"]').forEach(radio => {
    radio.addEventListener('change', (e) => {
      const mainCourseFields = document.getElementById('main-course-fields');
      const accompanimentFields = document.getElementById('accompaniment-fields');

      if (e.target.value === 'main_course') {
        mainCourseFields.style.display = 'block';
        accompanimentFields.style.display = 'none';
      } else if (e.target.value === 'accompaniment') {
        mainCourseFields.style.display = 'none';
        accompanimentFields.style.display = 'block';
      } else {
        mainCourseFields.style.display = 'none';
        accompanimentFields.style.display = 'none';
      }
    });
  });

  // Show/hide preferred categories
  document.getElementById('accepts_accompaniment').addEventListener('change', (e) => {
    const preferredCategories = document.getElementById('preferred-categories');
    preferredCategories.style.display = e.target.checked ? 'block' : 'none';
  });
</script>
```

**Alternative: TwinSpark Server-Driven Approach** (no JavaScript)
- Each selection triggers server request to re-render form section
- Progressive enhancement: form works without JS (all fields visible by default)

### Form Structure Additions

```html
<!-- Recipe Type Selection -->
<fieldset>
  <legend>Recipe Type</legend>
  <label><input type="radio" name="recipe_type" value="appetizer" /> Appetizer</label>
  <label><input type="radio" name="recipe_type" value="main_course" checked /> Main Course</label>
  <label><input type="radio" name="recipe_type" value="dessert" /> Dessert</label>
  <label><input type="radio" name="recipe_type" value="accompaniment" /> Accompaniment</label>
</fieldset>

<!-- Main Course: Accepts Accompaniment -->
<div id="main-course-fields" style="display: block;">
  <label>
    <input type="checkbox" name="accepts_accompaniment" id="accepts_accompaniment" />
    This dish accepts an accompaniment
  </label>

  <div id="preferred-categories" style="display: none;">
    <fieldset>
      <legend>Preferred Accompaniment Categories</legend>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Pasta" /> Pasta</label>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Rice" /> Rice</label>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Fries" /> Fries</label>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Salad" /> Salad</label>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Bread" /> Bread</label>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Vegetable" /> Vegetable</label>
      <label><input type="checkbox" name="preferred_accompaniments[]" value="Other" /> Other</label>
    </fieldset>
  </div>
</div>

<!-- Accompaniment: Category Selection -->
<div id="accompaniment-fields" style="display: none;">
  <fieldset>
    <legend>Accompaniment Category</legend>
    <label><input type="radio" name="accompaniment_category" value="Pasta" /> Pasta</label>
    <label><input type="radio" name="accompaniment_category" value="Rice" /> Rice</label>
    <label><input type="radio" name="accompaniment_category" value="Fries" /> Fries</label>
    <label><input type="radio" name="accompaniment_category" value="Salad" /> Salad</label>
    <label><input type="radio" name="accompaniment_category" value="Bread" /> Bread</label>
    <label><input type="radio" name="accompaniment_category" value="Vegetable" /> Vegetable</label>
    <label><input type="radio" name="accompaniment_category" value="Other" /> Other</label>
  </fieldset>
</div>

<!-- Cuisine Selection -->
<label for="cuisine">Cuisine</label>
<select id="cuisine" name="cuisine">
  <option value="">Select cuisine...</option>
  <option value="Italian">Italian</option>
  <option value="Indian">Indian</option>
  <option value="Mexican">Mexican</option>
  <option value="Chinese">Chinese</option>
  <option value="French">French</option>
  <option value="Thai">Thai</option>
  <option value="Japanese">Japanese</option>
  <option value="American">American</option>
  <option value="Mediterranean">Mediterranean</option>
  <option value="Custom">Custom</option>
</select>

<div id="custom-cuisine" style="display: none;">
  <label for="custom_cuisine_input">Custom Cuisine</label>
  <input type="text" id="custom_cuisine_input" name="custom_cuisine" />
</div>

<!-- Dietary Tags -->
<fieldset>
  <legend>Dietary Tags</legend>
  <label><input type="checkbox" name="dietary_tags[]" value="Vegetarian" /> Vegetarian</label>
  <label><input type="checkbox" name="dietary_tags[]" value="Vegan" /> Vegan</label>
  <label><input type="checkbox" name="dietary_tags[]" value="Gluten-Free" /> Gluten-Free</label>
  <label><input type="checkbox" name="dietary_tags[]" value="Dairy-Free" /> Dairy-Free</label>
  <label><input type="checkbox" name="dietary_tags[]" value="Nut-Free" /> Nut-Free</label>
  <label><input type="checkbox" name="dietary_tags[]" value="Halal" /> Halal</label>
  <label><input type="checkbox" name="dietary_tags[]" value="Kosher" /> Kosher</label>
</fieldset>
```

### Project Structure Notes

**Modified Files:**
- `templates/recipes/create_recipe.html` - Add new form fields and progressive disclosure logic

**Backend Integration:**
- `src/routes/recipes.rs` - Form handler already exists (Epic 6), may need updates for new fields
- Validation: Ensure accompaniment_category required if recipe_type="accompaniment"

### References

- [Source: docs/tech-spec-epic-9.md#Acceptance Criteria → Story 9.4 (AC-9.4.1 through AC-9.4.10)]
- [Source: docs/tech-spec-epic-9.md#Data Models and Contracts → Form Data Transfer Objects → CreateRecipeForm]
- [Source: docs/tech-spec-epic-9.md#Workflows → Workflow 4: Recipe Creation with Accompaniment Fields]
- [Source: docs/epics.md#Epic 9 → Story 9.4]
- [Source: docs/tech-spec-epic-6.md (Recipe domain model with accompaniment fields)]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-9.4.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
