# HTML Interface Specification

## Page Routes and Form Handling

ImKitchen uses server-side rendering with Askama templates. All interactions happen through standard HTML forms and twinspark-js fragment updates.

### Authentication Forms

```html
<!-- Login Form -->
<form method="post" action="/auth/login">
  <input name="email" type="email" required>
  <input name="password" type="password" required>
  <button type="submit">Login</button>
</form>

<!-- Registration Form -->
<form method="post" action="/auth/register">
  <input name="email" type="email" required>
  <input name="password" type="password" required>
  <select name="dietary_preferences" multiple>
    <option value="vegetarian">Vegetarian</option>
    <option value="vegan">Vegan</option>
    <option value="gluten_free">Gluten Free</option>
  </select>
  <button type="submit">Register</button>
</form>
```

### Recipe Management Forms

```html
<!-- Create Recipe Form -->
<form method="post" action="/recipes" 
      ts-req="/fragments/recipes"
      ts-trigger="submit"
      ts-target="#recipe-list">
  <input name="title" required>
  <textarea name="description"></textarea>
  <input name="prep_time" type="number" required>
  <input name="cook_time" type="number" required>
  <textarea name="ingredients" required></textarea>
  <textarea name="instructions" required></textarea>
  <button type="submit">Add Recipe</button>
</form>

<!-- Recipe Import Form -->
<form method="post" action="/recipes/import"
      ts-req="/fragments/recipe-import-status"
      ts-trigger="submit"
      ts-target="#import-status">
  <input name="url" type="url" placeholder="Recipe URL" required>
  <button type="submit">Import Recipe</button>
</form>
```

### Cooking Session Forms

```html
<!-- Start Cooking Session -->
<form method="post" action="/cooking/sessions">
  <input name="recipe_id" type="hidden" value="{{ recipe.id }}">
  <input name="scaling_factor" type="number" step="0.1" value="1.0">
  <button type="submit">Start Cooking</button>
</form>

<!-- Add Timer -->
<form method="post" action="/cooking/sessions/{{ session.id }}/timers"
      ts-req="/fragments/timers"
      ts-trigger="submit"
      ts-target="#timer-widget">
  <input name="name" placeholder="Timer name" required>
  <input name="duration" type="number" placeholder="Minutes" required>
  <button type="submit">Add Timer</button>
</form>
```

### Fragment Endpoints

Fragment endpoints return HTML snippets for twinspark-js to update page sections:

| Endpoint | Purpose | Returns |
|----------|---------|---------|
| `/fragments/recipes` | Recipe list updates | `<div id="recipe-list">...</div>` |
| `/fragments/recipe/{id}` | Single recipe card | `<div class="recipe-card">...</div>` |
| `/fragments/timers` | Timer widget updates | `<div id="timer-widget">...</div>` |
| `/fragments/meal-plan` | Meal plan updates | `<div id="meal-plan">...</div>` |
| `/fragments/notifications` | Notification updates | `<div id="notifications">...</div>` |

### Response Patterns

**Successful Form Submission:**
- Full page forms: Redirect to success page (HTTP 302)
- Fragment forms: Return updated HTML fragment (HTTP 200)

**Form Validation Errors:**
- Full page forms: Re-render form page with error messages
- Fragment forms: Return form fragment with error highlights

**Authentication Required:**
- Redirect to login page (HTTP 302) with return URL parameter
