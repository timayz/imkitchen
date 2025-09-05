# Data Models

Based on the PRD requirements and user flows, I've identified the core business entities for imkitchen's meal planning automation system:

## User

**Purpose:** Represents home cooking enthusiasts using the meal planning automation system

**Key Attributes:**
- id: UUID - Unique user identifier
- email: String - Authentication and communication
- password_hash: String - Secure authentication storage
- display_name: String - User-friendly identification
- created_at: DateTime - Account tracking
- dietary_restrictions: Vec<DietaryRestriction> - Meal planning filters
- preferences: UserPreferences - Personalization settings

### TypeScript Interface
```typescript
interface User {
  id: string;
  email: string;
  displayName: string;
  createdAt: Date;
  dietaryRestrictions: DietaryRestriction[];
  preferences: UserPreferences;
}

interface UserPreferences {
  defaultServings: number;
  mealComplexityPreference: 'easy' | 'medium' | 'hard';
  cookingTimePreference: number; // max minutes
  favoriteCategories: string[];
}

type DietaryRestriction = 'vegetarian' | 'vegan' | 'gluten_free' | 'dairy_free' | 'nut_free' | 'keto' | 'paleo';
```

### Relationships
- One-to-many: User → Recipe (user's personal recipes)
- One-to-many: User → MealPlan (historical meal plans)
- One-to-many: User → RecipeRating (community ratings)

## Recipe

**Purpose:** Core entity representing individual recipes with timing, ingredients, and community validation

**Key Attributes:**
- id: UUID - Unique recipe identifier
- title: String - Recipe name
- description: String - Brief recipe overview
- prep_time_minutes: i32 - Preparation time
- cook_time_minutes: i32 - Active cooking time
- total_time_minutes: i32 - Complete recipe duration
- difficulty_level: DifficultyLevel - Complexity indicator
- servings: i32 - Default serving size
- category: RecipeCategory - Meal type classification
- instructions: Vec<RecipeStep> - Ordered cooking steps
- created_by: UUID - Recipe creator (user_id)
- is_public: bool - Community visibility
- image_url: Option<String> - Recipe photo
- nutritional_info: Option<NutritionalInfo> - Health information

### TypeScript Interface
```typescript
interface Recipe {
  id: string;
  title: string;
  description: string;
  prepTimeMinutes: number;
  cookTimeMinutes: number;
  totalTimeMinutes: number;
  difficultyLevel: DifficultyLevel;
  servings: number;
  category: RecipeCategory;
  ingredients: RecipeIngredient[];
  instructions: RecipeStep[];
  createdBy: string;
  isPublic: boolean;
  imageUrl?: string;
  nutritionalInfo?: NutritionalInfo;
  communityRating?: number;
  personalRating?: number;
}

type DifficultyLevel = 'easy' | 'medium' | 'hard';
type RecipeCategory = 'breakfast' | 'lunch' | 'dinner' | 'snack' | 'dessert';

interface RecipeStep {
  order: number;
  instruction: string;
  duration?: number; // optional timer
}

interface RecipeIngredient {
  ingredientId: string;
  quantity: number;
  unit: string;
  notes?: string;
}
```

### Relationships
- Many-to-one: Recipe → User (created_by)
- One-to-many: Recipe → RecipeRating (community feedback)
- Many-to-many: Recipe ↔ Ingredient (via RecipeIngredient)
- One-to-many: Recipe → MealPlanSlot (scheduled meals)

## MealPlan

**Purpose:** Weekly meal planning automation with rotation tracking and shopping list generation

**Key Attributes:**
- id: UUID - Unique meal plan identifier
- user_id: UUID - Plan owner
- week_start_date: Date - Week beginning (Monday)
- created_at: DateTime - Generation timestamp
- is_current: bool - Active meal plan flag
- generation_method: GenerationMethod - How plan was created

### TypeScript Interface
```typescript
interface MealPlan {
  id: string;
  userId: string;
  weekStartDate: Date;
  createdAt: Date;
  isCurrent: boolean;
  generationMethod: GenerationMethod;
  slots: MealPlanSlot[];
}

type GenerationMethod = 'auto_fill_my_week' | 'manual' | 'partial_auto';

interface MealPlanSlot {
  id: string;
  mealPlanId: string;
  date: Date;
  mealType: MealType;
  recipeId?: string;
  customMealName?: string; // for non-recipe meals
  isCompleted: boolean;
}

type MealType = 'breakfast' | 'lunch' | 'dinner';
```

### Relationships
- Many-to-one: MealPlan → User
- One-to-many: MealPlan → MealPlanSlot
- One-to-many: MealPlan → ShoppingList

## RecipeRotation

**Purpose:** Tracks recipe usage history to ensure variety in "Fill My Week" automation

**Key Attributes:**
- id: UUID - Rotation tracking identifier
- user_id: UUID - Rotation owner
- recipe_id: UUID - Recipe being tracked
- last_used_date: Option<Date> - Most recent meal plan usage
- rotation_cycle: i32 - Current rotation round
- is_excluded: bool - User preference exclusion

### TypeScript Interface
```typescript
interface RecipeRotation {
  id: string;
  userId: string;
  recipeId: string;
  lastUsedDate?: Date;
  rotationCycle: number;
  isExcluded: boolean;
  usageCount: number;
}
```

### Relationships
- Many-to-one: RecipeRotation → User
- Many-to-one: RecipeRotation → Recipe