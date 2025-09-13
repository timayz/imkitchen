# Testing Strategy

## Testing Pyramid

```text
    E2E Tests (Playwright)
    /                    \
   Integration Tests      \
  /    (Backend API)       \
Frontend Unit Tests    Backend Unit Tests
(Rust + WASM)         (Rust + tokio-test)
```

## Test Organization

### Frontend Tests

```text
apps/frontend/tests/
├── unit/                     # Component and utility tests
│   ├── components/          # Template component tests
│   ├── services/            # Service layer tests
│   └── utils/               # Utility function tests
├── integration/             # Frontend integration tests
│   ├── api-client/         # API client tests
│   └── state-management/   # State management tests
└── e2e/                    # End-to-end browser tests
    ├── auth/               # Authentication flows
    ├── recipes/            # Recipe management flows
    ├── cooking/            # Cook mode functionality
    └── offline/            # Offline functionality tests
```

### Backend Tests

```text
apps/backend/tests/
├── unit/                   # Individual function tests
│   ├── services/          # Service layer tests
│   ├── repositories/      # Data access tests
│   └── utils/             # Utility function tests
├── integration/           # API endpoint tests
│   ├── auth/             # Authentication endpoints
│   ├── recipes/          # Recipe CRUD operations
│   └── cooking/          # Cooking session APIs
└── load/                 # Performance and load tests
    ├── recipe-parsing/   # Parser performance tests
    └── concurrent-users/ # Multi-user load tests
```

## Test Examples

### Frontend Component Test

```rust
use wasm_bindgen_test::*;
use web_sys::window;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_recipe_card_rendering() {
    let recipe = Recipe {
        id: "test-id".to_string(),
        title: "Test Recipe".to_string(),
        prep_time: 15,
        cook_time: 30,
        difficulty: SkillLevel::Beginner,
        ..Default::default()
    };
    
    let card = RecipeCard::new(recipe, &default_user_context());
    let rendered = card.render().unwrap();
    
    assert!(rendered.contains("Test Recipe"));
    assert!(rendered.contains("45 min")); // total time
    assert!(rendered.contains("Beginner"));
}
```

### Backend API Test

```rust
use axum_test::TestServer;
use sqlx::PgPool;

#[tokio::test]
async fn test_create_recipe() {
    let pool = setup_test_database().await;
    let app = create_app(pool).await;
    let server = TestServer::new(app).unwrap();
    
    let recipe_request = json!({
        "title": "Test Recipe",
        "ingredients": [{"name": "flour", "quantity": 2, "unit": "cups"}],
        "instructions": [{"step_number": 1, "description": "Mix ingredients"}],
        "prep_time": 15,
        "cook_time": 30
    });
    
    let response = server
        .post("/api/v1/recipes")
        .authorization_bearer("valid-jwt-token")
        .json(&recipe_request)
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    let recipe: Recipe = response.json();
    assert_eq!(recipe.title, "Test Recipe");
    assert_eq!(recipe.total_time, 45);
}
```

### E2E Test

```typescript
import { test, expect } from '@playwright/test';

test('complete cooking workflow', async ({ page }) => {
  // Login
  await page.goto('/auth/login');
  await page.fill('#email', 'test@example.com');
  await page.fill('#password', 'password123');
  await page.click('button[type="submit"]');

  // Navigate to recipe
  await page.goto('/recipes');
  await page.click('[data-testid="recipe-card"]:first-child');
  
  // Start cooking session
  await page.click('[data-testid="start-cooking"]');
  await expect(page).toHaveURL(/\/cook\/session\//);
  
  // Verify timer functionality
  await page.click('[data-testid="start-timer"]');
  await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();
  
  // Complete cooking session
  await page.click('[data-testid="complete-cooking"]');
  await page.fill('[data-testid="rating-input"]', '5');
  await page.click('[data-testid="submit-rating"]');
  
  await expect(page).toHaveURL('/dashboard');
  await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
});
```
