# imkitchen - Technical Specification: Batch Recipe Import

**Author:** Jonathan
**Date:** 2025-10-21
**Project Level:** Level 1 (Feature Addition to Level 3 Project)
**Project Type:** Web Application (PWA)
**Development Context:** Adding batch import functionality to existing recipe management system

---

## Source Tree Structure

```
imkitchen/
├── src/
│   └── routes/
│       └── recipes.rs                    # ✏️ MODIFIED: Add batch import handler
├── templates/
│   ├── pages/
│   │   └── recipe-list.html              # ✏️ MODIFIED: Add import button next to "New recipe"
│   └── components/
│       └── batch-import-modal.html       # 🆕 NEW: Modal for file upload and results
├── static/
│   └── js/
│       └── batch-import.js               # 🆕 NEW: Client-side file validation (optional progressive enhancement)
├── crates/
│   └── recipe/
│       ├── src/
│       │   ├── commands.rs                # ✏️ MODIFIED: Add BatchImportRecipesCommand
│       │   ├── events.rs                  # ✏️ MODIFIED: Add BatchImportCompleted event
│       │   └── lib.rs                     # ✏️ MODIFIED: Export batch import command
│       └── tests/
│           └── batch_import_tests.rs      # 🆕 NEW: Unit tests for batch import
└── tests/
    └── batch_import_integration_tests.rs  # 🆕 NEW: Integration tests

```

**File Change Summary:**
- **2 new files**: `batch-import-modal.html`, `batch_import_tests.rs`
- **1 optional new file**: `batch-import.js` (progressive enhancement)
- **1 new integration test**: `batch_import_integration_tests.rs`
- **4 modified files**: `recipes.rs`, `recipe-list.html`, `commands.rs`, `events.rs`, `lib.rs`

---

## Technical Approach

### Overview
Add batch recipe import capability allowing users to upload a JSON file containing an array of recipes. Each recipe must follow the format defined in `example-recipe.json`. The feature will:

1. **UI Addition**: Place "Import Recipes" button next to "New recipe" button in recipe list page
2. **File Upload**: Accept `.json` files via HTML file input
3. **Validation**: Server-side validation of JSON structure and recipe schema
4. **Batch Processing**: Create multiple recipes in a single transaction using evento
5. **User Feedback**: Display success/error messages with per-recipe results
6. **Free Tier Limit**: Enforce 10-recipe limit for free users (reject batch if it would exceed limit)

### Architecture Pattern
- **Command**: `BatchImportRecipesCommand` (new command in recipe crate)
- **Event**: `BatchImportCompleted { successful: Vec<RecipeId>, failed: Vec<(usize, String)> }`
- **Route**: `POST /recipes/import` (new endpoint)
- **Template**: Server-rendered modal with TwinSpark for progressive enhancement

### User Flow
1. User clicks "Import Recipes" button → Modal opens
2. User selects JSON file → Client-side preview (optional, progressive enhancement)
3. User clicks "Upload" → POST to `/recipes/import`
4. Server validates JSON array format
5. Server validates each recipe against schema
6. Server checks free tier limit (current count + import count ≤ 10)
7. Server creates recipes via `BatchImportRecipesCommand`
8. Server returns results → Modal displays success/failure per recipe
9. Page refreshes to show new recipes

---

## Implementation Stack

### Backend (Rust)
- **HTTP Handling**: Axum 0.8+ (multipart form for file upload)
- **JSON Parsing**: `serde_json` 1.0+ (existing dependency)
- **Validation**: `validator` 0.20+ (existing dependency)
- **Event Sourcing**: `evento` 1.3+ (existing dependency)
- **Database**: SQLite via `sqlx` 0.8+ (existing dependency)

### Frontend (Server-Rendered)
- **Templates**: Askama 0.14+ (existing dependency)
- **Progressive Enhancement**: TwinSpark (existing dependency)
- **File Upload**: Native HTML `<input type="file" accept=".json">`

### Testing
- **Unit Tests**: Rust `#[test]` with evento in-memory executor
- **Integration Tests**: Axum test server with SQLite in-memory database
- **E2E Tests**: Playwright 1.56+ (existing test infrastructure)

---

## Technical Details

### 1. JSON File Format

**Expected Input Format** (array of recipes):
```json
[
  {
    "title": "Classic Spaghetti Carbonara",
    "recipe_type": "main_course",
    "ingredients": [
      {
        "name": "spaghetti",
        "quantity": 400.0,
        "unit": "g"
      }
    ],
    "instructions": [
      {
        "step_number": 1,
        "instruction_text": "Bring water to boil.",
        "timer_minutes": null
      }
    ],
    "prep_time_min": 10,
    "cook_time_min": 20,
    "advance_prep_hours": null,
    "serving_size": 4
  },
  {
    // ... second recipe
  }
]
```

**Validation Rules:**
- Root must be JSON array `[]`
- Each array element must be valid recipe object
- **Required fields per recipe**:
  - `title` (string, 3-200 chars)
  - `recipe_type` (enum: "appetizer", "main_course", "dessert")
  - `ingredients` (array, min 1 ingredient)
  - `instructions` (array, min 1 instruction)
- **Optional fields**:
  - `prep_time_min`, `cook_time_min`, `advance_prep_hours`, `serving_size` (numbers)
- **Ingredient object** (required fields):
  - `name` (string), `quantity` (f32), `unit` (string)
- **Instruction object** (required fields):
  - `step_number` (u32), `instruction_text` (string), `timer_minutes` (Option<u32>)

### 2. Backend Implementation

#### New Command Structure (`crates/recipe/src/commands.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchImportRecipe {
    pub title: String,
    pub recipe_type: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<InstructionStep>,
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchImportRecipesCommand {
    pub user_id: String,
    pub recipes: Vec<BatchImportRecipe>,
}

#[derive(Debug, Clone, Serialize, Deserialize, evento::Event)]
pub struct BatchImportCompleted {
    pub user_id: String,
    pub successful_recipe_ids: Vec<String>,
    pub failed_imports: Vec<(usize, String)>, // (index, error_message)
    pub total_attempted: usize,
}
```

#### Batch Import Logic (`crates/recipe/src/lib.rs`)

```rust
pub async fn batch_import_recipes(
    cmd: BatchImportRecipesCommand,
    executor: &impl evento::Executor,
) -> Result<BatchImportResult, RecipeError> {
    let mut successful_ids = Vec::new();
    let mut failed_imports = Vec::new();

    // 1. Check free tier limit
    let current_count = query_recipe_count_by_user(&cmd.user_id, executor).await?;
    let total_after_import = current_count + cmd.recipes.len();

    let user_tier = query_user_tier(&cmd.user_id, executor).await?;
    if user_tier == "free" && total_after_import > 10 {
        return Err(RecipeError::RecipeLimitExceeded {
            current: current_count,
            attempted: cmd.recipes.len(),
            limit: 10,
        });
    }

    // 2. Process each recipe
    for (index, recipe_data) in cmd.recipes.iter().enumerate() {
        // Validate recipe
        match validate_recipe(recipe_data) {
            Ok(_) => {},
            Err(e) => {
                failed_imports.push((index, format!("Validation failed: {}", e)));
                continue;
            }
        }

        // Create recipe via existing command
        let create_cmd = CreateRecipeCommand {
            user_id: cmd.user_id.clone(),
            title: recipe_data.title.clone(),
            recipe_type: recipe_data.recipe_type.clone(),
            ingredients: recipe_data.ingredients.clone(),
            instructions: recipe_data.instructions.clone(),
            prep_time_min: recipe_data.prep_time_min,
            cook_time_min: recipe_data.cook_time_min(),
            advance_prep_hours: recipe_data.advance_prep_hours,
            serving_size: recipe_data.serving_size,
        };

        match create_recipe(create_cmd, executor).await {
            Ok(recipe_id) => successful_ids.push(recipe_id),
            Err(e) => failed_imports.push((index, format!("Creation failed: {}", e))),
        }
    }

    Ok(BatchImportResult {
        successful_recipe_ids: successful_ids,
        failed_imports,
        total_attempted: cmd.recipes.len(),
    })
}
```

#### Route Handler (`src/routes/recipes.rs`)

```rust
use axum::extract::Multipart;

pub async fn post_import_recipes(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    mut multipart: Multipart,
) -> Response {
    // 1. Extract file from multipart form
    let mut file_contents = String::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("recipes_file") {
            file_contents = match field.text().await {
                Ok(text) => text,
                Err(e) => {
                    return render_error("Failed to read file", &auth);
                }
            };
            break;
        }
    }

    // 2. Parse JSON array
    let recipes: Vec<BatchImportRecipe> = match serde_json::from_str(&file_contents) {
        Ok(r) => r,
        Err(e) => {
            return render_import_error(
                format!("Invalid JSON format: {}. Expected array of recipes.", e),
                &auth
            );
        }
    };

    // 3. Validate non-empty
    if recipes.is_empty() {
        return render_import_error("No recipes found in file", &auth);
    }

    // 4. Execute batch import
    let cmd = BatchImportRecipesCommand {
        user_id: auth.user_id.clone(),
        recipes,
    };

    let result = match batch_import_recipes(cmd, &state.write_pool).await {
        Ok(r) => r,
        Err(RecipeError::RecipeLimitExceeded { current, attempted, limit }) => {
            return render_import_error(
                format!(
                    "Import would exceed free tier limit. You have {}/{} recipes. \
                    Attempting to import {} more would exceed the limit.",
                    current, limit, attempted
                ),
                &auth
            );
        }
        Err(e) => {
            return render_import_error(format!("Import failed: {}", e), &auth);
        }
    };

    // 5. Render results modal
    let template = BatchImportResultTemplate {
        successful_count: result.successful_recipe_ids.len(),
        failed_count: result.failed_imports.len(),
        total_attempted: result.total_attempted,
        failures: result.failed_imports,
        user: Some(auth),
    };

    Html(template.render().unwrap()).into_response()
}
```

### 3. UI Implementation

#### Modified Recipe List Template (`templates/pages/recipe-list.html`)

```html
<!-- Existing "New recipe" button -->
<div class="flex gap-4 mb-6">
    <a href="/recipes/new" class="btn-primary">
        <svg><!-- Plus icon --></svg>
        New recipe
    </a>

    <!-- 🆕 NEW: Import button -->
    <button
        ts-req="/recipes/import-modal"
        ts-target="#import-modal"
        ts-swap="innerHTML"
        class="btn-secondary">
        <svg><!-- Upload icon --></svg>
        Import Recipes
    </button>
</div>

<!-- 🆕 NEW: Modal container (empty, populated by TwinSpark) -->
<div id="import-modal"></div>
```

#### New Batch Import Modal Template (`templates/components/batch-import-modal.html`)

```html
<div class="modal-backdrop">
    <div class="modal-content max-w-2xl">
        <div class="modal-header">
            <h2>Import Recipes</h2>
            <button ts-req="/close-modal" ts-target="#import-modal" ts-swap="innerHTML" aria-label="Close">
                ×
            </button>
        </div>

        <div class="modal-body">
            <p class="text-gray-600 mb-4">
                Upload a JSON file containing an array of recipes.
                See <a href="/example-recipe.json" download class="text-primary-500 underline">example-recipe.json</a> for the format.
            </p>

            <form
                action="/recipes/import"
                method="POST"
                enctype="multipart/form-data"
                ts-req="/recipes/import"
                ts-target="#import-modal"
                ts-swap="innerHTML">

                <div class="form-field">
                    <label for="recipes_file">Select JSON File</label>
                    <input
                        type="file"
                        id="recipes_file"
                        name="recipes_file"
                        accept=".json,application/json"
                        required
                        class="file-input" />
                    <p class="text-sm text-gray-500">Accepts .json files only</p>
                </div>

                <div class="modal-footer">
                    <button type="button" ts-req="/close-modal" ts-target="#import-modal" ts-swap="innerHTML" class="btn-secondary">
                        Cancel
                    </button>
                    <button type="submit" class="btn-primary">
                        Upload and Import
                    </button>
                </div>
            </form>
        </div>
    </div>
</div>
```

#### Results Template (`templates/components/batch-import-results.html`)

```html
<div class="modal-backdrop">
    <div class="modal-content max-w-2xl">
        <div class="modal-header">
            <h2>Import Results</h2>
            <button onclick="window.location.reload()" aria-label="Close">×</button>
        </div>

        <div class="modal-body">
            {% if successful_count > 0 %}
            <div class="success-message mb-4">
                <svg class="success-icon"><!-- Checkmark --></svg>
                <p><strong>{{ successful_count }}</strong> recipes imported successfully!</p>
            </div>
            {% endif %}

            {% if failed_count > 0 %}
            <div class="error-message mb-4">
                <svg class="error-icon"><!-- X icon --></svg>
                <p><strong>{{ failed_count }}</strong> recipes failed to import:</p>
                <ul class="mt-2 text-sm">
                    {% for (index, error) in failures %}
                    <li>Recipe #{{ index + 1 }}: {{ error }}</li>
                    {% endfor %}
                </ul>
            </div>
            {% endif %}

            <div class="modal-footer">
                <button onclick="window.location.reload()" class="btn-primary">
                    Done
                </button>
            </div>
        </div>
    </div>
</div>
```

### 4. Error Handling

**Error Scenarios & Responses:**

| Error | HTTP Status | User Message |
|-------|-------------|--------------|
| Invalid JSON syntax | 422 | "Invalid JSON format. Please check your file syntax." |
| Not an array | 422 | "Expected array of recipes. Root element must be [...]" |
| Empty array | 422 | "No recipes found in file" |
| Missing required field | 422 | "Recipe #N: Missing required field 'title'" |
| Invalid recipe_type | 422 | "Recipe #N: recipe_type must be 'appetizer', 'main_course', or 'dessert'" |
| Free tier limit exceeded | 403 | "Import would exceed free tier limit (10 recipes)" |
| Database error | 500 | "Server error during import. Please try again." |

**Partial Success Handling:**
- If some recipes succeed and some fail, show both counts
- Display specific error for each failed recipe
- Successfully imported recipes are saved (no rollback)
- User can fix failed recipes and re-import

---

## Development Setup

**Prerequisites:**
- Existing imkitchen development environment (Rust 1.90+, SQLite)
- No new dependencies required

**Local Testing:**
1. Create test JSON files:
   - `test-batch-valid.json` (2-3 valid recipes)
   - `test-batch-invalid.json` (mix of valid/invalid)
   - `test-batch-empty.json` (empty array `[]`)
   - `test-batch-limit.json` (11 recipes for free tier limit test)

2. Run unit tests:
   ```bash
   cargo test -p recipe batch_import
   ```

3. Run integration tests:
   ```bash
   cargo test batch_import_integration
   ```

4. Manual testing:
   ```bash
   cargo run serve
   # Navigate to http://localhost:3000/recipes
   # Click "Import Recipes", upload test files
   ```

---

## Implementation Guide

### Phase 1: Backend Foundation (TDD)
**Estimated Time: 3-4 hours**

1. **Write failing tests** (`crates/recipe/tests/batch_import_tests.rs`):
   ```rust
   #[tokio::test]
   async fn test_batch_import_valid_recipes() {
       // Arrange: 3 valid recipes
       // Act: batch_import_recipes()
       // Assert: 3 successful, 0 failed
   }

   #[tokio::test]
   async fn test_batch_import_rejects_free_tier_overflow() {
       // Arrange: Free user with 8 recipes, import 3 more
       // Act: batch_import_recipes()
       // Assert: RecipeLimitExceeded error
   }

   #[tokio::test]
   async fn test_batch_import_partial_failure() {
       // Arrange: 2 valid, 1 invalid recipe
       // Act: batch_import_recipes()
       // Assert: 2 successful, 1 failed with specific error
   }
   ```

2. **Implement command and event** (`crates/recipe/src/commands.rs`, `events.rs`):
   - Add `BatchImportRecipe` struct
   - Add `BatchImportRecipesCommand`
   - Add `BatchImportCompleted` event
   - Export from `lib.rs`

3. **Implement batch import logic** (`crates/recipe/src/lib.rs`):
   - Create `batch_import_recipes()` function
   - Reuse existing `create_recipe()` command per recipe
   - Implement free tier limit check
   - Collect success/failure results

4. **Verify tests pass**: `cargo test -p recipe batch_import`

### Phase 2: HTTP Route (TDD)
**Estimated Time: 2-3 hours**

1. **Write integration test** (`tests/batch_import_integration_tests.rs`):
   ```rust
   #[tokio::test]
   async fn test_post_import_recipes_endpoint() {
       let app = spawn_app().await;
       let client = reqwest::Client::new();

       // Create multipart form with JSON file
       let form = multipart::Form::new()
           .file("recipes_file", "test-batch-valid.json")
           .unwrap();

       let response = client
           .post(&format!("{}/recipes/import", app.address))
           .multipart(form)
           .send()
           .await
           .unwrap();

       assert_eq!(response.status(), StatusCode::OK);
       assert!(response.text().await.unwrap().contains("imported successfully"));
   }
   ```

2. **Implement route handler** (`src/routes/recipes.rs`):
   - Add `post_import_recipes()` function
   - Handle multipart file upload
   - Parse JSON to `Vec<BatchImportRecipe>`
   - Call `batch_import_recipes()`
   - Render results template

3. **Register route** (`src/server.rs` or `src/routes/mod.rs`):
   ```rust
   .route("/recipes/import", post(post_import_recipes))
   ```

4. **Verify integration test passes**: `cargo test batch_import_integration`

### Phase 3: UI Templates
**Estimated Time: 2-3 hours**

1. **Create modal template** (`templates/components/batch-import-modal.html`):
   - File upload form
   - TwinSpark attributes for AJAX submission
   - Link to example-recipe.json

2. **Create results template** (`templates/components/batch-import-results.html`):
   - Success/failure counts
   - Per-recipe error messages
   - "Done" button to refresh page

3. **Modify recipe list page** (`templates/pages/recipe-list.html`):
   - Add "Import Recipes" button next to "New recipe"
   - Add empty modal container

4. **Add route for modal** (`src/routes/recipes.rs`):
   ```rust
   pub async fn get_import_modal(Extension(auth): Extension<Auth>) -> Html<String> {
       let template = BatchImportModalTemplate { user: Some(auth) };
       Html(template.render().unwrap())
   }
   ```
   Register: `.route("/recipes/import-modal", get(get_import_modal))`

### Phase 4: Manual & E2E Testing
**Estimated Time: 1-2 hours**

1. **Manual testing**:
   - Test valid batch import (2-3 recipes)
   - Test invalid JSON syntax
   - Test empty array
   - Test free tier limit (create 9 recipes, import 2 more)
   - Test partial failure (mix valid/invalid)

2. **E2E test** (`e2e/tests/batch-import.spec.ts`):
   ```typescript
   test('batch import recipes flow', async ({ page }) => {
     await page.goto('/recipes');
     await page.click('button:has-text("Import Recipes")');

     const fileInput = page.locator('input[type="file"]');
     await fileInput.setInputFiles('fixtures/test-batch.json');

     await page.click('button:has-text("Upload and Import")');

     await expect(page.locator('text=imported successfully')).toBeVisible();
     await page.click('button:has-text("Done")');

     // Verify recipes appear in list
     await expect(page.locator('.recipe-card')).toHaveCount(3);
   });
   ```

3. **Accessibility check**:
   - Modal keyboard navigation (Tab, Escape)
   - Screen reader labels for file input
   - Focus management (return focus to trigger after close)

---

## Testing Approach

### Unit Tests (`crates/recipe/tests/batch_import_tests.rs`)

**Test Coverage:**
1. ✅ Valid batch import (all recipes succeed)
2. ✅ Empty array rejection
3. ✅ Free tier limit enforcement
4. ✅ Partial success (some recipes fail validation)
5. ✅ Invalid recipe_type rejection
6. ✅ Missing required fields rejection
7. ✅ Duplicate titles allowed (no uniqueness constraint)

**Test Strategy:**
- Use in-memory evento executor
- Mock user tier queries
- Verify events emitted correctly
- Assert success/failure counts

**Example Test:**
```rust
#[tokio::test]
async fn test_batch_import_enforces_free_tier_limit() {
    let executor = MockExecutor::new();

    // Create user with 8 existing recipes (free tier)
    let user_id = create_test_user_with_recipes(&executor, 8, "free").await;

    // Attempt to import 3 more (would be 11 total, exceeds limit of 10)
    let cmd = BatchImportRecipesCommand {
        user_id: user_id.clone(),
        recipes: vec![
            valid_test_recipe("Recipe 1"),
            valid_test_recipe("Recipe 2"),
            valid_test_recipe("Recipe 3"),
        ],
    };

    let result = batch_import_recipes(cmd, &executor).await;

    assert!(matches!(result, Err(RecipeError::RecipeLimitExceeded { .. })));
}
```

### Integration Tests (`tests/batch_import_integration_tests.rs`)

**Test Coverage:**
1. ✅ POST /recipes/import with valid JSON file
2. ✅ POST /recipes/import with invalid JSON syntax
3. ✅ POST /recipes/import exceeding free tier limit
4. ✅ Multipart form parsing
5. ✅ Response HTML contains success/failure counts
6. ✅ Authentication required (401 if not logged in)

**Example Test:**
```rust
#[tokio::test]
async fn test_batch_import_endpoint_success() {
    let app = spawn_test_app().await;
    let auth_cookie = app.login_test_user("test@example.com").await;

    let json_content = r#"[
        {
            "title": "Test Recipe 1",
            "recipe_type": "main_course",
            "ingredients": [{"name": "pasta", "quantity": 200, "unit": "g"}],
            "instructions": [{"step_number": 1, "instruction_text": "Cook pasta", "timer_minutes": null}]
        }
    ]"#;

    let form = multipart::Form::new()
        .text("recipes_file", json_content);

    let response = app.client
        .post(&format!("{}/recipes/import", app.address))
        .header("Cookie", auth_cookie)
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert!(body.contains("1 recipes imported successfully"));
}
```

### E2E Tests (Playwright)

**Test Coverage:**
1. ✅ Full user flow: Click import → upload file → see results
2. ✅ File validation (reject non-JSON files on client-side)
3. ✅ Modal keyboard navigation (Escape to close)
4. ✅ Recipes appear in list after import
5. ✅ Free tier limit error message displayed

**Test Priority:** Medium (integration tests provide adequate coverage)

---

## Deployment Strategy

### Pre-Deployment Checklist
- [ ] All unit tests pass (`cargo test -p recipe batch_import`)
- [ ] All integration tests pass (`cargo test batch_import_integration`)
- [ ] E2E test passes (`npx playwright test batch-import`)
- [ ] Manual testing completed (valid, invalid, limit scenarios)
- [ ] Code review completed (if team workflow)
- [ ] Accessibility audit passed (keyboard navigation, screen reader)

### Deployment Steps

**1. Database Migrations**
- **None required** (uses existing recipe tables, no schema changes)

**2. Feature Flag (Optional)**
```rust
// In config.yaml or environment variable
batch_import_enabled: true
```

**3. Deployment Process**
```bash
# 1. Run tests
cargo test --all-features

# 2. Build release
cargo build --release

# 3. Deploy via Docker/K8s (existing CI/CD pipeline)
docker build -t imkitchen:batch-import .
kubectl apply -f k8s/deployment.yaml

# 4. Verify deployment
curl https://imkitchen.app/recipes
# Check "Import Recipes" button present
```

### Rollback Plan
**If issues occur:**
1. **UI issue**: Hide "Import Recipes" button via feature flag
2. **Backend issue**: Disable route `/recipes/import` (503 Service Unavailable)
3. **Critical bug**: Rollback Docker image to previous version

**Rollback command:**
```bash
kubectl rollout undo deployment/imkitchen
```

### Monitoring
**Metrics to track (OpenTelemetry):**
- `batch_import_requests_total` (counter)
- `batch_import_success_count` (histogram)
- `batch_import_failure_count` (histogram)
- `batch_import_duration_seconds` (histogram)

**Alerts:**
- Alert if failure rate > 50% (indicates schema mismatch or bug)
- Alert if import duration > 10 seconds (indicates performance issue)

---

_This tech spec follows the BMad Method v6 for Level 1 feature additions. It provides definitive technical decisions for implementation with TDD enforcement._
