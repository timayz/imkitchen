# Architecture Update: Enhanced Meal Planning System

**Date:** 2025-10-24
**Author:** Jonathan
**Status:** Design Approved - Ready for Implementation
**Version:** 2.0

---

## Executive Summary

This document outlines architectural changes to enhance the meal planning system with three major features:

1. **Multi-Week Meal Plan Generation**: Generate all possible weeks from favorite recipes with easy navigation
2. **Accompaniment Recipe Type**: Add pasta, rice, fries, etc. as optional sides to main courses
3. **User Preferences Integration**: Use dietary restrictions and time constraints in algorithm (cuisine preferences inferred from favorites)

These changes significantly improve user experience by providing better planning horizons, more realistic meal compositions, and personalized scheduling.

---

## Table of Contents

- [1. Multi-Week Meal Plan Generation](#1-multi-week-meal-plan-generation)
- [2. Accompaniment Recipe Type System](#2-accompaniment-recipe-type-system)
- [3. User Preferences Integration](#3-user-preferences-integration)
- [4. Database Schema Changes](#4-database-schema-changes)
- [5. Domain Model Updates](#5-domain-model-updates)
- [6. Algorithm Changes](#6-algorithm-changes)
- [7. API/Route Changes](#7-apiroute-changes)
- [8. UX/UI Impact](#8-uxui-impact)
- [9. Migration Strategy](#9-migration-strategy)
- [10. Implementation Roadmap](#10-implementation-roadmap)

---

## 1. Multi-Week Meal Plan Generation

### 1.1 Current Behavior

- Generates **only next week** meal plan (7 days starting Monday)
- Single active meal plan at a time
- Shopping list tied to single week
- Cannot regenerate current week (safety constraint)

**Limitation:** Users can't see long-term meal variety or plan grocery shopping beyond one week.

### 1.2 New Behavior

**Generate ALL Possible Weeks:**
- Algorithm calculates maximum weeks based on favorite recipe counts
- Generates all weeks simultaneously in single batch
- Each week gets its own shopping list
- Easy navigation between weeks (tabs/arrows)

**Week Locking:**
- Current week (today falls within Monday-Sunday range) becomes **locked**
- Locked weeks cannot be regenerated (prevents disrupting in-progress meals)
- Future weeks can be regenerated individually or all at once

**Maximum Weeks Calculation:**
```
max_weeks = min(
    5,  // Hard cap at 5 weeks
    count(favorite appetizers),
    count(favorite main courses),
    count(favorite desserts)
)
```

**Example:**
- 10 favorite appetizers
- 15 favorite main courses
- 8 favorite desserts
- **Result:** 5 weeks generated (capped at 5 weeks max)

**Rationale for 5-Week Cap:**
- Prevents excessive upfront computation for users with large recipe libraries
- 5 weeks provides ~1 month planning horizon with flexibility
- Balances long-term planning with manageable UI complexity
- Users can regenerate after consuming initial weeks

### 1.3 Data Model Changes

```rust
pub struct MultiWeekMealPlan {
    user_id: UserId,
    generation_batch_id: String,      // Links weeks generated together
    generated_weeks: Vec<WeekMealPlan>,
    rotation_state: RotationState,
}

pub struct WeekMealPlan {
    id: String,
    user_id: UserId,
    start_date: Date,                  // Monday (ISO 8601)
    end_date: Date,                    // Sunday
    status: WeekStatus,                // future | current | past | archived
    is_locked: bool,                   // True if week started
    generation_batch_id: String,       // Links to batch
    meal_assignments: Vec<MealAssignment>, // 21 assignments (7 days × 3 courses)
    shopping_list_id: String,
    created_at: DateTime,
}

pub enum WeekStatus {
    Future,    // Week hasn't started yet
    Current,   // Today falls within week
    Past,      // Week completed
    Archived,  // User manually archived
}

pub struct RotationState {
    used_main_course_ids: Vec<RecipeId>,     // MUST be unique across all weeks
    used_appetizer_ids: Vec<RecipeId>,       // Can repeat after all used once
    used_dessert_ids: Vec<RecipeId>,         // Can repeat after all used once
    cuisine_usage_count: HashMap<Cuisine, u32>, // Track variety
    last_complex_meal_date: Option<Date>,    // Avoid consecutive complex
}
```

### 1.4 Events

```rust
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct MultiWeekMealPlanGenerated {
    generation_batch_id: String,
    user_id: UserId,
    weeks: Vec<WeekMealPlanData>,
    rotation_state: RotationState,
    generated_at: DateTime,
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct SingleWeekRegenerated {
    week_id: String,
    week_start_date: Date,
    meal_assignments: Vec<MealAssignment>,
    updated_rotation_state: RotationState,
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct AllFutureWeeksRegenerated {
    generation_batch_id: String,
    user_id: UserId,
    weeks: Vec<WeekMealPlanData>,
    preserved_current_week_id: Option<String>, // Current week kept intact
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct ShoppingListGenerated {
    shopping_list_id: String,
    meal_plan_id: String,
    week_start_date: Date,
    categories: Vec<ShoppingCategory>,
}
```

### 1.5 Algorithm: Multi-Week Generation

```rust
pub async fn generate_multi_week_meal_plans(
    user_id: UserId,
    favorite_recipes: Vec<Recipe>,
    preferences: UserPreferences,
) -> Result<MultiWeekMealPlan, Error> {
    // 1. Filter recipes by dietary restrictions
    let compatible_recipes = filter_by_dietary_restrictions(
        favorite_recipes,
        &preferences.dietary_restrictions
    );

    // 2. Calculate maximum weeks
    let appetizers = compatible_recipes.iter()
        .filter(|r| r.recipe_type == RecipeType::Appetizer)
        .collect::<Vec<_>>();
    let main_courses = compatible_recipes.iter()
        .filter(|r| r.recipe_type == RecipeType::MainCourse)
        .collect::<Vec<_>>();
    let desserts = compatible_recipes.iter()
        .filter(|r| r.recipe_type == RecipeType::Dessert)
        .collect::<Vec<_>>();

    let max_weeks = std::cmp::min(
        5,  // Hard cap at 5 weeks
        std::cmp::min(
            std::cmp::min(appetizers.len(), main_courses.len()),
            desserts.len()
        )
    );

    if max_weeks < 1 {
        return Err(Error::InsufficientRecipes {
            appetizers: appetizers.len(),
            main_courses: main_courses.len(),
            desserts: desserts.len(),
        });
    }

    // 3. Initialize rotation state
    let mut rotation_state = RotationState::new();
    let generation_batch_id = Uuid::new_v4().to_string();

    // 4. Generate each week
    let mut generated_weeks = Vec::new();
    let next_monday = get_next_monday();

    for week_offset in 0..max_weeks {
        let week_start = next_monday + Duration::weeks(week_offset as i64);
        let week_end = week_start + Duration::days(6);

        let week = generate_single_week(
            week_start,
            week_end,
            &appetizers,
            &main_courses,
            &desserts,
            &preferences,
            &mut rotation_state,
            generation_batch_id.clone(),
        ).await?;

        generated_weeks.push(week);
    }

    // 5. Generate shopping lists for all weeks
    for week in &mut generated_weeks {
        let shopping_list = generate_shopping_list_for_week(week).await?;
        week.shopping_list_id = shopping_list.id;
    }

    Ok(MultiWeekMealPlan {
        user_id,
        generation_batch_id,
        generated_weeks,
        rotation_state,
    })
}

fn generate_single_week(
    week_start: Date,
    week_end: Date,
    appetizers: &[&Recipe],
    main_courses: &[&Recipe],
    desserts: &[&Recipe],
    preferences: &UserPreferences,
    rotation_state: &mut RotationState,
    generation_batch_id: String,
) -> Result<WeekMealPlan, Error> {
    let mut meal_assignments = Vec::new();

    // Generate 7 days × 3 courses = 21 assignments
    for day_offset in 0..7 {
        let date = week_start + Duration::days(day_offset);
        let day_of_week = date.weekday(); // Mon, Tue, Wed, etc.

        // Appetizer
        let appetizer = select_appetizer(appetizers, rotation_state);
        meal_assignments.push(MealAssignment {
            id: Uuid::new_v4().to_string(),
            date,
            course_type: CourseType::Appetizer,
            recipe_id: appetizer.id.clone(),
            accompaniment_recipe_id: None,
            prep_required: appetizer.advance_prep_text.is_some(),
        });
        rotation_state.mark_used_appetizer(&appetizer.id);

        // Main Course (with potential accompaniment)
        let main_course = select_main_course_with_preferences(
            main_courses,
            day_of_week,
            preferences,
            rotation_state,
        );
        let accompaniment_id = if main_course.accepts_accompaniment {
            Some(select_accompaniment(preferences).id)
        } else {
            None
        };
        meal_assignments.push(MealAssignment {
            id: Uuid::new_v4().to_string(),
            date,
            course_type: CourseType::MainCourse,
            recipe_id: main_course.id.clone(),
            accompaniment_recipe_id,
            prep_required: main_course.advance_prep_text.is_some(),
        });
        rotation_state.mark_used_main_course(&main_course.id); // UNIQUE across all weeks

        // Dessert
        let dessert = select_dessert(desserts, rotation_state);
        meal_assignments.push(MealAssignment {
            id: Uuid::new_v4().to_string(),
            date,
            course_type: CourseType::Dessert,
            recipe_id: dessert.id.clone(),
            accompaniment_recipe_id: None,
            prep_required: dessert.advance_prep_text.is_some(),
        });
        rotation_state.mark_used_dessert(&dessert.id);
    }

    Ok(WeekMealPlan {
        id: Uuid::new_v4().to_string(),
        user_id: preferences.user_id,
        start_date: week_start,
        end_date: week_end,
        status: WeekStatus::Future,
        is_locked: false,
        generation_batch_id,
        meal_assignments,
        shopping_list_id: String::new(), // Set later
        created_at: Utc::now(),
    })
}
```

### 1.6 Shopping List Generation

**Rule:** Each week gets its own shopping list, generated from that week's meal assignments.

```rust
async fn generate_shopping_list_for_week(
    week: &WeekMealPlan
) -> Result<ShoppingList, Error> {
    let mut all_ingredients = Vec::new();

    for assignment in &week.meal_assignments {
        // Load main recipe
        let recipe = load_recipe(&assignment.recipe_id).await?;
        all_ingredients.extend(recipe.ingredients.clone());

        // Load accompaniment if present
        if let Some(accompaniment_id) = &assignment.accompaniment_recipe_id {
            let accompaniment = load_recipe(accompaniment_id).await?;
            all_ingredients.extend(accompaniment.ingredients.clone());
        }
    }

    // Aggregate and categorize
    let categories = aggregate_and_categorize_ingredients(all_ingredients);

    Ok(ShoppingList {
        id: Uuid::new_v4().to_string(),
        meal_plan_id: week.id.clone(),
        week_start_date: week.start_date,
        categories,
        generated_at: Utc::now(),
    })
}
```

**Regeneration Cascade:**
- When user clicks "Regenerate This Week" → regenerate that week's meal plan + shopping list
- When user clicks "Regenerate All Future Weeks":
  - Show confirmation dialog: "This will regenerate all future weeks (X weeks). Continue?"
  - If confirmed → regenerate all unlocked weeks + all shopping lists
  - Current week (locked) preserved

---

## 2. Accompaniment Recipe Type System

### 2.1 Current Behavior

- Recipe types: `appetizer`, `main_course`, `dessert` only
- Single recipe per course slot
- No concept of side dishes or pairings

**Limitation:** Unrealistic meal compositions (e.g., chicken tikka masala without rice)

### 2.2 New Behavior

**New Recipe Type:** `accompaniment`

**Accompaniment Categories:**
- Pasta (spaghetti, penne, linguine, etc.)
- Rice (white rice, fried rice, risotto, etc.)
- Fries (french fries, potato wedges, etc.)
- Salad (side salads, coleslaw, etc.)
- Bread (garlic bread, dinner rolls, etc.)
- Vegetable (roasted vegetables, steamed greens, etc.)
- Other

**Main Course Configuration:**
- Each main course recipe **optionally** specifies `accepts_accompaniment: bool`
- If true, optionally specify `preferred_accompaniments: Vec<AccompanimentCategory>`
- Algorithm selects accompaniment only if main course accepts it
- **Note:** Accompaniments are always optional (no "required" setting) - respects recipe creator's intent

**Rotation Rules:**
- Main courses MUST be unique across all generated weeks (existing rule)
- Accompaniments CAN repeat (not subject to uniqueness constraint)

### 2.3 Data Model Changes

```rust
// Extended recipe type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecipeType {
    Appetizer,
    MainCourse,
    Dessert,
    Accompaniment,  // NEW
}

// New accompaniment category enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AccompanimentCategory {
    Pasta,
    Rice,
    Fries,
    Salad,
    Bread,
    Vegetable,
    Other,
}

// Recipe struct extension
pub struct Recipe {
    pub id: RecipeId,
    pub user_id: UserId,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<InstructionStep>,
    pub prep_time_min: u32,
    pub cook_time_min: u32,
    pub advance_prep_text: Option<String>,
    pub serving_size: u32,
    pub recipe_type: RecipeType,
    pub complexity: Complexity,
    pub is_favorite: bool,
    pub is_shared: bool,

    // NEW FIELDS for accompaniments
    pub accepts_accompaniment: bool,  // True if main course can have side
    pub preferred_accompaniments: Vec<AccompanimentCategory>, // Filter
    pub accompaniment_category: Option<AccompanimentCategory>, // For accompaniment-type recipes

    // Existing fields
    pub cuisine: Option<Cuisine>,
    pub dietary_tags: Vec<DietaryTag>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// Meal assignment extension
pub struct MealAssignment {
    pub id: String,
    pub meal_plan_id: String,
    pub date: Date,
    pub course_type: CourseType,
    pub recipe_id: RecipeId,

    // NEW FIELD
    pub accompaniment_recipe_id: Option<RecipeId>,

    pub prep_required: bool,
}
```

### 2.4 Events

```rust
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct RecipeCreated {
    id: RecipeId,
    title: String,
    recipe_type: RecipeType,
    accepts_accompaniment: bool,
    preferred_accompaniments: Vec<AccompanimentCategory>,
    accompaniment_category: Option<AccompanimentCategory>,
    // ... other fields
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct RecipeAccompanimentSettingsUpdated {
    recipe_id: RecipeId,
    accepts_accompaniment: bool,
    preferred_accompaniments: Vec<AccompanimentCategory>,
}
```

### 2.5 Algorithm: Accompaniment Selection

```rust
async fn assign_main_course_with_accompaniment(
    date: Date,
    day_of_week: DayOfWeek,
    available_main_courses: &[Recipe],
    available_accompaniments: &[Recipe],
    preferences: &UserPreferences,
    rotation_state: &mut RotationState,
) -> MealAssignment {
    // 1. Select main course with preference-aware logic
    let main_course = select_main_course_with_preferences(
        available_main_courses,
        day_of_week,
        preferences,
        rotation_state,
    );

    // 2. Check if main course accepts accompaniment
    let accompaniment_id = if main_course.accepts_accompaniment {
        // 3. Filter accompaniments by preference
        let filtered_accompaniments = if !main_course.preferred_accompaniments.is_empty() {
            available_accompaniments.iter()
                .filter(|acc| {
                    acc.accompaniment_category
                        .as_ref()
                        .map(|cat| main_course.preferred_accompaniments.contains(cat))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>()
        } else {
            // No preference, use all accompaniments
            available_accompaniments.iter().collect()
        };

        // 4. Select random accompaniment (can repeat, not tracked in rotation)
        if !filtered_accompaniments.is_empty() {
            let selected = select_random(&filtered_accompaniments);
            Some(selected.id.clone())
        } else {
            None
        }
    } else {
        None
    };

    // 5. Mark main course as used (UNIQUE across all weeks)
    rotation_state.mark_used_main_course(&main_course.id);

    MealAssignment {
        id: Uuid::new_v4().to_string(),
        meal_plan_id: String::new(), // Set by caller
        date,
        course_type: CourseType::MainCourse,
        recipe_id: main_course.id,
        accompaniment_recipe_id,
        prep_required: main_course.advance_prep_text.is_some(),
    }
}

fn select_random<T>(items: &[T]) -> &T {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    items.choose(&mut rng).unwrap()
}
```

### 2.6 Shopping List Impact

Shopping lists now include both main recipe AND accompaniment ingredients:

```rust
for assignment in &week.meal_assignments {
    // Main recipe ingredients
    let recipe = load_recipe(&assignment.recipe_id).await?;
    all_ingredients.extend(recipe.ingredients.clone());

    // Accompaniment ingredients (if present)
    if let Some(accompaniment_id) = &assignment.accompaniment_recipe_id {
        let accompaniment = load_recipe(accompaniment_id).await?;
        all_ingredients.extend(accompaniment.ingredients.clone());
    }
}
```

---

## 3. User Preferences Integration

### 3.1 Current Behavior

- User profile stores preferences: dietary restrictions, household size, skill level, weeknight availability
- **Preferences NOT actively used in meal planning algorithm**

**Limitation:** Generated meal plans may not match user's real-world constraints (time, skill, dietary needs)

### 3.2 New Behavior

**Algorithm Considers:**
- **Time Constraints**: Max prep time weeknights vs weekends
- **Skill Level**: Beginner → simple only, Intermediate → simple+moderate, Advanced → all
- **Dietary Restrictions**: Filter out incompatible recipes before generation
- **Cuisine Variety**: Ensure variety across weeks (preferences implicit in favorited recipes)
- **Complexity Spacing**: Avoid consecutive complex meals

**Design Notes:**

1. **Cuisine Preferences:** We do NOT store an explicit `preferred_cuisines` field. Instead, cuisine preferences are **inferred from the user's favorite recipe selection**. If a user favorites 5 Italian recipes and 2 Mexican recipes, the algorithm naturally reflects this preference ratio. The `cuisine_variety_weight` slider controls how aggressively the algorithm spreads variety across cuisines versus allowing natural repetition based on favorites.

2. **Advance Prep Timing:** We do NOT store user preferences for advance prep willingness. Advance prep requirements (e.g., "marinate 4 hours", "rest dough overnight") are **recipe characteristics** stored in the recipe's `advance_prep_text` field. The algorithm schedules the meal appropriately and sends prep reminders based on the recipe's specific requirements. Users receive notifications at the right time, not based on a preference setting.

### 3.3 Data Model Changes

```rust
pub struct UserPreferences {
    pub user_id: UserId,

    // Existing preferences
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub household_size: u32,
    pub skill_level: SkillLevel,
    pub weeknight_availability: TimeRange,

    // NEW PREFERENCES
    pub max_prep_time_weeknight: u32,      // Minutes (default: 30)
    pub max_prep_time_weekend: u32,        // Minutes (default: 90)
    pub avoid_consecutive_complex: bool,   // Default: true
    pub cuisine_variety_weight: f32,       // 0.0-1.0, higher = more variety (inferred from favorites)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DietaryRestriction {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    Halal,
    Kosher,
    Custom(String),  // User-defined allergens
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillLevel {
    Beginner,      // Only simple recipes
    Intermediate,  // Simple + moderate
    Advanced,      // All complexity levels
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Cuisine {
    Italian,
    Indian,
    Mexican,
    Chinese,
    Japanese,
    French,
    American,
    Mediterranean,
    Thai,
    Korean,
    Vietnamese,
    Greek,
    Spanish,
    Custom(String),  // User-defined cuisines (e.g., "Fusion", "Regional Brazilian")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DietaryTag {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    Halal,
    Kosher,
}
```

### 3.4 Events

```rust
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct UserMealPlanningPreferencesUpdated {
    user_id: UserId,
    max_prep_time_weeknight: u32,
    max_prep_time_weekend: u32,
    avoid_consecutive_complex: bool,
    cuisine_variety_weight: f32,
    updated_at: DateTime,
}
```

### 3.5 Algorithm: Preference Integration

```rust
// Step 1: Filter by dietary restrictions BEFORE generation
fn filter_by_dietary_restrictions(
    recipes: Vec<Recipe>,
    restrictions: &[DietaryRestriction],
) -> Vec<Recipe> {
    recipes.into_iter()
        .filter(|recipe| {
            restrictions.iter().all(|restriction| {
                match restriction {
                    DietaryRestriction::Vegetarian => recipe.dietary_tags.contains(&DietaryTag::Vegetarian),
                    DietaryRestriction::Vegan => recipe.dietary_tags.contains(&DietaryTag::Vegan),
                    DietaryRestriction::GlutenFree => recipe.dietary_tags.contains(&DietaryTag::GlutenFree),
                    DietaryRestriction::DairyFree => recipe.dietary_tags.contains(&DietaryTag::DairyFree),
                    DietaryRestriction::NutFree => recipe.dietary_tags.contains(&DietaryTag::NutFree),
                    DietaryRestriction::Halal => recipe.dietary_tags.contains(&DietaryTag::Halal),
                    DietaryRestriction::Kosher => recipe.dietary_tags.contains(&DietaryTag::Kosher),
                    DietaryRestriction::Custom(allergen) => {
                        // Check ingredients for custom allergen
                        !recipe.ingredients.iter().any(|ing| ing.name.to_lowercase().contains(&allergen.to_lowercase()))
                    }
                }
            })
        })
        .collect()
}

// Step 2: Select main course with time/skill/cuisine preferences
fn select_main_course_with_preferences(
    available_main_courses: &[Recipe],
    day_of_week: DayOfWeek,
    preferences: &UserPreferences,
    rotation_state: &RotationState,
) -> Recipe {
    // 1. Determine time constraint (weeknight vs weekend)
    let is_weeknight = matches!(day_of_week, Mon | Tue | Wed | Thu | Fri);
    let max_prep_time = if is_weeknight {
        preferences.max_prep_time_weeknight
    } else {
        preferences.max_prep_time_weekend
    };

    // 2. Filter by time constraint
    let time_compatible = available_main_courses.iter()
        .filter(|r| r.prep_time_min + r.cook_time_min <= max_prep_time)
        .collect::<Vec<_>>();

    // 3. Filter by skill level
    let skill_compatible = match preferences.skill_level {
        SkillLevel::Beginner => {
            time_compatible.into_iter()
                .filter(|r| matches!(r.complexity, Complexity::Simple))
                .collect::<Vec<_>>()
        }
        SkillLevel::Intermediate => {
            time_compatible.into_iter()
                .filter(|r| !matches!(r.complexity, Complexity::Complex))
                .collect::<Vec<_>>()
        }
        SkillLevel::Advanced => time_compatible,
    };

    // 4. Filter by complexity spacing (if preference set)
    let complexity_compatible = if preferences.avoid_consecutive_complex {
        if let Some(last_complex_date) = rotation_state.last_complex_meal_date {
            let yesterday = Utc::now().date_naive() - Duration::days(1);
            if last_complex_date == yesterday {
                // Avoid complex today
                skill_compatible.into_iter()
                    .filter(|r| !matches!(r.complexity, Complexity::Complex))
                    .collect::<Vec<_>>()
            } else {
                skill_compatible
            }
        } else {
            skill_compatible
        }
    } else {
        skill_compatible
    };

    // 5. Score recipes by cuisine variety (from favorited recipes)
    let mut scored_recipes = complexity_compatible.iter()
        .map(|recipe| {
            let mut score = 0.0;

            // Apply variety weighting (penalize recently used cuisines)
            // Cuisine preferences are implicit in user's favorite selection
            if let Some(cuisine) = &recipe.cuisine {
                let cuisine_usage = rotation_state.cuisine_usage_count.get(cuisine).unwrap_or(&0);
                score += preferences.cuisine_variety_weight * (1.0 / (*cuisine_usage as f32 + 1.0));
            }

            (*recipe, score)
        })
        .collect::<Vec<_>>();

    // 6. Sort by score descending, pick highest
    scored_recipes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if scored_recipes.is_empty() {
        panic!("No compatible recipes found for preferences!");
    }

    scored_recipes[0].0.clone()
}
```

---

## 4. Database Schema Changes

See complete SQL migration in [Migration Strategy](#9-migration-strategy) section.

**Summary of Schema Changes:**

### 4.1 `meal_plans` Table

```sql
ALTER TABLE meal_plans ADD COLUMN end_date TEXT NOT NULL;
ALTER TABLE meal_plans ADD COLUMN is_locked BOOLEAN DEFAULT FALSE;
ALTER TABLE meal_plans ADD COLUMN generation_batch_id TEXT;
```

**New Indexes:**
```sql
CREATE INDEX idx_meal_plans_user_batch ON meal_plans(user_id, generation_batch_id);
CREATE INDEX idx_meal_plans_status ON meal_plans(user_id, status);
CREATE INDEX idx_meal_plans_dates ON meal_plans(start_date, end_date);
```

### 4.2 `recipes` Table

```sql
-- Accompaniment support
ALTER TABLE recipes ADD COLUMN accepts_accompaniment BOOLEAN DEFAULT FALSE;
ALTER TABLE recipes ADD COLUMN preferred_accompaniments TEXT; -- JSON array
ALTER TABLE recipes ADD COLUMN accompaniment_category TEXT;

-- Cuisine and dietary tags
ALTER TABLE recipes ADD COLUMN cuisine TEXT;
ALTER TABLE recipes ADD COLUMN dietary_tags TEXT; -- JSON array
```

**New Indexes:**
```sql
CREATE INDEX idx_recipes_accompaniment_type ON recipes(recipe_type) WHERE recipe_type = 'accompaniment';
CREATE INDEX idx_recipes_cuisine ON recipes(cuisine);
```

### 4.3 `meal_assignments` Table

```sql
ALTER TABLE meal_assignments ADD COLUMN accompaniment_recipe_id TEXT;
```

**New Index:**
```sql
CREATE INDEX idx_meal_assignments_accompaniment ON meal_assignments(accompaniment_recipe_id);
```

### 4.4 `users` Table

```sql
-- Meal planning preferences
ALTER TABLE users ADD COLUMN max_prep_time_weeknight INTEGER DEFAULT 30;
ALTER TABLE users ADD COLUMN max_prep_time_weekend INTEGER DEFAULT 90;
ALTER TABLE users ADD COLUMN avoid_consecutive_complex BOOLEAN DEFAULT TRUE;
ALTER TABLE users ADD COLUMN cuisine_variety_weight REAL DEFAULT 0.7;
```

### 4.5 New Table: `meal_plan_rotation_state`

```sql
CREATE TABLE meal_plan_rotation_state (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  generation_batch_id TEXT NOT NULL,
  used_main_course_ids TEXT NOT NULL, -- JSON array
  used_appetizer_ids TEXT NOT NULL,   -- JSON array
  used_dessert_ids TEXT NOT NULL,     -- JSON array
  cuisine_usage_count TEXT NOT NULL,  -- JSON object
  last_complex_meal_date TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
```

---

## 5. Domain Model Updates

### 5.1 Crate: `meal_planning`

**Files to Update/Create:**

```
crates/meal_planning/
├── src/
│   ├── lib.rs
│   ├── aggregate.rs            # MealPlanAggregate (updated)
│   ├── commands.rs             # New commands
│   ├── events.rs               # New events
│   ├── algorithm.rs            # Multi-week algorithm (major update)
│   ├── rotation.rs             # Rotation state logic (updated)
│   ├── preferences.rs          # NEW: User preference filtering
│   ├── accompaniments.rs       # NEW: Accompaniment selection logic
│   ├── read_model.rs           # Updated projections
│   └── error.rs
```

**New Commands:**

```rust
pub struct GenerateMultiWeekMealPlans {
    pub user_id: UserId,
    pub start_date: Date, // Next Monday
}

pub struct RegenerateSingleWeek {
    pub user_id: UserId,
    pub week_id: String,
    pub week_start_date: Date,
}

pub struct RegenerateAllFutureWeeks {
    pub user_id: UserId,
}

pub struct UpdateMealPlanningPreferences {
    pub user_id: UserId,
    pub max_prep_time_weeknight: u32,
    pub max_prep_time_weekend: u32,
    pub avoid_consecutive_complex: bool,
    pub cuisine_variety_weight: f32,
}
```

### 5.2 Crate: `recipe`

**Files to Update:**

```
crates/recipe/
├── src/
│   ├── aggregate.rs            # Recipe aggregate (updated)
│   ├── commands.rs             # CreateRecipe, UpdateRecipe (add new fields)
│   ├── events.rs               # RecipeCreated, RecipeUpdated (add new fields)
│   └── read_model.rs           # Updated projections
```

**Updated Events:**

```rust
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct RecipeCreated {
    // Existing fields...

    // NEW
    accepts_accompaniment: bool,
    preferred_accompaniments: Vec<AccompanimentCategory>,
    accompaniment_category: Option<AccompanimentCategory>,
    cuisine: Option<Cuisine>,
    dietary_tags: Vec<DietaryTag>,
}
```

### 5.3 Crate: `user`

**Files to Update:**

```
crates/user/
├── src/
│   ├── aggregate.rs            # UserAggregate (updated)
│   ├── commands.rs             # UpdateMealPlanningPreferences (NEW)
│   ├── events.rs               # UserMealPlanningPreferencesUpdated (NEW)
│   └── read_model.rs           # Updated user projections
```

---

## 6. Algorithm Changes

### 6.1 High-Level Flow

```
User clicks "Generate Meal Plan"
    ↓
Load user preferences
    ↓
Load favorite recipes (appetizers, main courses, desserts, accompaniments)
    ↓
Filter recipes by dietary restrictions
    ↓
Calculate max_weeks = min(appetizers, main_courses, desserts)
    ↓
FOR each week (0 to max_weeks):
    ↓
    FOR each day (Monday to Sunday):
        ↓
        Assign appetizer (rotation logic)
        ↓
        Assign main course (preference-aware + optional accompaniment)
        ↓
        Assign dessert (rotation logic)
    ↓
    Generate shopping list for week
    ↓
NEXT week
    ↓
Persist all weeks + shopping lists (evento events)
    ↓
Return MultiWeekMealPlan
```

### 6.2 Key Algorithm Functions

**Function 1: Main Course Selection with Preferences**

```rust
fn select_main_course_with_preferences(
    available: &[Recipe],
    day_of_week: DayOfWeek,
    preferences: &UserPreferences,
    rotation_state: &RotationState,
) -> Recipe {
    // Filter by time constraint
    // Filter by skill level
    // Filter by complexity spacing
    // Score by cuisine preference + variety
    // Return highest scored
}
```

**Function 2: Accompaniment Selection**

```rust
fn select_accompaniment(
    main_course: &Recipe,
    available_accompaniments: &[Recipe],
) -> Option<Recipe> {
    if !main_course.accepts_accompaniment {
        return None;
    }

    // Filter by preferred categories
    let filtered = if !main_course.preferred_accompaniments.is_empty() {
        available_accompaniments.iter()
            .filter(|acc| main_course.preferred_accompaniments.contains(&acc.accompaniment_category?))
            .collect()
    } else {
        available_accompaniments
    };

    // Random selection (accompaniments can repeat)
    Some(select_random(filtered))
}
```

**Function 3: Dietary Filter**

```rust
fn filter_by_dietary_restrictions(
    recipes: Vec<Recipe>,
    restrictions: &[DietaryRestriction],
) -> Vec<Recipe> {
    recipes.into_iter()
        .filter(|r| restrictions.iter().all(|restriction| r.is_compatible_with(restriction)))
        .collect()
}
```

**Function 4: Rotation State Management**

```rust
impl RotationState {
    // Main courses: MUST be unique across all weeks
    pub fn mark_used_main_course(&mut self, id: &RecipeId) {
        self.used_main_course_ids.push(id.clone());
    }

    pub fn is_main_course_used(&self, id: &RecipeId) -> bool {
        self.used_main_course_ids.contains(id)
    }

    // Appetizers/Desserts: Can repeat after all used once
    pub fn mark_used_appetizer(&mut self, id: &RecipeId) {
        self.used_appetizer_ids.push(id.clone());
    }

    pub fn reset_appetizers_if_all_used(&mut self, total_count: usize) {
        if self.used_appetizer_ids.len() >= total_count {
            self.used_appetizer_ids.clear();
        }
    }

    // Similar for desserts...

    // Cuisine variety tracking
    pub fn increment_cuisine_usage(&mut self, cuisine: &Cuisine) {
        *self.cuisine_usage_count.entry(cuisine.clone()).or_insert(0) += 1;
    }
}
```

---

## 7. API/Route Changes

### 7.1 New Routes

```rust
// Generate multi-week meal plans
POST /plan/generate-multi-week
  → Generates all possible weeks
  → Returns first week view, with navigation to other weeks

// Regenerate single week
POST /plan/week/:week_id/regenerate
  → Regenerates specific week (if not locked)
  → Updates shopping list for that week

// Regenerate all future weeks
POST /plan/regenerate-all-future
  → **Requires confirmation** (prevent accidental regeneration)
  → Preserves current week (locked)
  → Regenerates all future weeks
  → Recalculates all shopping lists
  → Returns confirmation dialog HTML or executes if confirmed

// Navigate between weeks
GET /plan/week/:week_id
  → Shows calendar view for specific week
  → Includes shopping list link

// Update meal planning preferences
PUT /profile/meal-planning-preferences
  → Updates user preferences
  → Triggers re-validation of existing plans (optional)

// Get shopping list for specific week
GET /shopping/week/:week_id
  → Returns shopping list for week
  → Includes checkoff state
```

### 7.2 Updated Routes

```rust
// Existing routes with new behavior:

GET /plan
  → Now shows multi-week view (tabs or carousel)
  → Displays current week by default
  → Navigation arrows/tabs to other weeks

GET /shopping
  → Defaults to current week's shopping list
  → Week selector dropdown to view other weeks

POST /plan/meal/:id/replace
  → When replacing meal, checks if accompaniment needed
  → Updates shopping list for that week
```

### 7.3 Response Format Changes

**Multi-Week Meal Plan Response:**

```json
{
  "generation_batch_id": "uuid",
  "weeks": [
    {
      "id": "week-uuid-1",
      "start_date": "2025-10-28",
      "end_date": "2025-11-03",
      "status": "current",
      "is_locked": true,
      "meal_assignments": [
        {
          "id": "assignment-uuid",
          "date": "2025-10-28",
          "course_type": "main_course",
          "recipe": {
            "id": "recipe-uuid",
            "title": "Chicken Tikka Masala",
            "image_url": "/recipes/123/image.jpg",
            "prep_time_min": 20,
            "cook_time_min": 30,
            "complexity": "moderate"
          },
          "accompaniment": {
            "id": "recipe-uuid-2",
            "title": "Basmati Rice",
            "accompaniment_category": "rice"
          },
          "prep_required": true
        }
        // ... 20 more assignments
      ],
      "shopping_list_id": "shopping-uuid-1"
    }
    // ... more weeks
  ],
  "max_weeks_possible": 8,
  "current_week_index": 0
}
```

---

## 8. UX/UI Impact

### 8.1 Meal Planning Calendar View

**Desktop - Multi-Week Tabs:**

```
┌─────────────────────────────────────────────────────────────┐
│  ← Week 1     Week 2     Week 3     Week 4     Week 5 →    │
│    Oct 28     Nov 4      Nov 11     Nov 18     Nov 25       │
│    (Current   (Future)   (Future)   (Future)   (Future)     │
│     🔒)                                                      │
├─────────────────────────────────────────────────────────────┤
│  Mon     Tue     Wed     Thu     Fri     Sat     Sun        │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐         │
│  │ A  │ │ A  │ │ A  │ │ A  │ │ A  │ │ A  │ │ A  │         │
│  │ M+R│ │ M+P│ │ M  │ │ M+S│ │ M+F│ │ M+P│ │ M+R│         │
│  │ D  │ │ D  │ │ D  │ │ D  │ │ D  │ │ D  │ │ D  │         │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘ └────┘         │
│                                                              │
│  A = Appetizer, M = Main Course, D = Dessert               │
│  +R = Rice, +P = Pasta, +S = Salad, +F = Fries            │
├─────────────────────────────────────────────────────────────┤
│  🔒 Current Week (locked - cannot regenerate)              │
│  🔄 Regenerate This Week  🔄 Regenerate All Future Weeks  │
└─────────────────────────────────────────────────────────────┘
```

**Regenerate All Confirmation Dialog (Modal):**

```
┌─────────────────────────────────────────────────┐
│  ⚠️  Regenerate All Future Weeks?               │
├─────────────────────────────────────────────────┤
│  This will regenerate 4 future weeks with new   │
│  meal assignments. Your current week will be    │
│  preserved.                                      │
│                                                  │
│  All shopping lists will be updated.            │
│                                                  │
│  [Cancel]              [Regenerate All Weeks]   │
└─────────────────────────────────────────────────┘
```

**Mobile - Week Carousel:**

```
┌─────────────────────────┐
│  ← Week 2 of 5 →        │
│    Nov 4 - Nov 10       │
├─────────────────────────┤
│  Monday, Nov 4          │
│  ┌──────────────────┐   │
│  │ Appetizer        │   │
│  │ Caesar Salad     │   │
│  └──────────────────┘   │
│  ┌──────────────────┐   │
│  │ Main Course      │   │
│  │ Chicken Tikka    │   │
│  │ + Basmati Rice   │   │ ← Accompaniment shown
│  └──────────────────┘   │
│  ┌──────────────────┐   │
│  │ Dessert          │   │
│  │ Tiramisu         │   │
│  └──────────────────┘   │
│                         │
│  [View Shopping List]   │
│  [Regenerate This Week] │
└─────────────────────────┘
```

### 8.2 Recipe Creation Form

**Main Course Accompaniment Settings:**

```
┌─────────────────────────────────────────────┐
│ Recipe Type:  ● Main Course                 │
│                                              │
│ ☑ This dish accepts an accompaniment       │ ← NEW
│                                              │
│ Preferred accompaniments (optional):        │ ← NEW
│  ☑ Pasta    ☑ Rice    ☐ Fries              │
│  ☑ Salad    ☐ Bread   ☐ Vegetables         │
│  ☐ Other                                    │
└─────────────────────────────────────────────┘
```

**Accompaniment Recipe Type:**

```
┌─────────────────────────────────────────────┐
│ Recipe Type:  ● Accompaniment               │ ← NEW option
│                                              │
│ Accompaniment Category:                     │
│  ○ Pasta                                    │
│  ● Rice                                     │
│  ○ Fries                                    │
│  ○ Salad                                    │
│  ○ Bread                                    │
│  ○ Vegetables                               │
│  ○ Other                                    │
└─────────────────────────────────────────────┘
```

**Cuisine Selection (All Recipe Types):**

```
┌─────────────────────────────────────────────┐
│ Cuisine:                                     │
│  ○ Italian    ○ Indian    ○ Mexican         │
│  ○ Chinese    ○ Japanese  ○ French          │
│  ○ American   ○ Mediterranean  ○ Thai       │
│  ○ Korean     ○ Vietnamese  ○ Greek         │
│  ○ Spanish                                   │
│  ● Custom: [Fusion           ]              │ ← Allow custom input
│                                              │
│  (Used for cuisine variety in meal planning)│
└─────────────────────────────────────────────┘
```

### 8.3 User Profile - Meal Planning Preferences

```
┌─────────────────────────────────────────────────┐
│ Meal Planning Preferences                        │
├─────────────────────────────────────────────────┤
│                                                  │
│ Time Constraints:                                │
│  Weeknights (Mon-Fri):  [30] minutes max        │
│  Weekends (Sat-Sun):    [90] minutes max        │
│                                                  │
│ Complexity Management:                           │
│  ☑ Avoid complex meals on consecutive days      │
│                                                  │
│ Cuisine Variety Preference:                     │
│  (Cuisines inferred from your favorited recipes)│
│  ◀═════●═══════▶                                │
│  Repeat OK    Mix it up!                        │
│                                                  │
│ [Save Preferences]                               │
└─────────────────────────────────────────────────┘
```

### 8.4 Shopping List Week Selector

```
┌─────────────────────────────────────────────┐
│ Shopping List                                │
│                                              │
│ Week: [Week 1 (Oct 28) ▼]  ← Dropdown      │
│       - Week 1 (Oct 28) - Current 🔒       │
│       - Week 2 (Nov 4)                      │
│       - Week 3 (Nov 11)                     │
│       - Week 4 (Nov 18)                     │
│                                              │
│ Progress: 14 of 31 items collected          │
│                                              │
│ ▼ Produce (8 items)                         │
│   ☐ Tomatoes - 6 whole                      │
│   ☐ Onions - 3 medium                       │
│   ☑ Garlic - 1 bulb                         │
│   ...                                        │
│                                              │
│ ▼ Grains & Pasta (3 items)                  │ ← NEW category
│   ☐ Basmati Rice - 2 cups                   │ ← From accompaniment
│   ☐ Spaghetti - 1 lb                        │
│   ...                                        │
└─────────────────────────────────────────────┘
```

---

## 9. Migration Strategy

### 9.1 Database Migration SQL

**File:** `migrations/XXX_enhanced_meal_planning.sql`

```sql
-- ============================================================================
-- Migration: Enhanced Meal Planning System
-- Date: 2025-10-24
-- Features:
--   1. Multi-week meal plan generation
--   2. Accompaniment recipe type
--   3. User preferences integration
-- ============================================================================

-- PART 1: Multi-Week Meal Plan Support
-- ============================================================================

ALTER TABLE meal_plans ADD COLUMN end_date TEXT NOT NULL DEFAULT '';
ALTER TABLE meal_plans ADD COLUMN is_locked BOOLEAN DEFAULT FALSE;
ALTER TABLE meal_plans ADD COLUMN generation_batch_id TEXT;

CREATE INDEX idx_meal_plans_user_batch ON meal_plans(user_id, generation_batch_id);
CREATE INDEX idx_meal_plans_status ON meal_plans(user_id, status);
CREATE INDEX idx_meal_plans_dates ON meal_plans(start_date, end_date);

-- Update existing meal plans with end_date
UPDATE meal_plans
SET end_date = date(start_date, '+6 days')
WHERE end_date = '';

-- Mark current/past weeks as locked
UPDATE meal_plans
SET is_locked = TRUE
WHERE date(start_date) <= date('now');

-- Update status for existing meal plans
UPDATE meal_plans
SET status = CASE
    WHEN date(start_date) <= date('now') AND date('now') <= date(end_date) THEN 'current'
    WHEN date(end_date) < date('now') THEN 'past'
    ELSE 'future'
END;

-- PART 2: Accompaniment Recipe Type
-- ============================================================================

ALTER TABLE recipes ADD COLUMN accepts_accompaniment BOOLEAN DEFAULT FALSE;
ALTER TABLE recipes ADD COLUMN preferred_accompaniments TEXT;
ALTER TABLE recipes ADD COLUMN accompaniment_category TEXT;

ALTER TABLE meal_assignments ADD COLUMN accompaniment_recipe_id TEXT;

CREATE INDEX idx_meal_assignments_accompaniment ON meal_assignments(accompaniment_recipe_id);
CREATE INDEX idx_recipes_accompaniment_type ON recipes(recipe_type) WHERE recipe_type = 'accompaniment';
CREATE INDEX idx_recipes_accompaniment_category ON recipes(accompaniment_category) WHERE accompaniment_category IS NOT NULL;

-- PART 3: User Preferences for Algorithm
-- ============================================================================

ALTER TABLE users ADD COLUMN max_prep_time_weeknight INTEGER DEFAULT 30;
ALTER TABLE users ADD COLUMN max_prep_time_weekend INTEGER DEFAULT 90;
ALTER TABLE users ADD COLUMN avoid_consecutive_complex BOOLEAN DEFAULT TRUE;
ALTER TABLE users ADD COLUMN cuisine_variety_weight REAL DEFAULT 0.7;

ALTER TABLE recipes ADD COLUMN cuisine TEXT;
ALTER TABLE recipes ADD COLUMN dietary_tags TEXT;

CREATE INDEX idx_recipes_cuisine ON recipes(cuisine);

-- PART 4: Rotation State Tracking
-- ============================================================================

CREATE TABLE meal_plan_rotation_state (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  generation_batch_id TEXT NOT NULL,
  used_main_course_ids TEXT NOT NULL,
  used_appetizer_ids TEXT NOT NULL,
  used_dessert_ids TEXT NOT NULL,
  cuisine_usage_count TEXT NOT NULL,
  last_complex_meal_date TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_rotation_state_user ON meal_plan_rotation_state(user_id);
CREATE INDEX idx_rotation_state_batch ON meal_plan_rotation_state(generation_batch_id);

-- PART 5: Triggers for Business Rules
-- ============================================================================

-- Prevent modification of locked weeks
CREATE TRIGGER prevent_locked_week_modification
BEFORE UPDATE ON meal_plans
WHEN OLD.is_locked = TRUE
BEGIN
    SELECT RAISE(FAIL, 'Cannot modify locked meal plan week');
END;

-- Auto-update meal plan status based on dates
CREATE TRIGGER update_meal_plan_status
AFTER UPDATE ON meal_plans
WHEN NEW.start_date != OLD.start_date OR NEW.end_date != OLD.end_date
BEGIN
    UPDATE meal_plans
    SET status = CASE
        WHEN date(NEW.start_date) <= date('now') AND date('now') <= date(NEW.end_date) THEN 'current'
        WHEN date(NEW.end_date) < date('now') THEN 'past'
        ELSE 'future'
    END
    WHERE id = NEW.id;
END;
```

### 9.2 Data Migration Strategy

1. **Backward Compatibility**: All new columns have sensible defaults
2. **Existing Data Preserved**: Current meal plans updated with `end_date`, `is_locked`, and `status`
3. **Gradual Rollout**: Users can continue using single-week generation until they opt into multi-week

### 9.3 Rollback Plan

```sql
-- Rollback migration if needed
DROP TRIGGER IF EXISTS prevent_locked_week_modification;
DROP TRIGGER IF EXISTS update_meal_plan_status;

DROP TABLE IF EXISTS meal_plan_rotation_state;

DROP INDEX IF EXISTS idx_rotation_state_user;
DROP INDEX IF EXISTS idx_rotation_state_batch;
DROP INDEX IF EXISTS idx_recipes_cuisine;
DROP INDEX IF EXISTS idx_recipes_accompaniment_category;
DROP INDEX IF EXISTS idx_recipes_accompaniment_type;
DROP INDEX IF EXISTS idx_meal_assignments_accompaniment;
DROP INDEX IF EXISTS idx_meal_plans_dates;
DROP INDEX IF EXISTS idx_meal_plans_status;
DROP INDEX IF EXISTS idx_meal_plans_user_batch;

ALTER TABLE recipes DROP COLUMN dietary_tags;
ALTER TABLE recipes DROP COLUMN cuisine;
ALTER TABLE users DROP COLUMN cuisine_variety_weight;
ALTER TABLE users DROP COLUMN avoid_consecutive_complex;
ALTER TABLE users DROP COLUMN max_prep_time_weekend;
ALTER TABLE users DROP COLUMN max_prep_time_weeknight;
ALTER TABLE meal_assignments DROP COLUMN accompaniment_recipe_id;
ALTER TABLE recipes DROP COLUMN accompaniment_category;
ALTER TABLE recipes DROP COLUMN preferred_accompaniments;
ALTER TABLE recipes DROP COLUMN accepts_accompaniment;
ALTER TABLE meal_plans DROP COLUMN generation_batch_id;
ALTER TABLE meal_plans DROP COLUMN is_locked;
ALTER TABLE meal_plans DROP COLUMN end_date;
```

---

## 10. Implementation Roadmap

### Phase 1: Database & Domain Foundation (Week 1-2)

**Tasks:**
1. ✅ Create database migration SQL
2. Run migration on development database
3. Update Rust domain models (Recipe, User, MealPlan structs)
4. Add new enums (AccompanimentCategory, Cuisine, DietaryTag)
5. Create new evento events
6. Update evento aggregates (User, Recipe, MealPlan)
7. Write unit tests for domain models

**Deliverables:**
- Migration file committed
- Domain models updated
- All tests passing

### Phase 2: Algorithm Implementation (Week 3-4)

**Tasks:**
1. Implement `filter_by_dietary_restrictions()`
2. Implement `select_main_course_with_preferences()`
3. Implement `select_accompaniment()`
4. Implement `RotationState` management
5. Implement `generate_single_week()` with preferences
6. Implement `generate_multi_week_meal_plans()` orchestrator
7. Write comprehensive algorithm tests

**Deliverables:**
- Algorithm functions implemented
- 80%+ test coverage on algorithm
- Performance benchmarks (< 5 seconds for 10 weeks)

### Phase 3: Backend Routes & Handlers (Week 5)

**Tasks:**
1. Create `/plan/generate-multi-week` route
2. Create `/plan/week/:week_id/regenerate` route
3. Create `/plan/regenerate-all-future` route
4. Update `/plan` route for multi-week view
5. Create `/profile/meal-planning-preferences` route
6. Update shopping list routes for week selection
7. Write integration tests for routes

**Deliverables:**
- All routes implemented
- Integration tests passing
- API documentation updated

### Phase 4: Frontend UX Implementation (Week 6-7)

**Tasks:**
1. Create multi-week calendar component (Askama template)
2. Add week navigation tabs/carousel
3. Create accompaniment display in meal slots
4. Create meal planning preferences form
5. Update recipe creation form (accompaniment fields)
6. Add week selector to shopping list
7. Implement TwinSpark interactions (week navigation, regenerate buttons)

**Deliverables:**
- All UX screens implemented
- Mobile + desktop responsive
- Accessibility compliance (WCAG AA)

### Phase 5: Testing & Refinement (Week 8)

**Tasks:**
1. End-to-end testing with Playwright
2. Performance testing (load 50 recipes, generate 10 weeks)
3. User acceptance testing (internal)
4. Bug fixes and edge case handling
5. Documentation updates

**Deliverables:**
- E2E tests passing
- Performance targets met
- User documentation complete

### Phase 6: Deployment & Monitoring (Week 9)

**Tasks:**
1. Deploy to staging environment
2. Run database migration on staging
3. Test with production-like data
4. Deploy to production
5. Monitor error rates and performance
6. Gather user feedback

**Deliverables:**
- Production deployment successful
- Monitoring dashboards active
- Rollback plan tested

---

## Summary of Changes

| Feature | Current State | New State | Impact |
|---------|---------------|-----------|--------|
| **Meal Plan Weeks** | Single week (next week only) | All possible weeks generated | Better planning horizon, reduced friction |
| **Recipe Types** | Appetizer, Main, Dessert | + Accompaniment (pasta, rice, etc.) | More realistic meal compositions |
| **Main Course** | Standalone only | Optional accompaniment pairing | Enhanced user experience |
| **Algorithm** | Basic rotation | Preference-aware (time, skill, cuisine, diet) | Personalized, practical schedules |
| **Shopping List** | Single week | Multiple weeks, week-selectable | Better grocery planning |
| **Week Locking** | No protection | Current week locked from regeneration | Prevents disruption of in-progress meals |
| **User Preferences** | Stored but unused | Actively used in algorithm | Tailored meal plans |

---

## Design Decisions (Approved)

### Stakeholder Decisions

1. ✅ **Max weeks limit**: **5 weeks maximum**
   - Provides ~1 month planning horizon
   - Prevents excessive computation for large recipe libraries
   - Balances planning horizon with UI complexity

2. ✅ **Accompaniment requirement**: **Always optional**
   - No "required" flag for accompaniments
   - Respects recipe creator's `accepts_accompaniment` boolean setting
   - Main course controls whether it accepts an accompaniment

3. ✅ **Custom cuisines**: **Allowed**
   - Users can enter custom cuisine names
   - Implemented as `Cuisine::Custom(String)` variant
   - Examples: "Fusion", "Regional Brazilian", "Home Cooking"

4. ✅ **Regeneration confirmation**: **Yes, show confirmation dialog**
   - "Regenerate All Future Weeks" displays modal confirmation
   - Prevents accidental regeneration of multiple weeks
   - Shows count of affected weeks

### Design Simplifications

5. ✅ **Cuisine preferences**: **REMOVED (redundant)**
   - User's favorite recipes already express cuisine preferences
   - Algorithm infers preferences from favorite selection ratios
   - `cuisine_variety_weight` slider controls variety vs. repetition

6. ✅ **Advance prep preferences**: **REMOVED (recipe-defined)**
   - Advance prep timing is a recipe characteristic, not user preference
   - Recipe stores "marinate 4 hours" or "rest overnight" in `advance_prep_text`
   - System sends prep reminders based on recipe requirements

### Default Values

- Max prep time (weeknight): **30 minutes**
- Max prep time (weekend): **90 minutes**
- Avoid consecutive complex: **true**
- Cuisine variety weight: **0.7** (0.0 = repeat frequently, 1.0 = maximum variety)

---

**Document Status:** ✅ Design Approved - Ready for Implementation
**Version:** 2.1 (Updated with stakeholder decisions)
**Next Steps:** Begin Phase 1 (Database & Domain Foundation)
