# Frontend Architecture

## Component Architecture

### Component Organization
```
templates/
├── layouts/
│   ├── base.html              # PWA shell with meta tags
│   └── authenticated.html     # Layout for logged-in users
├── pages/
│   ├── dashboard.html         # Weekly meal calendar
│   ├── recipe_discovery.html  # Recipe browsing and search
│   ├── collections.html       # Personal recipe collections
│   └── shopping_list.html     # Shopping list management
├── components/
│   ├── meal_slot.html         # Individual calendar meal slot
│   ├── recipe_card.html       # Recipe display card
│   ├── navigation.html        # Bottom navigation bar
│   └── loading_states.html    # Skeleton loading screens
└── fragments/
    ├── calendar_week.html     # Weekly calendar updates
    ├── search_results.html    # Live search results
    └── shopping_items.html    # Shopping list updates
```

### Component Template
```rust
use askama::Template;
use crate::domain::recipe::Recipe;

#[derive(Template)]
#[template(path = "components/recipe_card.html")]
pub struct RecipeCardTemplate {
    pub recipe: Recipe,
    pub is_favorited: bool,
    pub user_can_edit: bool,
}

impl RecipeCardTemplate {
    pub fn new(recipe: Recipe, user_id: Option<&str>) -> Self {
        Self {
            is_favorited: recipe.is_favorited_by(user_id),
            user_can_edit: recipe.created_by == user_id.unwrap_or(""),
            recipe,
        }
    }
}
```

## State Management Architecture

### State Structure
```rust
// Server-side state management through Evento projections
pub struct UserSessionState {
    pub user_id: Uuid,
    pub preferences: UserPreferences,
    pub active_meal_plan_id: Option<Uuid>,
    pub current_collections: Vec<RecipeCollection>,
}

pub struct ApplicationState {
    pub user_sessions: HashMap<String, UserSessionState>,
    pub recipe_cache: HashMap<Uuid, Recipe>,
    pub meal_plan_cache: HashMap<Uuid, MealPlan>,
}
```

### State Management Patterns
- **Server-side session storage** - User state maintained on server across requests
- **Evento projection caching** - Fast read access through materialized views
- **TwinSpark fragment updates** - Partial page updates without full refresh
- **Progressive enhancement** - Core functionality works without JavaScript

## Routing Architecture

### Route Organization
```
/                          # Dashboard with weekly calendar
/recipes/discover          # Recipe browsing and search
/recipes/{id}              # Recipe detail view
/collections               # Personal recipe collections
/collections/{id}          # Specific collection view
/meal-plans/current        # Current weekly meal plan
/shopping-lists/current    # Current shopping list
/profile                   # User profile and preferences
/auth/login                # Authentication pages
/auth/register             # User registration
```

### Protected Route Pattern
```rust
use axum::{middleware, Router};
use crate::auth::AuthMiddleware;

pub fn create_app_routes() -> Router {
    Router::new()
        .route("/", get(dashboard_handler))
        .route("/recipes/discover", get(recipe_discovery_handler))
        .route("/meal-plans/generate", post(generate_meal_plan_handler))
        .layer(middleware::from_fn(AuthMiddleware::require_auth))
        .route("/auth/login", get(login_page).post(login_handler))
        .route("/auth/register", get(register_page).post(register_handler))
}
```

## Frontend Services Layer

### API Client Setup
```rust
// TwinSpark integration - no traditional API client needed
use axum::response::Html;
use askama::Template;

pub struct TwinSparkResponse {
    pub target: String,
    pub content: Html<String>,
}

impl TwinSparkResponse {
    pub fn new(template: impl Template, target: &str) -> Self {
        Self {
            target: target.to_string(),
            content: Html(template.render().unwrap()),
        }
    }
}
```

### Service Example
```rust
use crate::domain::meal_planning::MealPlanningService;
use crate::templates::calendar::WeeklyCalendarTemplate;

use askama::Template;
use axum::{extract::State, response::Html, Form};
use evento::Context;

#[derive(Template)]
#[template(path = "weekly_calendar.html")]
pub struct WeeklyCalendarTemplate {
    pub meal_slots: Vec<MealSlot>,
    pub user_preferences: UserPreferences,
}

pub async fn generate_meal_plan_handler(
    State(context): State<Context>,
    user: AuthenticatedUser,
    Form(request): Form<GenerateMealPlanRequest>,
) -> Result<Html<String>, AppError> {
    // Create meal plan event using Evento
    let meal_plan_id = create::<MealPlan>()
        .data(&MealPlanGenerated {
            user_id: user.id,
            preferences: request.preferences.clone(),
            week_start: request.week_start,
        })?
        .commit(&context)
        .await?;

    // Load the updated aggregate to get current state
    let meal_plan = context.load::<MealPlan>(&meal_plan_id).await?;
    
    // Render Askama template - TwinSpark will replace ts-target="#weekly-calendar"
    let template = WeeklyCalendarTemplate {
        meal_slots: meal_plan.meal_slots,
        user_preferences: request.preferences,
    };
    
    Ok(Html(template.render()?))
}
```
