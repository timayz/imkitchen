# Data Models

## User

**Purpose:** Represents platform users with cooking preferences, dietary restrictions, and household configuration

**Key Attributes:**
- id: Uuid - Unique identifier
- email: String - Login credential and communication
- password_hash: String - Secure authentication storage
- dietary_preferences: Vec<DietaryRestriction> - Filtering and meal planning
- skill_level: SkillLevel - Recipe difficulty matching
- household_size: u32 - Recipe scaling and portions
- kitchen_equipment: Vec<Equipment> - Recipe feasibility checking
- created_at: DateTime - Account creation tracking
- updated_at: DateTime - Profile modification tracking

### TypeScript Interface

```typescript
interface User {
  id: string;
  email: string;
  dietary_preferences: DietaryRestriction[];
  skill_level: 'beginner' | 'intermediate' | 'advanced';
  household_size: number;
  kitchen_equipment: Equipment[];
  created_at: string;
  updated_at: string;
}

enum DietaryRestriction {
  Vegetarian = 'vegetarian',
  Vegan = 'vegan', 
  GlutenFree = 'gluten_free',
  DairyFree = 'dairy_free',
  NutFree = 'nut_free'
}
```

### Relationships
- One-to-many with Recipe (user's imported recipes)
- One-to-many with MealPlan (user's meal planning history)
- One-to-many with CookingSession (user's cooking activities)

## Recipe

**Purpose:** Core recipe data with ingredients, instructions, timing, and nutritional information

**Key Attributes:**
- id: Uuid - Unique identifier
- user_id: Uuid - Owner reference
- title: String - Recipe name
- description: Option<String> - Recipe summary
- ingredients: Vec<Ingredient> - Required components with quantities
- instructions: Vec<Instruction> - Step-by-step cooking guide
- prep_time: Duration - Preparation duration
- cook_time: Duration - Cooking duration
- total_time: Duration - Complete recipe duration
- servings: u32 - Default portion size
- difficulty: SkillLevel - Complexity rating
- cuisine_type: Option<String> - Culinary classification
- tags: Vec<String> - Searchable keywords
- nutritional_info: Option<Nutrition> - Health information
- source_url: Option<String> - Original recipe location
- created_at: DateTime - Import/creation time
- updated_at: DateTime - Last modification

### TypeScript Interface

```typescript
interface Recipe {
  id: string;
  user_id: string;
  title: string;
  description?: string;
  ingredients: Ingredient[];
  instructions: Instruction[];
  prep_time: number; // minutes
  cook_time: number; // minutes  
  total_time: number; // minutes
  servings: number;
  difficulty: SkillLevel;
  cuisine_type?: string;
  tags: string[];
  nutritional_info?: Nutrition;
  source_url?: string;
  created_at: string;
  updated_at: string;
}

interface Ingredient {
  name: string;
  quantity: number;
  unit: string;
  notes?: string;
}

interface Instruction {
  step_number: number;
  description: string;
  duration?: number; // minutes
  temperature?: number; // celsius
}
```

### Relationships
- Many-to-one with User (recipe owner)
- Many-to-many with MealPlan (planned meals)
- One-to-many with CookingSession (cooking instances)

## MealPlan

**Purpose:** Weekly meal planning with recipe assignments, shopping list generation, and schedule optimization

**Key Attributes:**
- id: Uuid - Unique identifier
- user_id: Uuid - Owner reference
- week_start: NaiveDate - Planning week beginning
- meals: HashMap<Weekday, Vec<PlannedMeal>> - Daily meal assignments
- shopping_list: Vec<ShoppingItem> - Aggregated ingredients
- generated_at: DateTime - AI generation timestamp
- confirmed_at: Option<DateTime> - User approval time
- completed_at: Option<DateTime> - Execution completion

### TypeScript Interface

```typescript
interface MealPlan {
  id: string;
  user_id: string;
  week_start: string; // ISO date
  meals: Record<Weekday, PlannedMeal[]>;
  shopping_list: ShoppingItem[];
  generated_at: string;
  confirmed_at?: string;
  completed_at?: string;
}

interface PlannedMeal {
  meal_type: 'breakfast' | 'lunch' | 'dinner' | 'snack';
  recipe_id: string;
  scheduled_time?: string; // ISO datetime
  scaling_factor: number;
  notes?: string;
}

interface ShoppingItem {
  ingredient_name: string;
  quantity: number;
  unit: string;
  category: string; // grocery aisle
  purchased: boolean;
}
```

### Relationships
- Many-to-one with User (meal plan owner)
- Many-to-many with Recipe (planned recipes)
- One-to-many with CookingSession (executed meals)

## CookingSession

**Purpose:** Active cooking tracking with timing coordination, progress monitoring, and feedback collection

**Key Attributes:**
- id: Uuid - Unique identifier  
- user_id: Uuid - Cook reference
- recipe_id: Uuid - Recipe being executed
- meal_plan_id: Option<Uuid> - Associated meal plan
- start_time: DateTime - Cooking commencement
- end_time: Option<DateTime> - Cooking completion
- current_step: u32 - Progress tracking
- timers: Vec<Timer> - Active cooking timers
- scaling_factor: f32 - Recipe portion adjustment
- notes: Vec<CookingNote> - User observations
- rating: Option<u8> - Recipe satisfaction (1-5)
- timing_accuracy: Option<i32> - Actual vs predicted time difference

### TypeScript Interface

```typescript
interface CookingSession {
  id: string;
  user_id: string;
  recipe_id: string;
  meal_plan_id?: string;
  start_time: string;
  end_time?: string;
  current_step: number;
  timers: Timer[];
  scaling_factor: number;
  notes: CookingNote[];
  rating?: number; // 1-5
  timing_accuracy?: number; // minutes difference
}

interface Timer {
  id: string;
  name: string;
  duration: number; // minutes
  remaining: number; // minutes
  status: 'running' | 'paused' | 'completed';
}

interface CookingNote {
  timestamp: string;
  step_number: number;
  content: string;
  note_type: 'observation' | 'modification' | 'issue';
}
```

### Relationships
- Many-to-one with User (cooking session owner)
- Many-to-one with Recipe (recipe being cooked)
- Many-to-one with MealPlan (associated meal plan)
