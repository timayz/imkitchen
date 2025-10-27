## imkitchen Meal Planning API Documentation

**Version:** 1.0.0
**Base URL:** `http://localhost:3000` (development) | `https://api.imkitchen.app` (production)

### Overview

The imkitchen Meal Planning API provides automated multi-week meal planning functionality with intelligent recipe scheduling, advance preparation tracking, and shopping list generation.

**Key Features:**
- Multi-week meal plan generation (up to 5 weeks)
- Week-by-week navigation and viewing
- Single week and bulk week regeneration
- Customizable meal planning preferences

**Authentication:** All routes require a JWT cookie (`session`) containing a valid authentication token.

---

### Authentication

All meal planning routes require authentication via JWT cookie.

**Cookie Name:** `session`
**Type:** JWT (JSON Web Token)
**Expiration:** 30 days (configurable)

**Example Request with Cookie:**
```bash
curl -X POST http://localhost:3000/plan/generate-multi-week \
  -H "Cookie: session=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Error Responses:**
- `401 Unauthorized`: JWT cookie missing or invalid
- `403 Forbidden`: Valid authentication but insufficient permissions (e.g., accessing another user's week)

---

### Rate Limiting

To prevent abuse and ensure fair usage, the following rate limits apply:

| Route | Limit |
|-------|-------|
| POST /plan/generate-multi-week | 5 requests/hour per user |
| POST /plan/week/:week_id/regenerate | 10 requests/hour per user |
| POST /plan/regenerate-all-future | 10 requests/hour per user |
| GET /plan/week/:week_id | No limit (read-only) |
| PUT /profile/meal-planning-preferences | No limit |

**Rate Limit Headers:**
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets

**Rate Limit Exceeded Response:**
```json
{
  "error": "RateLimitExceeded",
  "message": "Too many requests. Please try again in 45 minutes.",
  "retry_after": 2700
}
```
**HTTP Status:** `429 Too Many Requests`

---

### API Routes

#### 1. Generate Multi-Week Meal Plan

Generates an automated multi-week meal plan (up to 5 weeks) based on user's favorite recipes and preferences.

**Endpoint:** `POST /plan/generate-multi-week`
**Authentication:** Required
**Rate Limit:** 5 requests/hour

**Request:**
```bash
curl -X POST http://localhost:3000/plan/generate-multi-week \
  -H "Cookie: session={JWT}"
```

**Response (200 OK):**
```json
{
  "generation_batch_id": "550e8400-e29b-41d4-a716-446655440000",
  "max_weeks_possible": 5,
  "current_week_index": 0,
  "first_week": {
    "id": "week_1",
    "start_date": "2025-10-28",
    "end_date": "2025-11-03",
    "status": "future",
    "is_locked": false,
    "meal_assignments": [
      {
        "id": "assignment_1",
        "date": "2025-10-28",
        "course_type": "main_course",
        "recipe": {
          "id": "recipe_123",
          "title": "Chicken Tikka Masala",
          "prep_time_min": 20,
          "cook_time_min": 30,
          "complexity": "moderate"
        },
        "accompaniment": {
          "id": "accompaniment_45",
          "title": "Basmati Rice",
          "category": "rice"
        },
        "prep_required": true,
        "algorithm_reasoning": "Saturday: Weekend allows longer prep time"
      }
      // ... 20 more assignments (7 days × 3 meals)
    ],
    "shopping_list_id": "shopping_1"
  },
  "navigation": {
    "next_week_id": "week_2",
    "week_links": [
      { "week_id": "week_1", "start_date": "2025-10-28", "is_current": false },
      { "week_id": "week_2", "start_date": "2025-11-04", "is_current": false }
    ]
  }
}
```

**Error Responses:**

**400 Bad Request - Insufficient Recipes:**
```json
{
  "error": "InsufficientRecipes",
  "message": "You need at least 7 favorite recipes in each category to generate a meal plan.",
  "details": {
    "appetizers": 5,
    "main_courses": 3,
    "desserts": 7
  },
  "action": {
    "label": "Add More Recipes",
    "url": "/recipes/new"
  }
}
```

**500 Internal Server Error - Algorithm Timeout:**
```json
{
  "error": "AlgorithmTimeout",
  "message": "Meal plan generation took too long. Please try again.",
  "status": 500
}
```

---

#### 2. Get Week Detail

Retrieves detailed information for a specific week including meal assignments and shopping list.

**Endpoint:** `GET /plan/week/:week_id`
**Authentication:** Required
**Rate Limit:** None

**Request:**
```bash
curl -X GET http://localhost:3000/plan/week/week_1 \
  -H "Cookie: session={JWT}"
```

**Response (200 OK):**
```json
{
  "week": {
    "id": "week_1",
    "start_date": "2025-10-28",
    "end_date": "2025-11-03",
    "status": "current",
    "is_locked": true,
    "meal_assignments": [ /* 21 assignments */ ],
    "shopping_list_id": "shopping_1"
  },
  "shopping_list": {
    "id": "shopping_1",
    "categories": [
      {
        "name": "Produce",
        "items": [
          {
            "ingredient_name": "Tomatoes",
            "quantity": 6,
            "unit": "whole",
            "from_recipe_ids": ["recipe_123", "recipe_456"]
          }
        ]
      }
    ]
  },
  "navigation": {
    "previous_week_id": null,
    "next_week_id": "week_2"
  }
}
```

**Error Responses:**

**404 Not Found:**
```json
{
  "error": "WeekNotFound",
  "message": "Week not found or does not belong to you.",
  "status": 404
}
```

**403 Forbidden:**
```json
{
  "error": "Forbidden",
  "message": "This week belongs to a different user.",
  "status": 403
}
```

---

#### 3. Regenerate Single Week

Regenerates meal assignments for a specific future week. Current and past weeks cannot be regenerated.

**Endpoint:** `POST /plan/week/:week_id/regenerate`
**Authentication:** Required
**Rate Limit:** 10 requests/hour

**Request:**
```bash
curl -X POST http://localhost:3000/plan/week/week_2/regenerate \
  -H "Cookie: session={JWT}"
```

**Response (200 OK):**
```json
{
  "week": {
    "id": "week_2",
    "start_date": "2025-11-04",
    "status": "future",
    "is_locked": false,
    "meal_assignments": [ /* regenerated assignments */ ],
    "shopping_list_id": "shopping_2"
  },
  "message": "Week regenerated successfully. Shopping list updated."
}
```

**Error Responses:**

**403 Forbidden - Locked Week:**
```json
{
  "error": "WeekLocked",
  "message": "Cannot regenerate current week. It is locked to prevent disrupting in-progress meals.",
  "status": 403
}
```

**400 Bad Request - Already Started:**
```json
{
  "error": "WeekAlreadyStarted",
  "message": "Cannot regenerate a week that has already started.",
  "status": 400
}
```

---

#### 4. Regenerate All Future Weeks

Regenerates all future weeks while preserving the current week. Requires explicit confirmation.

**Endpoint:** `POST /plan/regenerate-all-future`
**Authentication:** Required
**Rate Limit:** 10 requests/hour

**Request:**
```bash
curl -X POST http://localhost:3000/plan/regenerate-all-future \
  -H "Cookie: session={JWT}" \
  -H "Content-Type: application/json" \
  -d '{"confirmation": true}'
```

**Response (200 OK):**
```json
{
  "regenerated_weeks": 4,
  "preserved_current_week_id": "week_1",
  "first_future_week": {
    "id": "week_2",
    "start_date": "2025-11-04",
    "meal_assignments": [ /* list */ ]
  },
  "message": "All 4 future weeks regenerated successfully. Current week preserved."
}
```

**Error Responses:**

**400 Bad Request - Missing Confirmation:**
```json
{
  "error": "ConfirmationRequired",
  "message": "This action requires confirmation. Include { \"confirmation\": true } in request body.",
  "status": 400
}
```

---

#### 5. Update Meal Planning Preferences

Updates user's meal planning preferences which will be applied to future meal plan generations.

**Endpoint:** `PUT /profile/meal-planning-preferences`
**Authentication:** Required
**Rate Limit:** None

**Request:**
```bash
curl -X PUT http://localhost:3000/profile/meal-planning-preferences \
  -H "Cookie: session={JWT}" \
  -H "Content-Type: application/json" \
  -d '{
    "max_prep_time_weeknight": 30,
    "max_prep_time_weekend": 90,
    "avoid_consecutive_complex": true,
    "cuisine_variety_weight": 0.7
  }'
```

**Response (200 OK):**
```json
{
  "preferences": {
    "max_prep_time_weeknight": 30,
    "max_prep_time_weekend": 90,
    "avoid_consecutive_complex": true,
    "cuisine_variety_weight": 0.7
  },
  "message": "Meal planning preferences updated. Changes will apply to your next meal plan generation."
}
```

**Error Responses:**

**400 Bad Request - Validation Failed:**
```json
{
  "error": "ValidationFailed",
  "message": "Invalid preferences provided.",
  "details": {
    "max_prep_time_weeknight": "Must be greater than 0",
    "cuisine_variety_weight": "Must be between 0.0 and 1.0"
  },
  "status": 400
}
```

---

### Common Error Codes

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `InsufficientRecipes` | 400 | User has fewer than 7 favorite recipes per category |
| `AlgorithmTimeout` | 500 | Meal plan generation exceeded time limit |
| `WeekNotFound` | 404 | Requested week_id does not exist |
| `Forbidden` | 403 | User attempting to access another user's resource |
| `WeekLocked` | 403 | Attempting to regenerate current (locked) week |
| `WeekAlreadyStarted` | 400 | Attempting to regenerate past week |
| `ConfirmationRequired` | 400 | Bulk regeneration requires explicit confirmation |
| `ValidationFailed` | 400 | Request body validation failed |
| `Unauthorized` | 401 | JWT cookie missing or invalid |
| `RateLimitExceeded` | 429 | Too many requests in time window |

---

### Data Models

#### MealPlanningPreferences

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `max_prep_time_weeknight` | integer | > 0 | Maximum prep time for weeknight meals (minutes) |
| `max_prep_time_weekend` | integer | > 0 | Maximum prep time for weekend meals (minutes) |
| `avoid_consecutive_complex` | boolean | - | Avoid complex recipes on consecutive days |
| `cuisine_variety_weight` | float | 0.0 - 1.0 | Weight for cuisine variety (0.0 = ignore, 1.0 = maximize) |

#### WeekStatus

| Value | Description |
|-------|-------------|
| `current` | The current week (locked, cannot be regenerated) |
| `future` | Upcoming week (can be regenerated) |
| `past` | Previous week (cannot be regenerated) |

#### CourseType

| Value | Description |
|-------|-------------|
| `appetizer` | Starter or appetizer course |
| `main_course` | Main course |
| `dessert` | Dessert course |

#### Complexity

| Value | Description |
|-------|-------------|
| `simple` | Quick and easy recipe |
| `moderate` | Standard complexity |
| `complex` | Advanced recipe requiring more time/skill |

---

### Examples

#### Example 1: Generate Meal Plan and View First Week

```bash
# Step 1: Generate multi-week meal plan
curl -X POST http://localhost:3000/plan/generate-multi-week \
  -H "Cookie: session={JWT}" \
  -o meal_plan.json

# Step 2: Extract first week ID
WEEK_ID=$(jq -r '.first_week.id' meal_plan.json)

# Step 3: View week details
curl -X GET "http://localhost:3000/plan/week/$WEEK_ID" \
  -H "Cookie: session={JWT}"
```

#### Example 2: Update Preferences and Regenerate All Future Weeks

```bash
# Step 1: Update preferences
curl -X PUT http://localhost:3000/profile/meal-planning-preferences \
  -H "Cookie: session={JWT}" \
  -H "Content-Type: application/json" \
  -d '{
    "max_prep_time_weeknight": 20,
    "max_prep_time_weekend": 60,
    "avoid_consecutive_complex": true,
    "cuisine_variety_weight": 0.8
  }'

# Step 2: Regenerate all future weeks with new preferences
curl -X POST http://localhost:3000/plan/regenerate-all-future \
  -H "Cookie: session={JWT}" \
  -H "Content-Type: application/json" \
  -d '{"confirmation": true}'
```

---

### Performance Targets

| Route | P95 Latency Target |
|-------|-------------------|
| POST /plan/generate-multi-week | < 500ms (route overhead only, excluding algorithm) |
| GET /plan/week/:week_id | < 100ms |
| POST /plan/week/:week_id/regenerate | < 500ms (route overhead only) |
| POST /plan/regenerate-all-future | < 2000ms for 4 weeks (route overhead only) |
| PUT /profile/meal-planning-preferences | < 100ms |

**Note:** Algorithm execution time (3-5 seconds for meal plan generation) is excluded from route overhead measurements.

---

### Additional Resources

- **OpenAPI Specification:** See [meal-planning-routes-openapi.yaml](./meal-planning-routes-openapi.yaml) for machine-readable API definition
- **Testing Documentation:** See `/docs/testing/evento-test-pattern.md` for integration testing patterns
- **Technical Specification:** See `/docs/tech-spec-epic-8.md` for detailed architecture and design decisions

---

### Support

For API support or bug reports:
- **Email:** support@imkitchen.app
- **GitHub Issues:** https://github.com/imkitchen/imkitchen/issues
