# Coding Standards and Best Practices

Comprehensive coding standards for IMKitchen development, covering Rust best practices, crate organization, and project-specific conventions.

## Table of Contents

- [Critical Rules](#critical-rules)
- [Crate Organization](#crate-organization)
- [Naming Conventions](#naming-conventions)
- [Code Style Guidelines](#code-style-guidelines)
- [Domain-Driven Design Patterns](#domain-driven-design-patterns)
- [Askama Template Standards](#askama-template-standards)
- [Testing Standards](#testing-standards)
- [Error Handling Patterns](#error-handling-patterns)
- [Security Guidelines](#security-guidelines)

## Critical Rules

These rules are non-negotiable and must be followed in all code:

### 1. Type Safety Across Boundaries
```rust
// ✅ CORRECT: Use shared types between crates
use imkitchen_shared::types::{UserId, Email, RecipeId};

pub struct CreateRecipeCommand {
    pub user_id: UserId,           // Strong typing
    pub title: String,
    pub ingredients: Vec<Ingredient>,
}

// ❌ INCORRECT: Using raw strings for IDs
pub struct CreateRecipeCommand {
    pub user_id: String,           // Weak typing - error prone
    pub title: String,
}
```

### 2. Event Sourcing Consistency
```rust
// ✅ CORRECT: Use Evento create() builder pattern
let recipe = Recipe::create(CreateRecipeCommand {
    title: validated_title,
    user_id: user.id,
    ingredients: validated_ingredients,
})?;

// ❌ INCORRECT: Direct mutation
let mut recipe = Recipe::new();
recipe.title = title;  // Bypasses domain validation
```

### 3. Template Security
```rust
// ✅ CORRECT: Use Askama filters for user input
<h1>{{ recipe.title|e }}</h1>
<p>{{ user.bio|e|nl2br }}</p>

// ❌ INCORRECT: Raw HTML injection
<h1>{{ recipe.title }}</h1>  // XSS vulnerability
```

### 4. TwinSpark Patterns
```html
<!-- ✅ CORRECT: Use ts-* attributes -->
<form ts-req="/recipes" ts-target="#result" ts-swap="innerHTML">
  <input name="title" required>
  <button type="submit">Create</button>
</form>

<!-- ❌ INCORRECT: Custom JavaScript -->
<form onsubmit="submitRecipe(); return false;">
  <!-- Custom JS not allowed -->
</form>
```

### 5. Error Handling
```rust
// ✅ CORRECT: All handlers return Result<T, AppError>
pub async fn create_recipe_handler(
    State(app_state): State<AppState>,
    user: UserSession,
    Form(request): Form<CreateRecipeRequest>,
) -> Result<Html<String>, AppError> {
    let command = CreateRecipeCommand::try_from(request)?;
    let recipe = app_state.recipe_service.create_recipe(command).await?;
    Ok(Html(render_recipe_created(&recipe)?))
}

// ❌ INCORRECT: Unwrapping or unhandled errors
pub async fn create_recipe_handler(/* ... */) -> Html<String> {
    let recipe = service.create_recipe(command).unwrap(); // Panic risk
    Html(template.render().unwrap())  // Panic risk
}
```

### 6. Validation First
```rust
// ✅ CORRECT: Validate before business logic
#[derive(Debug, Validate, Deserialize)]
pub struct CreateRecipeRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(range(min = 1, max = 480))]
    pub prep_time_minutes: u16,
    
    #[validate(length(min = 1))]
    pub ingredients: Vec<IngredientRequest>,
}

pub async fn create_recipe_handler(
    Form(request): Form<CreateRecipeRequest>,
) -> Result<Html<String>, AppError> {
    request.validate()?;  // Validate first
    // ... business logic
}
```

### 7. Crate Boundaries
```rust
// ✅ CORRECT: Domain crates only depend on shared types
// In imkitchen-recipe/Cargo.toml
[dependencies]
imkitchen-shared = { path = "../imkitchen-shared" }
# No dependency on web crate

// ❌ INCORRECT: Domain depending on web
[dependencies]
imkitchen-web = { path = "../imkitchen-web" }  // Violates boundaries
```

## Crate Organization

### Bounded Context Structure

```
crates/
├── imkitchen-shared/          # Common types and utilities
│   ├── src/
│   │   ├── types/             # Shared value objects
│   │   ├── events/            # Domain event definitions
│   │   ├── errors/            # Common error types
│   │   └── utils/             # Utility functions
│   └── Cargo.toml
│
├── imkitchen-user/            # User bounded context
│   ├── src/
│   │   ├── domain/            # User domain logic
│   │   │   ├── user.rs        # User aggregate
│   │   │   ├── profile.rs     # Profile value object
│   │   │   └── events.rs      # User domain events
│   │   ├── commands/          # CQRS commands
│   │   ├── queries/           # CQRS queries
│   │   ├── projections/       # Read model projections
│   │   └── lib.rs
│   ├── tests/
│   │   ├── integration/       # Integration tests
│   │   └── unit/              # Unit tests
│   └── Cargo.toml
│
├── imkitchen-recipe/          # Recipe bounded context
├── imkitchen-meal-planning/   # Meal planning bounded context
├── imkitchen-shopping/        # Shopping bounded context
├── imkitchen-notification/    # Notification bounded context
│
└── imkitchen-web/             # Web presentation layer
    ├── src/
    │   ├── handlers/          # Axum request handlers
    │   │   ├── auth.rs        # Authentication handlers
    │   │   ├── recipes.rs     # Recipe handlers
    │   │   └── meal_plans.rs  # Meal planning handlers
    │   ├── middleware/        # HTTP middleware
    │   ├── templates/         # Template data structures
    │   └── lib.rs
    ├── templates/             # Askama template files
    │   ├── layout/            # Base layouts
    │   ├── auth/              # Authentication templates
    │   ├── recipes/           # Recipe templates
    │   └── meal_plans/        # Meal planning templates
    ├── static/                # Static assets
    │   ├── css/               # Tailwind CSS
    │   ├── js/                # TwinSpark and minimal JS
    │   └── images/            # UI images
    └── Cargo.toml
```

### Domain Crate Structure

Each domain crate follows consistent internal organization:

```rust
// src/lib.rs - Public API
pub mod domain;
pub mod commands;
pub mod queries;
pub mod projections;

pub use domain::*;
pub use commands::*;
pub use queries::*;

// src/domain/mod.rs - Domain layer
pub mod user;
pub mod profile;
pub mod events;

pub use user::*;
pub use profile::*;
pub use events::*;

// src/domain/user.rs - Aggregate root
use imkitchen_shared::types::{UserId, Email};

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    // ... other fields
}

impl User {
    pub fn create(command: CreateUserCommand) -> Result<Self, UserError> {
        // Domain validation and creation logic
    }
    
    pub fn update_profile(&mut self, command: UpdateProfileCommand) -> Result<ProfileUpdated, UserError> {
        // Domain business logic
    }
}
```

## Naming Conventions

### File and Directory Names

| Type | Convention | Example |
|------|------------|---------|
| **Crate Names** | `kebab-case` | `imkitchen-meal-planning` |
| **File Names** | `snake_case.rs` | `meal_plan_service.rs` |
| **Directory Names** | `snake_case` | `meal_planning/` |
| **Template Files** | `snake_case.html` | `weekly_calendar.html` |
| **Static Assets** | `kebab-case` | `recipe-card.css` |

### Rust Code Naming

| Element | Convention | Example |
|---------|------------|---------|
| **Structs** | `PascalCase` | `CreateRecipeCommand` |
| **Enums** | `PascalCase` | `CookingDifficulty` |
| **Functions** | `snake_case` | `create_meal_plan` |
| **Variables** | `snake_case` | `prep_time_minutes` |
| **Constants** | `SCREAMING_SNAKE_CASE` | `MAX_RECIPE_TITLE_LENGTH` |
| **Modules** | `snake_case` | `meal_planning` |

### Domain-Specific Naming

| Type | Convention | Example |
|------|------------|---------|
| **Aggregates** | Singular noun | `Recipe`, `User`, `MealPlan` |
| **Value Objects** | Descriptive noun | `Email`, `RecipeTitle`, `CookingTime` |
| **Commands** | Verb + noun + Command | `CreateRecipeCommand` |
| **Events** | Past tense + Event | `RecipeCreatedEvent` |
| **Queries** | Noun + By + Criteria + Query | `RecipesByCuisineQuery` |
| **Handlers** | Action + Handler | `create_recipe_handler` |

### Template Naming

| Type | Convention | Example |
|------|------------|---------|
| **Pages** | `entity_action.html` | `recipe_create.html` |
| **Components** | `component_name.html` | `recipe_card.html` |
| **Layouts** | `layout_type.html` | `base_layout.html` |
| **Partials** | `_partial_name.html` | `_navigation.html` |

## Code Style Guidelines

### Rust Code Formatting

Use `cargo fmt` with default settings. Key formatting rules:

```rust
// ✅ CORRECT: Function definition
pub async fn create_recipe_handler(
    State(app_state): State<AppState>,
    user: UserSession,
    Form(request): Form<CreateRecipeRequest>,
) -> Result<Html<String>, AppError> {
    // Function body
}

// ✅ CORRECT: Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub title: String,
    pub description: Option<String>,
    pub prep_time: CookingTime,
    pub ingredients: Vec<Ingredient>,
}

// ✅ CORRECT: Error handling
let recipe = recipe_service
    .create_recipe(command)
    .await
    .map_err(|e| AppError::ServiceError(e.to_string()))?;

// ✅ CORRECT: Match patterns
match difficulty {
    CookingDifficulty::Easy => "This recipe is perfect for beginners",
    CookingDifficulty::Medium => "Some cooking experience helpful",
    CookingDifficulty::Hard => "Advanced cooking skills required",
}
```

### Import Organization

```rust
// Standard library imports first
use std::collections::HashMap;
use std::time::Duration;

// External crate imports
use axum::{extract::State, response::Html, Form};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn, error};
use validator::Validate;

// Internal crate imports (current crate)
use crate::domain::{Recipe, RecipeError};
use crate::commands::CreateRecipeCommand;

// Shared crate imports
use imkitchen_shared::types::{UserId, RecipeId};
use imkitchen_shared::events::RecipeCreated;
```

### Documentation Standards

```rust
/// Creates a new recipe for the authenticated user.
/// 
/// This handler validates the recipe data, creates the recipe through the
/// domain service, and returns either the created recipe view or validation errors.
/// 
/// # Arguments
/// 
/// * `app_state` - Application state containing services and database connection
/// * `user` - Authenticated user session (automatically extracted)
/// * `request` - Form data containing recipe details
/// 
/// # Returns
/// 
/// * `Ok(Html<String>)` - Recipe created successfully, returns recipe view
/// * `Err(AppError)` - Validation errors or service failures
/// 
/// # Examples
/// 
/// ```
/// // Called automatically by Axum when POST /recipes is requested
/// // Form data: title=Pasta&prep_time=15&ingredients[0][name]=Spaghetti
/// ```
pub async fn create_recipe_handler(
    State(app_state): State<AppState>,
    user: UserSession,
    Form(request): Form<CreateRecipeRequest>,
) -> Result<Html<String>, AppError> {
    // Implementation
}
```

## Domain-Driven Design Patterns

### Aggregate Patterns

```rust
// ✅ CORRECT: Aggregate with clear boundaries
#[derive(Debug, Clone)]
pub struct Recipe {
    // Aggregate ID
    pub id: RecipeId,
    
    // Aggregate data
    pub title: RecipeTitle,
    pub description: Option<String>,
    pub user_id: UserId,
    
    // Value objects
    pub prep_time: CookingTime,
    pub difficulty: CookingDifficulty,
    
    // Entity collections (within boundary)
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    
    // Aggregate metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Recipe {
    /// Create new recipe with validation
    pub fn create(command: CreateRecipeCommand) -> Result<(Self, Vec<DomainEvent>), RecipeError> {
        // Validate command
        command.validate()?;
        
        // Create recipe
        let recipe = Self {
            id: RecipeId::new(),
            title: command.title,
            // ... other fields
        };
        
        // Create domain events
        let events = vec![
            DomainEvent::RecipeCreated(RecipeCreated {
                recipe_id: recipe.id.clone(),
                user_id: recipe.user_id.clone(),
                title: recipe.title.clone(),
                occurred_at: Utc::now(),
            })
        ];
        
        Ok((recipe, events))
    }
    
    /// Update recipe with business rules
    pub fn update(&mut self, command: UpdateRecipeCommand) -> Result<Vec<DomainEvent>, RecipeError> {
        // Business validation
        self.validate_update(&command)?;
        
        // Apply changes
        self.title = command.title;
        self.updated_at = Utc::now();
        
        // Generate events
        Ok(vec![
            DomainEvent::RecipeUpdated(RecipeUpdated {
                recipe_id: self.id.clone(),
                // ... event data
            })
        ])
    }
}
```

### Value Object Patterns

```rust
// ✅ CORRECT: Value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        // Validate email format
        if !validator::validate_email(&value) {
            return Err(ValidationError::InvalidEmail);
        }
        
        // Normalize email (lowercase)
        Ok(Self(value.to_lowercase()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Email {
    type Err = ValidationError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

// ✅ CORRECT: Cooking time value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CookingTime {
    minutes: u16,
}

impl CookingTime {
    pub fn new(minutes: u16) -> Result<Self, ValidationError> {
        if minutes == 0 || minutes > 480 {  // Max 8 hours
            return Err(ValidationError::InvalidCookingTime);
        }
        Ok(Self { minutes })
    }
    
    pub fn minutes(&self) -> u16 {
        self.minutes
    }
    
    pub fn hours_and_minutes(&self) -> (u16, u16) {
        (self.minutes / 60, self.minutes % 60)
    }
}
```

### Command and Query Patterns

```rust
// ✅ CORRECT: Command with validation
#[derive(Debug, Validate, Deserialize)]
pub struct CreateRecipeCommand {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    
    #[validate(range(min = 1, max = 480))]
    pub prep_time_minutes: u16,
    
    #[validate(length(min = 1))]
    pub ingredients: Vec<CreateIngredientCommand>,
    
    pub user_id: UserId,  // Set by handler
}

// ✅ CORRECT: Query with criteria
#[derive(Debug, Clone)]
pub struct FindRecipesByCriteriaQuery {
    pub user_id: Option<UserId>,
    pub cuisine_type: Option<String>,
    pub difficulty: Option<CookingDifficulty>,
    pub max_prep_time: Option<u16>,
    pub search_text: Option<String>,
    pub page: u32,
    pub limit: u32,
}

impl Default for FindRecipesByCriteriaQuery {
    fn default() -> Self {
        Self {
            user_id: None,
            cuisine_type: None,
            difficulty: None,
            max_prep_time: None,
            search_text: None,
            page: 1,
            limit: 20,
        }
    }
}
```

## Askama Template Standards

### Template Organization

```
templates/
├── layout/
│   ├── base.html              # Main layout
│   ├── auth.html              # Authentication layout
│   └── modal.html             # Modal layout
├── components/
│   ├── _navigation.html       # Navigation component
│   ├── _recipe_card.html      # Recipe card component
│   └── _pagination.html       # Pagination component
├── auth/
│   ├── login.html             # Login page
│   └── register.html          # Registration page
├── recipes/
│   ├── list.html              # Recipe list
│   ├── detail.html            # Recipe detail
│   └── create.html            # Recipe creation form
└── errors/
    ├── 404.html               # Not found
    └── 500.html               # Server error
```

### Template Security Standards

```html
<!-- ✅ CORRECT: Always escape user input -->
<h1>{{ recipe.title|e }}</h1>
<p>{{ recipe.description|e|nl2br }}</p>
<div class="author">By {{ recipe.author.name|e }}</div>

<!-- ✅ CORRECT: Safe HTML for trusted content -->
<div class="instructions">
  {{ recipe.instructions_html|safe }}  <!-- Only if HTML is sanitized -->
</div>

<!-- ✅ CORRECT: URL encoding for links -->
<a href="/recipes/search?q={{ search_query|urlencode }}">Search</a>

<!-- ❌ INCORRECT: Raw user input -->
<h1>{{ recipe.title }}</h1>  <!-- XSS vulnerability -->
<script>var title = "{{ recipe.title }}";</script>  <!-- Script injection -->
```

### Template Data Structures

```rust
// ✅ CORRECT: Template data with proper types
#[derive(Debug, Template)]
#[template(path = "recipes/detail.html")]
pub struct RecipeDetailTemplate {
    pub recipe: Recipe,
    pub current_user: Option<UserSession>,
    pub csrf_token: String,
    pub page_title: String,
    pub can_edit: bool,
}

impl RecipeDetailTemplate {
    pub fn new(
        recipe: Recipe,
        current_user: Option<UserSession>,
        csrf_token: String,
    ) -> Self {
        let can_edit = current_user
            .as_ref()
            .map(|user| user.user_id == recipe.user_id)
            .unwrap_or(false);
            
        Self {
            page_title: format!("Recipe: {}", recipe.title),
            recipe,
            current_user,
            csrf_token,
            can_edit,
        }
    }
}
```

### TwinSpark Integration Patterns

```html
<!-- ✅ CORRECT: Form with TwinSpark -->
<form 
  class="recipe-form" 
  ts-req="/recipes" 
  ts-target="#recipe-result"
  ts-swap="innerHTML">
  
  <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
  
  <div class="form-field">
    <label for="title">Recipe Title</label>
    <input 
      id="title" 
      name="title" 
      required
      ts-req="/validate/recipe-title"
      ts-trigger="blur"
      ts-target="#title-validation">
    <div id="title-validation" class="validation-feedback"></div>
  </div>
  
  <button type="submit" class="primary-button">Create Recipe</button>
</form>

<div id="recipe-result"></div>
```

## Testing Standards

### Test Organization

```rust
// tests/unit/domain/recipe_tests.rs
use crate::domain::{Recipe, CreateRecipeCommand};

#[test]
fn test_create_recipe_with_valid_data() {
    // Arrange
    let command = CreateRecipeCommand {
        title: "Pasta Carbonara".to_string(),
        prep_time_minutes: 15,
        // ... other fields
    };
    
    // Act
    let result = Recipe::create(command);
    
    // Assert
    assert!(result.is_ok());
    let (recipe, events) = result.unwrap();
    assert_eq!(recipe.title, "Pasta Carbonara");
    assert_eq!(events.len(), 1);
}

// tests/integration/recipe_handlers_tests.rs
use axum_test_helper::TestClient;

#[tokio::test]
async fn test_create_recipe_handler() {
    // Arrange
    let app = create_test_app().await;
    let client = TestClient::new(app);
    
    // Act
    let response = client
        .post("/recipes")
        .form(&[
            ("title", "Test Recipe"),
            ("prep_time_minutes", "30"),
        ])
        .send()
        .await;
    
    // Assert
    assert_eq!(response.status(), 201);
    assert!(response.text().await.contains("Recipe created"));
}
```

### Test Data Builders

```rust
// ✅ CORRECT: Builder pattern for test data
pub struct RecipeTestBuilder {
    title: String,
    prep_time: u16,
    user_id: UserId,
    ingredients: Vec<Ingredient>,
}

impl RecipeTestBuilder {
    pub fn new() -> Self {
        Self {
            title: "Test Recipe".to_string(),
            prep_time: 30,
            user_id: UserId::new(),
            ingredients: vec![],
        }
    }
    
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    
    pub fn with_prep_time(mut self, minutes: u16) -> Self {
        self.prep_time = minutes;
        self
    }
    
    pub fn build(self) -> CreateRecipeCommand {
        CreateRecipeCommand {
            title: self.title,
            prep_time_minutes: self.prep_time,
            user_id: self.user_id,
            ingredients: self.ingredients,
            // ... other fields with defaults
        }
    }
}

// Usage in tests
#[test]
fn test_recipe_creation() {
    let command = RecipeTestBuilder::new()
        .with_title("Spaghetti Carbonara")
        .with_prep_time(15)
        .build();
        
    let result = Recipe::create(command);
    assert!(result.is_ok());
}
```

## Error Handling Patterns

### Error Type Hierarchy

```rust
// ✅ CORRECT: Structured error hierarchy
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    
    #[error("Authentication required")]
    Unauthorized,
    
    #[error("Access denied: {reason}")]
    Forbidden { reason: String },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Template error: {0}")]
    TemplateError(#[from] askama::Error),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

// Error conversion for different contexts
impl From<RecipeError> for AppError {
    fn from(err: RecipeError) -> Self {
        match err {
            RecipeError::ValidationError(msg) => AppError::ValidationError(ValidationError::Custom(msg)),
            RecipeError::NotFound(id) => AppError::NotFound { 
                resource: format!("Recipe {}", id) 
            },
            RecipeError::PermissionDenied => AppError::Forbidden { 
                reason: "Cannot modify recipe".to_string() 
            },
        }
    }
}
```

## Security Guidelines

### Input Validation

```rust
// ✅ CORRECT: Comprehensive validation
#[derive(Debug, Validate, Deserialize)]
pub struct CreateRecipeRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: String,
    
    #[validate(length(max = 2000, message = "Description too long"))]
    pub description: Option<String>,
    
    #[validate(range(min = 1, max = 480, message = "Prep time must be 1-480 minutes"))]
    pub prep_time_minutes: u16,
    
    #[validate(length(min = 1, message = "At least one ingredient required"))]
    #[validate]
    pub ingredients: Vec<IngredientRequest>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct IngredientRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(range(min = 0.01, max = 10000.0))]
    pub quantity: f64,
    
    #[validate(length(min = 1, max = 20))]
    pub unit: String,
}
```

### CSRF Protection

```rust
// ✅ CORRECT: CSRF token validation
pub async fn create_recipe_handler(
    State(app_state): State<AppState>,
    user: UserSession,
    Form(request): Form<CreateRecipeRequest>,
) -> Result<Html<String>, AppError> {
    // CSRF token is validated by middleware
    // User session is validated by middleware
    
    let command = CreateRecipeCommand {
        title: request.title,
        user_id: user.user_id,  // Always use authenticated user
        // ... other fields
    };
    
    // ... rest of handler
}
```

### SQL Injection Prevention

```rust
// ✅ CORRECT: Use SQLx query macros
let recipes = sqlx::query_as!(
    Recipe,
    r#"
    SELECT id, title, description, prep_time_minutes, user_id, created_at
    FROM recipes 
    WHERE user_id = ? AND cuisine_type = ?
    ORDER BY created_at DESC
    LIMIT ?
    "#,
    user_id,
    cuisine_type,
    limit
)
.fetch_all(&pool)
.await?;

// ❌ INCORRECT: String concatenation
let query = format!(
    "SELECT * FROM recipes WHERE title = '{}'", 
    title  // SQL injection vulnerability
);
```

For more coding standards:
- [Testing Guide](testing.md)
- [Security Best Practices](security.md)
- [Performance Guidelines](performance.md)
- [Code Review Checklist](code-review.md)