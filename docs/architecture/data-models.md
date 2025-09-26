# Data Models

## User Profile Model

**Purpose:** Represents user account, preferences, and cooking profile for personalized meal planning

**Key Attributes:**
- user_id: Uuid - Unique identifier for user account
- email: Email - Validated email address for authentication
- dietary_restrictions: Vec<DietaryRestriction> - List of dietary constraints
- family_size: FamilySize - Number of people (1-8) for meal planning
- cooking_skill_level: SkillLevel - Beginner/Intermediate/Advanced for recipe complexity
- available_cooking_time: CookingTime - Weekday/weekend time availability

### Rust Struct with Evento Integration
```rust
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Evento aggregate for user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub email: String,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub family_size: u8,
    pub cooking_skill_level: SkillLevel,
    pub available_cooking_time: CookingTimeAvailability,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Evento events as individual structs
#[derive(AggregatorName, Encode, Decode)]
struct UserRegistered {
    pub email: String,
    pub family_size: u8,
    pub cooking_skill_level: SkillLevel,
}

#[derive(AggregatorName, Encode, Decode)]
struct ProfileUpdated {
    pub family_size: u8,
    pub cooking_skill_level: SkillLevel,
}

#[derive(AggregatorName, Encode, Decode)]
struct DietaryRestrictionsChanged {
    pub restrictions: Vec<DietaryRestriction>,
}

// Evento aggregate implementation
#[evento::aggregator]
impl UserProfile {
    async fn user_registered(&mut self, event: EventDetails<UserRegistered>) -> Result<()> {
        self.user_id = Uuid::parse_str(&event.aggregate_id)?;
        self.email = event.data.email.clone();
        self.family_size = event.data.family_size;
        self.cooking_skill_level = event.data.cooking_skill_level.clone();
        self.created_at = event.created_at;
        self.updated_at = event.created_at;
        Ok(())
    }
    
    async fn profile_updated(&mut self, event: EventDetails<ProfileUpdated>) -> Result<()> {
        self.family_size = event.data.family_size;
        self.cooking_skill_level = event.data.cooking_skill_level.clone();
        self.updated_at = event.created_at;
        Ok(())
    }
    
    async fn dietary_restrictions_changed(&mut self, event: EventDetails<DietaryRestrictionsChanged>) -> Result<()> {
        self.dietary_restrictions = event.data.restrictions.clone();
        self.updated_at = event.created_at;
        Ok(())
    }
}
```

### Relationships
- Has many Recipe collections (one-to-many)
- Has many MealPlan instances (one-to-many)
- Has many ShoppingList instances (one-to-many)

## Recipe Model

**Purpose:** Represents individual recipes with ingredients, instructions, and community metadata

**Key Attributes:**
- recipe_id: Uuid - Unique recipe identifier
- title: String - Recipe name (1-200 characters)
- ingredients: Vec<Ingredient> - List of recipe ingredients with quantities
- instructions: Vec<Instruction> - Step-by-step cooking instructions
- prep_time_minutes: u32 - Preparation time in minutes
- cook_time_minutes: u32 - Cooking time in minutes
- difficulty: Difficulty - Easy/Medium/Hard complexity rating
- category: RecipeCategory - Meal type classification

### Rust Struct Definition
```rust
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Recipe {
    pub recipe_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub ingredients: Vec<Ingredient>,
    #[validate(length(min = 1))]
    pub instructions: Vec<Instruction>,
    #[validate(range(min = 1))]
    pub prep_time_minutes: u32,
    #[validate(range(min = 1))]
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    #[validate(range(min = 0.0, max = 5.0))]
    pub rating: f32,
    pub review_count: u32,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}
```

### Relationships
- Belongs to many Recipe collections (many-to-many)
- Has many Recipe ratings (one-to-many)
- Used in many MealSlot instances (one-to-many)

## MealPlan Model

**Purpose:** Represents weekly meal schedules with automated planning intelligence

**Key Attributes:**
- meal_plan_id: Uuid - Unique meal plan identifier
- user_id: Uuid - Owner of the meal plan
- week_start_date: Date - Beginning of the planned week
- meal_slots: Vec<MealSlot> - 21 slots (7 days × 3 meals)
- generation_algorithm: String - Algorithm version used
- preferences_snapshot: UserPreferences - User settings at generation time

### Rust Struct Definition
```rust
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Date, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MealPlan {
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    pub week_start_date: Date<Utc>,
    #[validate(length(min = 21, max = 21))] // 7 days × 3 meals
    pub meal_slots: Vec<MealSlot>,
    pub generation_algorithm: String,
    pub preferences_snapshot: UserPreferences,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSlot {
    pub day_of_week: u8, // 0-6
    pub meal_type: MealType,
    pub recipe_id: Option<Uuid>,
    pub requires_advance_prep: bool,
    pub prep_start_time: Option<chrono::NaiveTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}
```

### Relationships
- Belongs to one User (many-to-one)
- Contains many MealSlot instances (one-to-many)
- Generates one ShoppingList (one-to-one)

## ShoppingList Model

**Purpose:** Represents consolidated ingredient lists with optimization and family sharing

**Key Attributes:**
- shopping_list_id: Uuid - Unique shopping list identifier
- meal_plan_id: Uuid - Source meal plan
- items: Vec<ShoppingItem> - Consolidated ingredients with quantities
- store_sections: Vec<StoreSection> - Organized by grocery store layout
- estimated_cost: Option<Decimal> - Calculated shopping cost
- shared_with: Vec<Uuid> - Family members with access

### Rust Struct Definition
```rust
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ShoppingList {
    pub shopping_list_id: Uuid,
    pub meal_plan_id: Uuid,
    #[validate(length(min = 1))]
    pub items: Vec<ShoppingItem>,
    pub store_sections: Vec<StoreSection>,
    pub estimated_cost: Option<Decimal>,
    pub shared_with: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_shared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingItem {
    pub ingredient_name: String,
    pub quantity: Decimal,
    pub unit: String,
    pub is_checked: bool,
    pub estimated_cost: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoreSection {
    Produce,
    Dairy,
    Meat,
    Pantry,
    Frozen,
    Bakery,
    Other,
}
```

### Relationships
- Generated from one MealPlan (one-to-one)
- Shared with many Users (many-to-many)
- Contains many ShoppingItem instances (one-to-many)
