# Unified Application Architecture

## Template and Handler Organization

### Project Structure

```text
src/
├── main.rs                   # Application entry point and server setup
├── routes/                   # Template and fragment route handlers
│   ├── mod.rs
│   ├── pages/                # Full page handlers
│   │   ├── dashboard.rs      # Dashboard page
│   │   ├── recipes.rs        # Recipe management pages
│   │   ├── planner.rs        # Meal planning interface
│   │   ├── cooking.rs        # Cook mode pages
│   │   └── auth.rs           # Authentication pages
│   └── fragments/            # HTML fragment handlers (for twinspark-js)
│       ├── recipe_card.rs    # Recipe card component updates
│       ├── timer_widget.rs   # Timer widget updates
│       ├── meal_list.rs      # Meal list updates
│       └── notifications.rs  # Notification fragments
├── templates/                # Askama templates (embedded in binary)
│   ├── layouts/              # Base layouts (main.html, auth.html)
│   ├── pages/                # Full page templates
│   │   ├── dashboard.html    # Main dashboard
│   │   ├── recipes/          # Recipe management pages
│   │   ├── planner/          # Meal planning interface
│   │   └── cook/             # Cooking mode templates
│   ├── components/           # Reusable template fragments
│   │   ├── recipe_card.html
│   │   ├── timer_widget.html
│   │   └── meal_planner_grid.html
│   └── fragments/            # Small fragments for twinspark-js updates
│       ├── recipe_item.html
│       ├── timer_display.html
│       └── notification.html
├── services/                 # Business logic services
├── repositories/             # Data access layer
├── models/                   # Domain models
├── middleware/               # HTTP middleware
└── config/                   # Configuration
static/                       # Static assets served by axum
├── css/                      # Tailwind compiled CSS
├── js/                       # twinspark-js for DOM updates
├── images/                   # UI images and icons
└── sw.js                     # Service worker
```

### Template Handler Pattern

```rust
use askama::Template;
use axum::{response::Html, Extension, extract::Query};
use crate::{models::Recipe, auth::UserClaims, services::RecipeService};

#[derive(Template)]
#[template(path = "pages/recipes/index.html")]
struct RecipesPage {
    user: UserClaims,
    recipes: Vec<Recipe>,
    current_page: u32,
    total_pages: u32,
}

#[derive(Template)]
#[template(path = "fragments/recipe_list.html")]
struct RecipeListFragment {
    recipes: Vec<Recipe>,
}

// Full page handler
pub async fn recipes_page(
    Extension(user): Extension<UserClaims>,
    Extension(recipe_service): Extension<RecipeService>,
) -> Result<Html<String>, AppError> {
    let recipes = recipe_service.get_user_recipes(user.sub).await?;
    
    let template = RecipesPage {
        user,
        recipes,
        current_page: 1,
        total_pages: 1,
    };
    
    Ok(Html(template.render()?))
}

// Fragment handler for twinspark-js updates
pub async fn recipes_list_fragment(
    Extension(user): Extension<UserClaims>,
    Extension(recipe_service): Extension<RecipeService>,
    Query(params): Query<ListParams>,
) -> Result<Html<String>, AppError> {
    let recipes = recipe_service.get_user_recipes_filtered(user.sub, params).await?;
    
    let template = RecipeListFragment { recipes };
    
    Ok(Html(template.render()?))
}
```

## State Management Architecture

### State Structure

```typescript
// Frontend state managed through Alpine.js stores
interface AppState {
  user: {
    id: string;
    preferences: UserPreferences;
    settings: UserSettings;
  };
  cooking: {
    activeSessions: CookingSession[];
    timers: Timer[];
    currentRecipe?: Recipe;
  };
  planner: {
    currentWeek: MealPlan;
    isGenerating: boolean;
    isDirty: boolean;
  };
  ui: {
    notifications: Notification[];
    modals: ModalState;
    connectivity: 'online' | 'offline';
  };
}
```

### State Management Patterns

- **Server-Side Rendering:** All state managed server-side, DOM updated via HTML fragments
- **Fragment Updates:** twinspark-js requests return HTML fragments that replace DOM sections
- **Form Submissions:** Standard form posts return updated page fragments or full page redirects
- **Real-time Updates:** WebSocket events trigger fragment requests to refresh specific components

## Unified Routing Architecture

### Route Organization

```text
Unified Routes (single axum application):
# Page routes (return full HTML pages)
/                           # Dashboard page
/auth/login                 # Login page  
/auth/register              # Registration page
/recipes                    # Recipe library page
/recipes/new                # Add/import recipe page
/recipes/{id}               # Recipe detail page
/recipes/{id}/cook          # Cook mode page
/planner                    # Meal planning page
/shopping                   # Shopping list page

# Fragment routes (return HTML fragments for twinspark-js)
/fragments/recipes          # Recipe list fragment
/fragments/recipe/{id}      # Recipe card fragment
/fragments/timers           # Timer widgets fragment
/fragments/meal-plan        # Meal plan fragment
/fragments/notifications    # Notification fragment

# Form submission routes (POST/PUT/DELETE)
/recipes                    # Create recipe (POST)
/recipes/{id}               # Update/delete recipe
/cooking/sessions           # Start cooking session
/meal-plans                 # Generate meal plan

# Static assets
/static/*                   # CSS, JS, images served by axum
```

### Unified Route Setup Pattern

```rust
use axum::{middleware, Router, routing::{get, post}};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

pub fn create_app() -> Router {
    Router::new()
        // Public routes
        .route("/", get(pages::dashboard::dashboard_page))
        .route("/auth/login", get(pages::auth::login_page).post(pages::auth::login_submit))
        .route("/auth/register", get(pages::auth::register_page).post(pages::auth::register_submit))
        
        // Protected page routes
        .route("/recipes", get(pages::recipes::recipes_page).post(pages::recipes::create_recipe))
        .route("/recipes/new", get(pages::recipes::new_recipe_page))
        .route("/recipes/:id", get(pages::recipes::recipe_detail_page))
        .route("/planner", get(pages::planner::meal_planner_page))
        .route("/cooking/:id", get(pages::cooking::cooking_session_page))
        
        // Fragment routes for twinspark-js
        .route("/fragments/recipes", get(fragments::recipe_list::recipe_list_fragment))
        .route("/fragments/recipe/:id", get(fragments::recipe_card::recipe_card_fragment))
        .route("/fragments/timers", get(fragments::timer_widget::timers_fragment))
        .route("/fragments/meal-plan", get(fragments::meal_list::meal_plan_fragment))
        .route("/fragments/notifications", get(fragments::notifications::notifications_fragment))
        
        // Form submission routes
        .route("/cooking/sessions", post(pages::cooking::start_cooking_session))
        .route("/meal-plans", post(pages::planner::generate_meal_plan))
        
        // Apply authentication to protected routes
        .layer(ServiceBuilder::new()
            .layer(middleware::from_fn(auth::require_authentication))
            .layer(middleware::from_fn(session::load_user_context))
        )
        
        // Static file serving
        .nest_service("/static", ServeDir::new("static"))
}
```

## Client-Side Interactivity

### twinspark-js Integration

twinspark-js handles DOM updates by making requests to fragment endpoints and updating specific page sections. No custom JavaScript API client needed.

```html
<!-- Example: Recipe card with interactive rating -->
<div class="recipe-card" 
     ts-req="/fragments/recipe/{{ recipe.id }}"
     ts-trigger="click"
     ts-target="#recipe-{{ recipe.id }}">
  <h3>{{ recipe.title }}</h3>
  <p>Prep: {{ recipe.prep_time }} min</p>
  <button class="favorite-btn">♡ Favorite</button>
</div>

<!-- Example: Timer widget that updates automatically -->
<div id="timer-widget" 
     ts-req="/fragments/timers"
     ts-trigger="every 1s"
     ts-target="#timer-widget">
  {% for timer in timers %}
    <div class="timer">{{ timer.name }}: {{ timer.remaining }}s</div>
  {% endfor %}
</div>

<!-- Example: Form submission with fragment update -->
<form ts-req="/recipes" 
      ts-trigger="submit"
      ts-target="#recipe-list">
  <input name="title" placeholder="Recipe name">
  <textarea name="instructions" placeholder="Instructions"></textarea>
  <button type="submit">Add Recipe</button>
</form>
```

### Alpine.js for Local State

```javascript
// Simple Alpine.js store for local UI state
document.addEventListener('alpine:init', () => {
  Alpine.store('ui', {
    notifications: [],
    isOffline: false,
    activeTimers: [],
    
    addNotification(message, type = 'info') {
      this.notifications.push({ message, type, id: Date.now() });
    },
    
    removeNotification(id) {
      this.notifications = this.notifications.filter(n => n.id !== id);
    },
    
    updateConnectivity(online) {
      this.isOffline = !online;
    }
  });
});
```

### WebSocket Integration

```javascript
// WebSocket connection for real-time updates
class TimingUpdates {
  constructor() {
    this.ws = null;
    this.connect();
  }
  
  connect() {
    if (window.location.pathname.includes('/cooking/')) {
      const sessionId = window.location.pathname.split('/').pop();
      this.ws = new WebSocket(`wss://${window.location.host}/ws/cooking/${sessionId}`);
      
      this.ws.onmessage = (event) => {
        const update = JSON.parse(event.data);
        
        // Trigger fragment refresh for timer updates
        if (update.type === 'timer_update') {
          document.querySelector('#timer-widget').dispatchEvent(
            new CustomEvent('ts:trigger', { detail: { force: true } })
          );
        }
      };
    }
  }
}
```
