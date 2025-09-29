# API Documentation

IMKitchen uses server-side rendered templates with TwinSpark for dynamic interactions. This documentation covers the API patterns, authentication flows, and endpoint specifications.

## Table of Contents

- [API Overview](#api-overview)
- [TwinSpark Patterns](#twinspark-patterns)
- [Authentication Flow](#authentication-flow)
- [Request/Response Formats](#requestresponse-formats)
- [Endpoint Reference](#endpoint-reference)
- [Error Handling](#error-handling)
- [Examples](#examples)

## API Overview

### Architecture Pattern

IMKitchen follows a **Server-Side Rendered + Progressive Enhancement** pattern:

- **Primary**: Server-rendered Askama templates with full functionality
- **Enhancement**: TwinSpark JavaScript for dynamic interactions
- **Fallback**: Full page refreshes when JavaScript is disabled

### Key Principles

- **Progressive Enhancement**: Works without JavaScript, enhanced with it
- **Type Safety**: All endpoints use strongly-typed Rust handlers
- **Security First**: CSRF protection, input validation, secure sessions
- **Performance**: Minimal JavaScript, optimized server rendering

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Backend** | Axum 0.8+ | HTTP server and routing |
| **Templates** | Askama 0.14+ | Server-side HTML rendering |
| **Enhancement** | TwinSpark 1.0+ | Progressive JavaScript interactions |
| **Authentication** | Sessions | Secure cookie-based auth |
| **Validation** | validator 0.20+ | Input validation and sanitization |

## TwinSpark Patterns

### Overview

TwinSpark enables dynamic interactions without custom JavaScript:

```html
<!-- Basic form submission with dynamic response -->
<form ts-req="/recipes" ts-swap="innerHTML" ts-target="#recipe-list">
  <input name="title" required>
  <button type="submit">Add Recipe</button>
</form>

<!-- Dynamic content loading -->
<button ts-req="/recipes/123" ts-swap="outerHTML" ts-target="#recipe-card-123">
  Load Details
</button>

<!-- Form validation with real-time feedback -->
<input name="email" ts-req="/validate/email" ts-trigger="blur" ts-target="#email-error">
```

### TwinSpark Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `ts-req` | HTTP request endpoint | `ts-req="/recipes"` |
| `ts-swap` | Content replacement method | `ts-swap="innerHTML"` |
| `ts-target` | Target element selector | `ts-target="#content"` |
| `ts-trigger` | Event trigger | `ts-trigger="click"` |
| `ts-method` | HTTP method override | `ts-method="DELETE"` |

### Common Patterns

#### 1. Form Submission with Validation

```html
<form id="recipe-form" ts-req="/recipes" ts-target="#form-result">
  <div class="field">
    <label for="title">Recipe Title</label>
    <input 
      id="title" 
      name="title" 
      required
      ts-req="/validate/recipe-title"
      ts-trigger="blur"
      ts-target="#title-error">
    <div id="title-error" class="error-message"></div>
  </div>
  
  <button type="submit">Create Recipe</button>
</form>

<div id="form-result"></div>
```

#### 2. Dynamic Content Loading

```html
<!-- Recipe list with pagination -->
<div id="recipe-list">
  <!-- Initial recipes loaded server-side -->
</div>

<button 
  ts-req="/recipes?page=2" 
  ts-swap="beforeend" 
  ts-target="#recipe-list">
  Load More Recipes
</button>
```

#### 3. Interactive Components

```html
<!-- Recipe rating component -->
<div class="recipe-rating" id="rating-123">
  <span>Rate this recipe:</span>
  <button ts-req="/recipes/123/rate" ts-method="POST" ts-data='{"rating": 1}' ts-target="#rating-123">★</button>
  <button ts-req="/recipes/123/rate" ts-method="POST" ts-data='{"rating": 2}' ts-target="#rating-123">★</button>
  <button ts-req="/recipes/123/rate" ts-method="POST" ts-data='{"rating": 3}' ts-target="#rating-123">★</button>
  <button ts-req="/recipes/123/rate" ts-method="POST" ts-data='{"rating": 4}' ts-target="#rating-123">★</button>
  <button ts-req="/recipes/123/rate" ts-method="POST" ts-data='{"rating": 5}' ts-target="#rating-123">★</button>
</div>
```

## Authentication Flow

### Session-Based Authentication

IMKitchen uses secure cookie-based sessions:

```rust
// Session structure
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub csrf_token: String,
}
```

### Authentication Endpoints

#### 1. Login Flow

```http
POST /auth/login
Content-Type: application/x-www-form-urlencoded

email=user@example.com&password=secretpassword
```

**Response (Success):**
```html
<!-- Redirect or render dashboard -->
<div class="dashboard">
  <h1>Welcome back, User!</h1>
  <!-- Dashboard content -->
</div>
```

**Response (Error):**
```html
<div class="login-form">
  <div class="error">Invalid email or password</div>
  <!-- Form with error state -->
</div>
```

#### 2. Registration Flow

```http
POST /auth/register
Content-Type: application/x-www-form-urlencoded

email=user@example.com&password=securepassword&confirm_password=securepassword
```

#### 3. Logout Flow

```http
POST /auth/logout
```

### Authentication Middleware

All protected endpoints require valid session:

```rust
// Protected route example
async fn protected_handler(
    session: UserSession,  // Extracted by middleware
    // ... other parameters
) -> Result<Html<String>, AppError> {
    // Handler logic with authenticated user
}
```

### CSRF Protection

All forms include CSRF tokens:

```html
<form method="POST" action="/recipes">
  <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
  <!-- Form fields -->
</form>
```

## Request/Response Formats

### Content Types

| Content Type | Usage | Example |
|--------------|-------|---------|
| `text/html` | Page rendering, fragments | Main pages, TwinSpark responses |
| `application/x-www-form-urlencoded` | Form submissions | Login, recipe creation |
| `application/json` | API responses | Validation results, data exchange |

### Request Headers

```http
Content-Type: application/x-www-form-urlencoded
Cookie: session_id=abc123...
X-Requested-With: XMLHttpRequest  # For TwinSpark requests
```

### Response Headers

```http
Content-Type: text/html; charset=utf-8
Set-Cookie: session_id=abc123...; HttpOnly; Secure; SameSite=Strict
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
```

### HTML Response Format

#### Full Page Response
```html
<!DOCTYPE html>
<html>
<head>
  <title>IMKitchen - Recipe Details</title>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="/static/css/tailwind.css">
</head>
<body>
  <nav><!-- Navigation --></nav>
  <main><!-- Page content --></main>
  <script src="/static/js/twinspark.js"></script>
</body>
</html>
```

#### Fragment Response (TwinSpark)
```html
<!-- Just the updated content -->
<div class="recipe-card" id="recipe-123">
  <h3>{{ recipe.title }}</h3>
  <p>{{ recipe.description }}</p>
  <div class="actions">
    <button ts-req="/recipes/123/favorite" ts-target="#recipe-123">♥ Favorite</button>
  </div>
</div>
```

## Endpoint Reference

### Authentication Endpoints

#### `GET /auth/login`
Display login form

**Response:**
```html
<form class="login-form" ts-req="/auth/login" ts-target="#auth-content">
  <input name="email" type="email" required>
  <input name="password" type="password" required>
  <button type="submit">Login</button>
</form>
```

#### `POST /auth/login`
Process login credentials

**Request:**
```
email=user@example.com&password=secret
```

**Response (Success):** Redirect to `/dashboard`
**Response (Error):** Login form with error message

#### `POST /auth/logout`
Destroy user session

**Response:** Redirect to `/`

### Recipe Endpoints

#### `GET /recipes`
List recipes with pagination and filters

**Query Parameters:**
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)
- `cuisine` - Filter by cuisine type
- `difficulty` - Filter by difficulty level

**Response:**
```html
<div class="recipe-grid">
  <div class="recipe-card" id="recipe-123">
    <img src="/static/images/recipes/123.jpg" alt="Recipe image">
    <h3>Pasta Carbonara</h3>
    <p>Prep: 15 min | Cook: 10 min | Difficulty: Medium</p>
    <div class="actions">
      <button ts-req="/recipes/123" ts-target="#recipe-detail">View</button>
    </div>
  </div>
  <!-- More recipe cards -->
</div>

<div class="pagination">
  <button ts-req="/recipes?page=2" ts-target="#recipe-grid">Next Page</button>
</div>
```

#### `GET /recipes/{id}`
Get recipe details

**Response:**
```html
<div class="recipe-detail" id="recipe-detail">
  <h1>{{ recipe.title }}</h1>
  <div class="recipe-meta">
    <span>Prep: {{ recipe.prep_time }} min</span>
    <span>Cook: {{ recipe.cook_time }} min</span>
    <span>Serves: {{ recipe.servings }}</span>
  </div>
  
  <div class="ingredients">
    <h2>Ingredients</h2>
    <ul>
      {% for ingredient in recipe.ingredients %}
      <li>{{ ingredient.quantity }} {{ ingredient.unit }} {{ ingredient.name }}</li>
      {% endfor %}
    </ul>
  </div>
  
  <div class="instructions">
    <h2>Instructions</h2>
    <ol>
      {% for step in recipe.instructions %}
      <li>{{ step.instruction }}</li>
      {% endfor %}
    </ol>
  </div>
</div>
```

#### `POST /recipes`
Create new recipe

**Request:**
```
title=Pasta Carbonara
description=Classic Italian pasta dish
prep_time_minutes=15
cook_time_minutes=10
servings=4
difficulty=medium
ingredients[0][name]=Spaghetti
ingredients[0][quantity]=400
ingredients[0][unit]=g
```

**Response (Success):** Recipe detail view
**Response (Error):** Form with validation errors

### Meal Planning Endpoints

#### `GET /meal-plans`
List user's meal plans

**Response:**
```html
<div class="meal-plans">
  {% for plan in meal_plans %}
  <div class="meal-plan-card">
    <h3>{{ plan.title }}</h3>
    <p>Week of {{ plan.week_start_date }}</p>
    <button ts-req="/meal-plans/{{ plan.id }}" ts-target="#meal-plan-detail">View Plan</button>
  </div>
  {% endfor %}
</div>
```

#### `POST /meal-plans/{id}/entries`
Add meal to plan

**Request:**
```
recipe_id=123
day_of_week=1
meal_type=dinner
servings=4
```

### Validation Endpoints

#### `POST /validate/email`
Validate email address availability

**Request:**
```
email=user@example.com
```

**Response (Available):**
```html
<div class="validation-success">✓ Email available</div>
```

**Response (Taken):**
```html
<div class="validation-error">✗ Email already in use</div>
```

## Error Handling

### Error Response Format

```html
<!-- Validation errors -->
<div class="form-errors">
  <div class="error">Email is required</div>
  <div class="error">Password must be at least 8 characters</div>
</div>

<!-- System errors -->
<div class="system-error">
  <h3>Something went wrong</h3>
  <p>Please try again or contact support if the problem persists.</p>
</div>
```

### HTTP Status Codes

| Code | Usage | Example |
|------|-------|---------|
| 200 | Success | Recipe created successfully |
| 302 | Redirect | Login success → dashboard |
| 400 | Bad Request | Invalid form data |
| 401 | Unauthorized | Login required |
| 403 | Forbidden | Access denied |
| 404 | Not Found | Recipe doesn't exist |
| 422 | Validation Error | Form validation failed |
| 500 | Server Error | Database connection failed |

### Error Handling Patterns

```rust
// Handler error types
#[derive(Debug)]
pub enum AppError {
    ValidationError(Vec<String>),
    NotFound(String),
    Unauthorized,
    DatabaseError(sqlx::Error),
    InternalError(String),
}

// Error response rendering
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ValidationError(errors) => {
                // Render validation errors template
            }
            AppError::NotFound(resource) => {
                // Render 404 page
            }
            // ... other error types
        }
    }
}
```

## Examples

### Complete Recipe Creation Flow

#### 1. Display Recipe Form

```http
GET /recipes/new
```

```html
<form id="recipe-form" ts-req="/recipes" ts-target="#recipe-result">
  <div class="field">
    <label for="title">Recipe Title</label>
    <input 
      id="title" 
      name="title" 
      required
      ts-req="/validate/recipe-title"
      ts-trigger="blur"
      ts-target="#title-feedback">
    <div id="title-feedback"></div>
  </div>
  
  <div class="field">
    <label for="description">Description</label>
    <textarea id="description" name="description"></textarea>
  </div>
  
  <div class="ingredients-section">
    <h3>Ingredients</h3>
    <div id="ingredients-list">
      <div class="ingredient-row">
        <input name="ingredients[0][quantity]" type="number" placeholder="Amount">
        <select name="ingredients[0][unit]">
          <option value="g">grams</option>
          <option value="ml">milliliters</option>
          <option value="pieces">pieces</option>
        </select>
        <input name="ingredients[0][name]" placeholder="Ingredient name">
      </div>
    </div>
    <button type="button" ts-req="/recipes/new/ingredient-row" ts-target="#ingredients-list" ts-swap="beforeend">
      Add Ingredient
    </button>
  </div>
  
  <button type="submit">Create Recipe</button>
</form>

<div id="recipe-result"></div>
```

#### 2. Add Ingredient Row Dynamically

```http
POST /recipes/new/ingredient-row
```

```html
<div class="ingredient-row">
  <input name="ingredients[{{ index }}][quantity]" type="number" placeholder="Amount">
  <select name="ingredients[{{ index }}][unit]">
    <option value="g">grams</option>
    <option value="ml">milliliters</option>
    <option value="pieces">pieces</option>
  </select>
  <input name="ingredients[{{ index }}][name]" placeholder="Ingredient name">
  <button type="button" ts-req="/recipes/ingredient-row/remove" ts-method="DELETE" ts-target=".ingredient-row" ts-swap="delete">Remove</button>
</div>
```

#### 3. Validate Recipe Title

```http
POST /validate/recipe-title
Content-Type: application/x-www-form-urlencoded

title=Pasta Carbonara
```

```html
<!-- Success response -->
<div class="validation-success">✓ Title looks good</div>

<!-- Error response -->
<div class="validation-error">✗ You already have a recipe with this title</div>
```

#### 4. Submit Recipe

```http
POST /recipes
Content-Type: application/x-www-form-urlencoded

title=Pasta Carbonara
description=Classic Italian pasta dish
prep_time_minutes=15
cook_time_minutes=10
servings=4
difficulty=medium
ingredients[0][quantity]=400
ingredients[0][unit]=g
ingredients[0][name]=Spaghetti
ingredients[1][quantity]=4
ingredients[1][unit]=pieces
ingredients[1][name]=Eggs
```

```html
<!-- Success response -->
<div class="recipe-created">
  <div class="success-message">✓ Recipe created successfully!</div>
  <div class="recipe-preview">
    <h3>Pasta Carbonara</h3>
    <p>Classic Italian pasta dish</p>
    <a href="/recipes/456" class="button">View Recipe</a>
  </div>
</div>

<!-- Error response -->
<div class="recipe-form-errors">
  <div class="error">Preparation time is required</div>
  <div class="error">At least one ingredient is required</div>
  <!-- Form re-rendered with errors and user input preserved -->
</div>
```

### Meal Planning Flow

#### 1. Generate Weekly Meal Plan

```http
POST /meal-plans/generate
Content-Type: application/x-www-form-urlencoded

week_start_date=2025-09-29
dietary_restrictions=vegetarian
family_size=4
```

```html
<div class="meal-plan" id="weekly-plan">
  <h2>Meal Plan for Week of Sept 29, 2025</h2>
  
  <div class="week-grid">
    {% for day in days %}
    <div class="day-column">
      <h3>{{ day.name }}</h3>
      
      {% for meal_type in ['breakfast', 'lunch', 'dinner'] %}
      <div class="meal-slot" data-day="{{ day.index }}" data-meal="{{ meal_type }}">
        {% if day.meals[meal_type] %}
          <div class="assigned-meal">
            <h4>{{ day.meals[meal_type].recipe.title }}</h4>
            <p>{{ day.meals[meal_type].recipe.prep_time }}min</p>
            <button ts-req="/meal-plans/{{ plan.id }}/entries/{{ day.meals[meal_type].id }}" 
                    ts-method="DELETE" 
                    ts-target=".meal-slot[data-day='{{ day.index }}'][data-meal='{{ meal_type }}']">
              Remove
            </button>
          </div>
        {% else %}
          <div class="empty-meal">
            <p>No meal planned</p>
            <button ts-req="/meal-plans/{{ plan.id }}/suggest" 
                    ts-data='{"day": {{ day.index }}, "meal_type": "{{ meal_type }}"}'
                    ts-target=".meal-slot[data-day='{{ day.index }}'][data-meal='{{ meal_type }}']">
              Suggest Recipe
            </button>
          </div>
        {% endif %}
      </div>
      {% endfor %}
    </div>
    {% endfor %}
  </div>
  
  <div class="meal-plan-actions">
    <button ts-req="/meal-plans/{{ plan.id }}/shopping-list" ts-target="#shopping-list-modal">
      Generate Shopping List
    </button>
  </div>
</div>
```

For more API details, see:
- [TwinSpark Integration Guide](twinspark.md)
- [Authentication Guide](authentication.md)
- [Validation Patterns](validation.md)
- [Error Handling](errors.md)