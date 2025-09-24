# Components

## Meal Planning Engine
**Responsibility:** Intelligent weekly meal plan generation using multi-factor optimization considering user preferences, recipe rotation, preparation complexity, and family schedule constraints

**Key Interfaces:**
- `generate_meal_plan(user_id, week_start, preferences) -> MealPlan`
- `reschedule_meal(meal_plan_entry, new_date) -> Result<(), Error>`
- `suggest_easy_alternatives(recipe_id) -> Vec<Recipe>`

**Dependencies:** Recipe repository, User preferences service, Event system for notifications

**Technology Stack:** Rust with custom optimization algorithms, SQLite for recipe data access, Evento for meal plan change events

## Recipe Management Service
**Responsibility:** CRUD operations for recipes, community rating system, recipe collection management, and search/filtering functionality

**Key Interfaces:**
- `create_recipe(recipe_data) -> Recipe`
- `search_recipes(filters) -> Vec<Recipe>`
- `rate_recipe(user_id, recipe_id, rating) -> RecipeRating`
- `get_user_collections(user_id) -> Vec<RecipeCollection>`

**Dependencies:** User authentication service, Image storage service, Database layer

**Technology Stack:** Rust with Axum handlers, SQLite for data persistence, File system for recipe images

## User Authentication Service
**Responsibility:** OWASP-compliant user registration, login, session management, and authorization with secure password handling and session token management

**Key Interfaces:**
- `register_user(email, password, profile) -> Result<User, Error>`
- `authenticate(email, password) -> Result<Session, Error>`
- `validate_session(session_token) -> Result<User, Error>`
- `logout(session_token) -> Result<(), Error>`

**Dependencies:** Password hashing library, Session storage, Email validation

**Technology Stack:** Rust with argon2 password hashing, secure session cookies, CSRF protection middleware

## Shopping List Service
**Responsibility:** Automatic shopping list generation from meal plans with ingredient consolidation, store section organization, and family sharing capabilities

**Key Interfaces:**
- `generate_shopping_list(meal_plan_id) -> ShoppingList`
- `consolidate_ingredients(recipes) -> Vec<ShoppingItem>`
- `share_with_family(list_id, user_ids) -> Result<(), Error>`
- `mark_purchased(item_id, user_id) -> Result<(), Error>`

**Dependencies:** Meal plan service, Recipe repository, User management service

**Technology Stack:** Rust with ingredient normalization algorithms, SQLite for list persistence, real-time updates via Evento

## Community Features Service
**Responsibility:** Recipe sharing, user-generated content moderation, community challenges, and social interaction features including recipe ratings and reviews

**Key Interfaces:**
- `publish_recipe(user_id, recipe_id) -> Result<(), Error>`
- `get_trending_recipes() -> Vec<Recipe>`
- `submit_review(user_id, recipe_id, review) -> RecipeReview`
- `moderate_content(content_id, action) -> Result<(), Error>`

**Dependencies:** Recipe service, User authentication, Content moderation filters, Notification service

**Technology Stack:** Rust with community algorithms, SQLite for social data, basic content filtering rules
