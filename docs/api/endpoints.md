# API Endpoints Reference

Complete reference for all IMKitchen API endpoints, including request/response formats and examples.

## Table of Contents

- [Authentication Endpoints](#authentication-endpoints)
- [User Management](#user-management)
- [Recipe Management](#recipe-management)
- [Meal Planning](#meal-planning)
- [Shopping Lists](#shopping-lists)
- [Validation Endpoints](#validation-endpoints)
- [Static Content](#static-content)

## Authentication Endpoints

### Login and Registration

#### `GET /auth/login`
Display login form

**Response:** HTML login form with TwinSpark enhancements

```html
<div class="auth-container">
  <form class="login-form" ts-req="/auth/login" ts-target="#auth-result">
    <h2>Sign In to IMKitchen</h2>
    
    <div class="form-field">
      <label for="email">Email Address</label>
      <input 
        id="email" 
        name="email" 
        type="email" 
        required
        ts-req="/validate/email-format"
        ts-trigger="blur"
        ts-target="#email-validation">
      <div id="email-validation" class="validation-feedback"></div>
    </div>
    
    <div class="form-field">
      <label for="password">Password</label>
      <input id="password" name="password" type="password" required>
    </div>
    
    <div class="form-actions">
      <button type="submit" class="primary-button">Sign In</button>
      <a href="/auth/register" class="secondary-link">Create Account</a>
    </div>
    
    <div class="forgot-password">
      <a href="/auth/forgot-password">Forgot your password?</a>
    </div>
  </form>
  
  <div id="auth-result"></div>
</div>
```

#### `POST /auth/login`
Process login credentials

**Request Body:**
```
Content-Type: application/x-www-form-urlencoded

email=user@example.com
password=userpassword
csrf_token=abc123...
```

**Response (Success - 302 Redirect):**
```
Location: /dashboard
Set-Cookie: session_id=xyz789...; HttpOnly; Secure; SameSite=Strict
```

**Response (Error - 422 Validation Error):**
```html
<div class="login-form with-errors">
  <div class="auth-error">Invalid email or password. Please try again.</div>
  
  <form class="login-form" ts-req="/auth/login" ts-target="#auth-result">
    <div class="form-field error">
      <label for="email">Email Address</label>
      <input id="email" name="email" type="email" value="user@example.com" class="error">
      <div class="field-error">Please check your email and password</div>
    </div>
    
    <div class="form-field error">
      <label for="password">Password</label>
      <input id="password" name="password" type="password" class="error">
    </div>
    
    <button type="submit" class="primary-button">Sign In</button>
  </form>
</div>
```

#### `GET /auth/register`
Display registration form

**Response:**
```html
<div class="auth-container">
  <form class="register-form" ts-req="/auth/register" ts-target="#register-result">
    <h2>Create Your Account</h2>
    
    <div class="form-field">
      <label for="email">Email Address</label>
      <input 
        id="email" 
        name="email" 
        type="email" 
        required
        ts-req="/validate/email-availability"
        ts-trigger="blur"
        ts-target="#email-check">
      <div id="email-check" class="validation-feedback"></div>
    </div>
    
    <div class="form-field">
      <label for="password">Password</label>
      <input 
        id="password" 
        name="password" 
        type="password" 
        required
        minlength="8"
        ts-req="/validate/password-strength"
        ts-trigger="input"
        ts-target="#password-strength">
      <div id="password-strength" class="validation-feedback"></div>
    </div>
    
    <div class="form-field">
      <label for="confirm_password">Confirm Password</label>
      <input 
        id="confirm_password" 
        name="confirm_password" 
        type="password" 
        required>
    </div>
    
    <div class="form-field">
      <label for="display_name">Display Name</label>
      <input id="display_name" name="display_name" type="text">
    </div>
    
    <button type="submit" class="primary-button">Create Account</button>
  </form>
  
  <div id="register-result"></div>
</div>
```

#### `POST /auth/register`
Process registration

**Request Body:**
```
email=newuser@example.com
password=securepassword123
confirm_password=securepassword123
display_name=John Smith
csrf_token=abc123...
```

**Response (Success):**
```html
<div class="registration-success">
  <h3>Account Created Successfully!</h3>
  <p>Please check your email for a verification link.</p>
  <a href="/auth/login" class="primary-button">Sign In</a>
</div>
```

#### `POST /auth/logout`
Destroy user session

**Response (302 Redirect):**
```
Location: /
Set-Cookie: session_id=; Max-Age=0; HttpOnly; Secure
```

## User Management

### Profile Management

#### `GET /profile`
Display user profile

**Headers:** `Cookie: session_id=...` (authenticated)

**Response:**
```html
<div class="profile-page">
  <h1>Your Profile</h1>
  
  <form class="profile-form" ts-req="/profile" ts-target="#profile-result">
    <div class="form-section">
      <h2>Basic Information</h2>
      
      <div class="form-field">
        <label for="display_name">Display Name</label>
        <input id="display_name" name="display_name" value="{{ user.display_name }}">
      </div>
      
      <div class="form-field">
        <label for="email">Email Address</label>
        <input id="email" name="email" value="{{ user.email }}" readonly>
        <small>Contact support to change your email address</small>
      </div>
    </div>
    
    <div class="form-section">
      <h2>Cooking Preferences</h2>
      
      <div class="form-field">
        <label for="family_size">Family Size</label>
        <select id="family_size" name="family_size">
          <option value="1" {% if profile.family_size == 1 %}selected{% endif %}>1 person</option>
          <option value="2" {% if profile.family_size == 2 %}selected{% endif %}>2 people</option>
          <option value="4" {% if profile.family_size == 4 %}selected{% endif %}>4 people</option>
          <option value="6" {% if profile.family_size == 6 %}selected{% endif %}>6 people</option>
        </select>
      </div>
      
      <div class="form-field">
        <label for="skill_level">Cooking Skill Level</label>
        <select id="skill_level" name="skill_level">
          <option value="beginner" {% if profile.skill_level == "beginner" %}selected{% endif %}>Beginner</option>
          <option value="intermediate" {% if profile.skill_level == "intermediate" %}selected{% endif %}>Intermediate</option>
          <option value="advanced" {% if profile.skill_level == "advanced" %}selected{% endif %}>Advanced</option>
        </select>
      </div>
      
      <div class="form-field">
        <label>Dietary Restrictions</label>
        <div class="checkbox-group">
          {% for restriction in available_restrictions %}
          <label class="checkbox-label">
            <input 
              type="checkbox" 
              name="dietary_restrictions" 
              value="{{ restriction.value }}"
              {% if restriction.value in profile.dietary_restrictions %}checked{% endif %}>
            {{ restriction.label }}
          </label>
          {% endfor %}
        </div>
      </div>
    </div>
    
    <button type="submit" class="primary-button">Save Changes</button>
  </form>
  
  <div id="profile-result"></div>
</div>
```

#### `POST /profile`
Update user profile

**Request Body:**
```
display_name=John Smith
family_size=4
skill_level=intermediate
dietary_restrictions=vegetarian
dietary_restrictions=gluten_free
csrf_token=abc123...
```

**Response (Success):**
```html
<div class="profile-updated">
  <div class="success-message">✓ Profile updated successfully!</div>
  <!-- Updated profile form with new values -->
</div>
```

## Recipe Management

### Recipe CRUD Operations

#### `GET /recipes`
List recipes with filtering and pagination

**Query Parameters:**
- `page` (int, default: 1) - Page number
- `limit` (int, default: 20) - Items per page
- `cuisine` (string) - Filter by cuisine type
- `difficulty` (string) - Filter by difficulty (easy/medium/hard)
- `prep_time_max` (int) - Maximum prep time in minutes
- `search` (string) - Search in title and description

**Example Request:**
```
GET /recipes?cuisine=italian&difficulty=easy&page=1&limit=12
```

**Response:**
```html
<div class="recipes-page">
  <div class="recipes-header">
    <h1>Recipes</h1>
    <a href="/recipes/new" class="primary-button">Add Recipe</a>
  </div>
  
  <div class="recipes-filters">
    <form class="filter-form" ts-req="/recipes" ts-target="#recipes-content" ts-trigger="change">
      <select name="cuisine">
        <option value="">All Cuisines</option>
        <option value="italian" {% if filters.cuisine == "italian" %}selected{% endif %}>Italian</option>
        <option value="mexican" {% if filters.cuisine == "mexican" %}selected{% endif %}>Mexican</option>
        <option value="asian" {% if filters.cuisine == "asian" %}selected{% endif %}>Asian</option>
      </select>
      
      <select name="difficulty">
        <option value="">All Difficulties</option>
        <option value="easy" {% if filters.difficulty == "easy" %}selected{% endif %}>Easy</option>
        <option value="medium" {% if filters.difficulty == "medium" %}selected{% endif %}>Medium</option>
        <option value="hard" {% if filters.difficulty == "hard" %}selected{% endif %}>Hard</option>
      </select>
      
      <input 
        name="search" 
        type="search" 
        placeholder="Search recipes..."
        value="{{ filters.search }}"
        ts-req="/recipes"
        ts-trigger="keyup delay:500ms"
        ts-target="#recipes-content">
    </form>
  </div>
  
  <div id="recipes-content">
    <div class="recipes-grid">
      {% for recipe in recipes %}
      <div class="recipe-card" id="recipe-{{ recipe.id }}">
        <div class="recipe-image">
          {% if recipe.image_url %}
          <img src="{{ recipe.image_url }}" alt="{{ recipe.title }}">
          {% else %}
          <div class="placeholder-image">No Image</div>
          {% endif %}
        </div>
        
        <div class="recipe-info">
          <h3>{{ recipe.title }}</h3>
          <p class="recipe-description">{{ recipe.description | truncate(100) }}</p>
          
          <div class="recipe-meta">
            <span class="prep-time">{{ recipe.prep_time_minutes }}min prep</span>
            <span class="difficulty">{{ recipe.difficulty | title }}</span>
            <span class="cuisine">{{ recipe.cuisine_type | title }}</span>
          </div>
          
          <div class="recipe-actions">
            <button 
              ts-req="/recipes/{{ recipe.id }}" 
              ts-target="#recipe-modal" 
              ts-swap="innerHTML"
              class="view-button">
              View Recipe
            </button>
            
            {% if recipe.user_id == current_user.id %}
            <a href="/recipes/{{ recipe.id }}/edit" class="edit-link">Edit</a>
            {% endif %}
          </div>
        </div>
      </div>
      {% endfor %}
    </div>
    
    {% if recipes|length == 0 %}
    <div class="no-recipes">
      <p>No recipes found matching your criteria.</p>
      <a href="/recipes/new" class="primary-button">Create Your First Recipe</a>
    </div>
    {% endif %}
    
    <div class="pagination">
      {% if pagination.has_previous %}
      <button 
        ts-req="/recipes?page={{ pagination.previous_page }}&{{ current_filters }}"
        ts-target="#recipes-content">
        Previous
      </button>
      {% endif %}
      
      <span class="page-info">Page {{ pagination.current_page }} of {{ pagination.total_pages }}</span>
      
      {% if pagination.has_next %}
      <button 
        ts-req="/recipes?page={{ pagination.next_page }}&{{ current_filters }}"
        ts-target="#recipes-content">
        Next
      </button>
      {% endif %}
    </div>
  </div>
</div>

<!-- Modal for recipe details -->
<div id="recipe-modal" class="modal"></div>
```

#### `GET /recipes/{id}`
Get recipe details

**Response:**
```html
<div class="recipe-detail">
  <div class="recipe-header">
    <h1>{{ recipe.title }}</h1>
    <div class="recipe-meta">
      <span>Prep: {{ recipe.prep_time_minutes }} min</span>
      <span>Cook: {{ recipe.cook_time_minutes }} min</span>
      <span>Serves: {{ recipe.servings }}</span>
      <span class="difficulty-{{ recipe.difficulty }}">{{ recipe.difficulty | title }}</span>
    </div>
  </div>
  
  {% if recipe.image_url %}
  <div class="recipe-image">
    <img src="{{ recipe.image_url }}" alt="{{ recipe.title }}">
  </div>
  {% endif %}
  
  <div class="recipe-description">
    <p>{{ recipe.description }}</p>
  </div>
  
  <div class="recipe-content">
    <div class="ingredients-section">
      <h2>Ingredients</h2>
      <ul class="ingredients-list">
        {% for ingredient in recipe.ingredients %}
        <li class="ingredient-item">
          <span class="quantity">{{ ingredient.quantity }}</span>
          <span class="unit">{{ ingredient.unit }}</span>
          <span class="name">{{ ingredient.name }}</span>
          {% if ingredient.preparation %}
          <span class="preparation">({{ ingredient.preparation }})</span>
          {% endif %}
        </li>
        {% endfor %}
      </ul>
    </div>
    
    <div class="instructions-section">
      <h2>Instructions</h2>
      <ol class="instructions-list">
        {% for instruction in recipe.instructions %}
        <li class="instruction-step">
          <div class="step-content">{{ instruction.instruction }}</div>
          {% if instruction.duration_minutes %}
          <div class="step-duration">{{ instruction.duration_minutes }} minutes</div>
          {% endif %}
        </li>
        {% endfor %}
      </ol>
    </div>
  </div>
  
  <div class="recipe-actions">
    <button 
      ts-req="/meal-plans/add-recipe" 
      ts-data='{"recipe_id": "{{ recipe.id }}"}'
      ts-target="#meal-plan-modal"
      class="primary-button">
      Add to Meal Plan
    </button>
    
    {% if recipe.user_id == current_user.id %}
    <a href="/recipes/{{ recipe.id }}/edit" class="secondary-button">Edit Recipe</a>
    {% endif %}
    
    <button 
      ts-req="/recipes/{{ recipe.id }}/favorite" 
      ts-method="POST"
      ts-target="#favorite-status"
      class="favorite-button">
      ♥ Favorite
    </button>
  </div>
  
  <div id="favorite-status"></div>
</div>
```

#### `POST /recipes`
Create new recipe

**Request Body:**
```
title=Spaghetti Carbonara
description=Classic Roman pasta dish with eggs, cheese, and pancetta
prep_time_minutes=15
cook_time_minutes=15
servings=4
difficulty=medium
cuisine_type=italian
ingredients[0][name]=Spaghetti
ingredients[0][quantity]=400
ingredients[0][unit]=g
ingredients[1][name]=Pancetta
ingredients[1][quantity]=150
ingredients[1][unit]=g
instructions[0][instruction]=Bring a large pot of salted water to boil
instructions[0][step_number]=1
instructions[1][instruction]=Cook spaghetti according to package directions
instructions[1][step_number]=2
csrf_token=abc123...
```

**Response (Success - 201 Created):**
```html
<div class="recipe-created">
  <div class="success-message">
    ✓ Recipe "Spaghetti Carbonara" created successfully!
  </div>
  
  <div class="recipe-preview">
    <h3>{{ recipe.title }}</h3>
    <p>{{ recipe.description }}</p>
    <div class="actions">
      <a href="/recipes/{{ recipe.id }}" class="primary-button">View Recipe</a>
      <a href="/recipes/new" class="secondary-button">Create Another</a>
    </div>
  </div>
</div>
```

## Meal Planning

### Meal Plan Management

#### `GET /meal-plans`
List user's meal plans

**Response:**
```html
<div class="meal-plans-page">
  <div class="meal-plans-header">
    <h1>Your Meal Plans</h1>
    <button 
      ts-req="/meal-plans/new" 
      ts-target="#meal-plan-modal"
      class="primary-button">
      Create New Plan
    </button>
  </div>
  
  <div class="meal-plans-list">
    {% for plan in meal_plans %}
    <div class="meal-plan-card">
      <div class="plan-header">
        <h3>{{ plan.title }}</h3>
        <span class="plan-date">Week of {{ plan.week_start_date | date_format }}</span>
      </div>
      
      <div class="plan-summary">
        <span>{{ plan.total_meals }} meals planned</span>
        <span>{{ plan.recipes_count }} unique recipes</span>
      </div>
      
      <div class="plan-actions">
        <button 
          ts-req="/meal-plans/{{ plan.id }}" 
          ts-target="#meal-plan-detail"
          class="view-button">
          View Plan
        </button>
        
        <button 
          ts-req="/meal-plans/{{ plan.id }}/shopping-list" 
          ts-target="#shopping-list-modal"
          class="shopping-button">
          Shopping List
        </button>
      </div>
    </div>
    {% endfor %}
  </div>
  
  <div id="meal-plan-detail"></div>
  <div id="meal-plan-modal" class="modal"></div>
  <div id="shopping-list-modal" class="modal"></div>
</div>
```

#### `GET /meal-plans/{id}`
Get meal plan details

**Response:**
```html
<div class="meal-plan-detail">
  <div class="plan-header">
    <h2>{{ meal_plan.title }}</h2>
    <span class="plan-date">{{ meal_plan.week_start_date | date_format }} - {{ meal_plan.week_end_date | date_format }}</span>
  </div>
  
  <div class="week-calendar">
    {% for day in week_days %}
    <div class="day-column">
      <h3>{{ day.name }}</h3>
      <div class="day-date">{{ day.date | date_format }}</div>
      
      {% for meal_type in ['breakfast', 'lunch', 'dinner'] %}
      <div class="meal-slot" 
           data-day="{{ day.index }}" 
           data-meal="{{ meal_type }}"
           id="meal-{{ day.index }}-{{ meal_type }}">
        
        {% if day.meals[meal_type] %}
        <div class="planned-meal">
          <h4>{{ day.meals[meal_type].recipe.title }}</h4>
          <div class="meal-meta">
            <span>{{ day.meals[meal_type].servings }} servings</span>
            <span>{{ day.meals[meal_type].recipe.prep_time_minutes }}min</span>
          </div>
          
          <div class="meal-actions">
            <button 
              ts-req="/recipes/{{ day.meals[meal_type].recipe.id }}" 
              ts-target="#recipe-modal"
              class="view-recipe">
              View
            </button>
            
            <button 
              ts-req="/meal-plans/{{ meal_plan.id }}/entries/{{ day.meals[meal_type].id }}" 
              ts-method="DELETE"
              ts-target="#meal-{{ day.index }}-{{ meal_type }}"
              class="remove-meal">
              Remove
            </button>
          </div>
        </div>
        {% else %}
        <div class="empty-meal">
          <p class="empty-text">No {{ meal_type }} planned</p>
          <button 
            ts-req="/meal-plans/{{ meal_plan.id }}/suggest" 
            ts-data='{"day_of_week": {{ day.index }}, "meal_type": "{{ meal_type }}"}'
            ts-target="#meal-{{ day.index }}-{{ meal_type }}"
            class="suggest-meal">
            Add Meal
          </button>
        </div>
        {% endif %}
      </div>
      {% endfor %}
    </div>
    {% endfor %}
  </div>
  
  <div class="plan-actions">
    <button 
      ts-req="/meal-plans/{{ meal_plan.id }}/shopping-list" 
      ts-target="#shopping-list-modal"
      class="primary-button">
      Generate Shopping List
    </button>
    
    <button 
      ts-req="/meal-plans/{{ meal_plan.id }}/duplicate" 
      ts-target="#meal-plan-result"
      class="secondary-button">
      Duplicate Plan
    </button>
  </div>
</div>
```

## Shopping Lists

#### `GET /shopping-lists/{id}`
Get shopping list details

**Response:**
```html
<div class="shopping-list">
  <div class="list-header">
    <h2>{{ shopping_list.title }}</h2>
    {% if shopping_list.store_name %}
    <span class="store-name">For: {{ shopping_list.store_name }}</span>
    {% endif %}
  </div>
  
  <div class="list-progress">
    <div class="progress-bar">
      <div class="progress-fill" style="width: {{ completion_percentage }}%"></div>
    </div>
    <span class="progress-text">{{ completed_items }}/{{ total_items }} items</span>
  </div>
  
  <div class="shopping-categories">
    {% for category in categories %}
    <div class="category-section">
      <h3>{{ category.name }}</h3>
      
      <div class="items-list">
        {% for item in category.items %}
        <div class="shopping-item {% if item.is_purchased %}completed{% endif %}" 
             id="item-{{ item.id }}">
          <label class="item-checkbox">
            <input 
              type="checkbox" 
              {% if item.is_purchased %}checked{% endif %}
              ts-req="/shopping-lists/{{ shopping_list.id }}/items/{{ item.id }}/toggle"
              ts-method="POST"
              ts-target="#item-{{ item.id }}">
            
            <div class="item-details">
              <span class="item-name">{{ item.ingredient_name }}</span>
              <span class="item-quantity">{{ item.quantity }} {{ item.unit }}</span>
              {% if item.notes %}
              <span class="item-notes">{{ item.notes }}</span>
              {% endif %}
            </div>
            
            {% if item.estimated_price %}
            <span class="item-price">${{ item.estimated_price | currency }}</span>
            {% endif %}
          </label>
        </div>
        {% endfor %}
      </div>
    </div>
    {% endfor %}
  </div>
  
  <div class="list-summary">
    <div class="cost-estimate">
      <span>Estimated Total: ${{ estimated_total | currency }}</span>
    </div>
    
    <div class="list-actions">
      <button 
        ts-req="/shopping-lists/{{ shopping_list.id }}/complete"
        ts-method="POST"
        ts-target="#shopping-result"
        class="primary-button">
        Mark Shopping Complete
      </button>
    </div>
  </div>
</div>
```

## Validation Endpoints

### Real-time Validation

#### `POST /validate/email-availability`
Check if email is available for registration

**Request Body:**
```
email=user@example.com
```

**Response (Available):**
```html
<div class="validation-success">
  <span class="check-icon">✓</span>
  Email is available
</div>
```

**Response (Taken):**
```html
<div class="validation-error">
  <span class="error-icon">✗</span>
  This email is already registered
</div>
```

#### `POST /validate/password-strength`
Validate password strength

**Request Body:**
```
password=mypassword123
```

**Response:**
```html
<div class="password-strength">
  <div class="strength-meter strength-medium">
    <div class="strength-fill" style="width: 60%"></div>
  </div>
  <div class="strength-requirements">
    <div class="requirement met">✓ At least 8 characters</div>
    <div class="requirement met">✓ Contains numbers</div>
    <div class="requirement not-met">✗ Contains uppercase letter</div>
    <div class="requirement not-met">✗ Contains special character</div>
  </div>
  <div class="strength-label">Medium strength</div>
</div>
```

#### `POST /validate/recipe-title`
Validate recipe title uniqueness for user

**Request Body:**
```
title=Pasta Carbonara
```

**Response (Valid):**
```html
<div class="validation-success">
  <span class="check-icon">✓</span>
  Title looks good
</div>
```

**Response (Duplicate):**
```html
<div class="validation-warning">
  <span class="warning-icon">⚠</span>
  You already have a recipe with this title
</div>
```

## Static Content

### Asset Serving

#### `GET /static/css/tailwind.css`
Compiled CSS styles

#### `GET /static/js/twinspark.js`
TwinSpark JavaScript library

#### `GET /static/images/recipes/{id}.{ext}`
Recipe images

#### `GET /static/icons/{name}.svg`
UI icons and graphics

### Health Check

#### `GET /health`
Application health status

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "database": "connected",
  "timestamp": "2025-09-29T10:30:00Z"
}
```

## Error Responses

### Standard Error Format

All endpoints return consistent error responses:

#### 400 Bad Request
```html
<div class="error-page">
  <h2>Bad Request</h2>
  <p>The request could not be understood. Please check your input and try again.</p>
  <a href="/" class="button">Go Home</a>
</div>
```

#### 401 Unauthorized
```html
<div class="auth-required">
  <h2>Sign In Required</h2>
  <p>You need to sign in to access this page.</p>
  <a href="/auth/login" class="primary-button">Sign In</a>
</div>
```

#### 404 Not Found
```html
<div class="not-found">
  <h2>Page Not Found</h2>
  <p>The page you're looking for doesn't exist.</p>
  <a href="/" class="primary-button">Go Home</a>
</div>
```

#### 422 Validation Error
```html
<div class="validation-errors">
  <h3>Please fix the following errors:</h3>
  <ul class="error-list">
    <li>Recipe title is required</li>
    <li>At least one ingredient must be specified</li>
    <li>Preparation time must be a positive number</li>
  </ul>
</div>
```

For more API documentation:
- [TwinSpark Integration](twinspark.md)
- [Authentication Guide](authentication.md)
- [Error Handling](errors.md)
- [Testing API Endpoints](testing.md)