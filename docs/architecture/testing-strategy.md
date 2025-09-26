# Testing Strategy

## Testing Pyramid
```
    E2E Tests (Playwright)
         /              \
   Integration Tests (Rust)
     /                      \
Frontend Unit Tests      Backend Unit Tests
  (Template Tests)         (Domain Tests)
```

## Test Organization

### Frontend Tests
```
crates/imkitchen-web/tests/
├── integration/
│   ├── template_rendering.rs  # Askama template tests
│   ├── twinsparkl_integration.rs # TwinSpark endpoint tests
│   └── auth_flows.rs          # Authentication integration
├── unit/
│   ├── handlers.rs            # Handler unit tests
│   └── middleware.rs          # Middleware tests
└── common/
    └── test_helpers.rs        # Shared test utilities
```

### Backend Tests
```
crates/imkitchen-meal-planning/tests/
├── integration/
│   ├── meal_plan_generation.rs # Full workflow tests
│   └── evento_integration.rs   # Event sourcing tests
├── unit/
│   ├── algorithms.rs          # Meal planning algorithm tests
│   ├── domain_models.rs       # Domain object tests
│   └── commands.rs            # CQRS command tests
└── fixtures/
    └── test_data.rs           # Test recipe and user data
```

### E2E Tests
```
e2e/tests/
├── meal_planning_flow.spec.ts # Complete meal planning journey
├── recipe_management.spec.ts  # Recipe CRUD operations
├── shopping_list.spec.ts      # Shopping list generation
└── auth_and_profile.spec.ts   # User management flows
```

## Test Examples

### Frontend Component Test
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;
    use crate::domain::recipe::Recipe;

    #[test]
    fn test_recipe_card_template_rendering() {
        let recipe = Recipe::builder()
            .title("Test Recipe")
            .prep_time_minutes(15)
            .difficulty(Difficulty::Easy)
            .build();
            
        let template = RecipeCardTemplate::new(recipe, Some("user123"));
        let rendered = template.render().unwrap();
        
        assert!(rendered.contains("Test Recipe"));
        assert!(rendered.contains("15 min"));
        assert!(rendered.contains("Easy"));
    }
}
```

### Backend API Test
```rust
#[tokio::test]
async fn test_generate_meal_plan_handler() {
    let app_state = create_test_app_state().await;
    let user = create_test_user();
    
    let request = GenerateMealPlanRequest {
        preferences: MealPlanPreferences::default(),
    };
    
    let response = generate_meal_plan_handler(
        State(app_state),
        user,
        Form(request),
    ).await.unwrap();
    
    assert!(response.0.contains("weekly-calendar"));
    assert!(response.0.contains("Monday"));
}
```

### E2E Test
```typescript
import { test, expect } from '@playwright/test';

test('complete meal planning workflow', async ({ page }) => {
  // Login
  await page.goto('/auth/login');
  await page.fill('[name="email"]', 'test@example.com');
  await page.fill('[name="password"]', 'password123');
  await page.click('button[type="submit"]');
  
  // Generate meal plan
  await page.click('#fill-my-week-button');
  await expect(page.locator('#weekly-calendar')).toBeVisible();
  
  // Verify meal slots are populated
  await expect(page.locator('.meal-slot')).toHaveCount(21);
  
  // Generate shopping list
  await page.click('#generate-shopping-list');
  await expect(page.locator('#shopping-list')).toBeVisible();
});
```
