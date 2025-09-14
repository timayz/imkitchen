# Data Models

## User Model

**Purpose:** Represents individual users with their preferences, dietary restrictions, and household membership

**Key Attributes:**
- id: string (UUID) - Unique user identifier
- email: string - Primary authentication identifier
- name: string - Display name for household coordination
- dietaryPreferences: string[] - Vegetarian, vegan, keto, etc.
- allergies: string[] - Food allergies and intolerances
- householdId: string - Reference to shared household data
- language: string - Preferred interface language
- timezone: string - For meal planning and cooking schedules

### TypeScript Interface

```typescript
interface User {
  id: string;
  email: string;
  name: string;
  dietaryPreferences: DietaryPreference[];
  allergies: string[];
  householdId: string;
  language: Language;
  timezone: string;
  createdAt: Date;
  updatedAt: Date;
}

type DietaryPreference = 'vegetarian' | 'vegan' | 'gluten-free' | 'keto' | 'paleo' | 'dairy-free';
type Language = 'en' | 'es' | 'fr' | 'de';
```

### Relationships

- Belongs to one Household
- Has many InventoryItems
- Has many MealPlans
- Has many RecipeRatings

## Household Model

**Purpose:** Shared kitchen space for families/roommates with coordinated meal planning and inventory

**Key Attributes:**
- id: string (UUID) - Unique household identifier
- name: string - Household display name
- members: User[] - Array of household members
- settings: HouseholdSettings - Shared preferences and configurations

### TypeScript Interface

```typescript
interface Household {
  id: string;
  name: string;
  settings: HouseholdSettings;
  createdAt: Date;
  updatedAt: Date;
}

interface HouseholdSettings {
  defaultMeasurementUnit: 'metric' | 'imperial';
  sharedInventory: boolean;
  mealPlanningAccess: 'owner' | 'all-members';
  notificationPreferences: NotificationSettings;
}
```

### Relationships

- Has many Users (members)
- Has one shared Inventory
- Has many shared MealPlans

## InventoryItem Model

**Purpose:** Individual ingredients and food items tracked in pantry, refrigerator, or freezer

**Key Attributes:**
- id: string (UUID) - Unique item identifier
- name: string - Ingredient name with i18n support
- quantity: number - Current quantity available
- unit: string - Measurement unit (cups, grams, pieces, etc.)
- category: InventoryCategory - Organization category
- location: StorageLocation - Where item is stored
- expirationDate: Date - When item expires
- purchaseDate: Date - When item was acquired
- estimatedCost: number - Optional cost tracking

### TypeScript Interface

```typescript
interface InventoryItem {
  id: string;
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  category: InventoryCategory;
  location: StorageLocation;
  expirationDate: Date;
  purchaseDate: Date;
  estimatedCost?: number;
  householdId: string;
  addedBy: string; // User ID
  createdAt: Date;
  updatedAt: Date;
}

type InventoryCategory = 'proteins' | 'vegetables' | 'fruits' | 'grains' | 'dairy' | 'spices' | 'condiments' | 'beverages' | 'baking' | 'frozen';
type StorageLocation = 'pantry' | 'refrigerator' | 'freezer';
type MeasurementUnit = 'grams' | 'kilograms' | 'ounces' | 'pounds' | 'cups' | 'tablespoons' | 'teaspoons' | 'pieces' | 'milliliters' | 'liters';
```

### Relationships

- Belongs to one Household
- Added by one User
- Referenced in many ShoppingListItems
- Used in many Recipes (through ingredients)

## Recipe Model

**Purpose:** Cooking instructions with ingredients, steps, and metadata for meal planning

**Key Attributes:**
- id: string (UUID) - Unique recipe identifier
- title: string - Recipe name with i18n support
- description: string - Brief recipe description
- ingredients: RecipeIngredient[] - Required ingredients with quantities
- instructions: RecipeStep[] - Cooking steps in order
- cookingTime: number - Total cooking time in minutes
- difficulty: DifficultyLevel - Complexity rating
- servings: number - Number of servings recipe yields
- cuisine: string - Cuisine type for categorization
- tags: string[] - Searchable tags

### TypeScript Interface

```typescript
interface Recipe {
  id: string;
  title: string;
  description: string;
  ingredients: RecipeIngredient[];
  instructions: RecipeStep[];
  cookingTime: number;
  prepTime: number;
  difficulty: DifficultyLevel;
  servings: number;
  cuisine: string;
  tags: string[];
  imageUrl?: string;
  nutritionInfo?: NutritionInfo;
  source: RecipeSource;
  createdAt: Date;
  updatedAt: Date;
}

interface RecipeIngredient {
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  notes?: string;
  essential: boolean;
}

interface RecipeStep {
  stepNumber: number;
  instruction: string;
  duration?: number;
  temperature?: number;
  image?: string;
}

type DifficultyLevel = 'easy' | 'medium' | 'hard';
type RecipeSource = 'user-created' | 'imported' | 'api-external';
```

### Relationships

- Has many RecipeRatings from Users
- Saved in many UserRecipeCollections
- Used in many MealPlanEntries
- Generates many ShoppingListItems

## MealPlan Model

**Purpose:** Weekly or monthly meal scheduling with family coordination

**Key Attributes:**
- id: string (UUID) - Unique meal plan identifier
- name: string - Meal plan name
- startDate: Date - Beginning of meal plan period
- endDate: Date - End of meal plan period
- entries: MealPlanEntry[] - Individual meal assignments
- householdId: string - Associated household

### TypeScript Interface

```typescript
interface MealPlan {
  id: string;
  name: string;
  startDate: Date;
  endDate: Date;
  entries: MealPlanEntry[];
  householdId: string;
  createdBy: string; // User ID
  createdAt: Date;
  updatedAt: Date;
}

interface MealPlanEntry {
  id: string;
  date: Date;
  mealType: MealType;
  recipeId: string;
  servings: number;
  notes?: string;
  assignedCook?: string; // User ID
  status: MealStatus;
}

type MealType = 'breakfast' | 'lunch' | 'dinner' | 'snack';
type MealStatus = 'planned' | 'in-progress' | 'completed' | 'skipped';
```

### Relationships

- Belongs to one Household
- Created by one User
- Contains many MealPlanEntries
- Generates ShoppingLists

## ShoppingList Model

**Purpose:** Automated and manual shopping lists with store organization and purchase tracking

**Key Attributes:**
- id: string (UUID) - Unique shopping list identifier
- name: string - Shopping list name
- items: ShoppingListItem[] - Items to purchase
- generatedFrom: string[] - Source meal plan IDs
- status: ShoppingListStatus - Current list status
- estimatedTotal: number - Projected cost

### TypeScript Interface

```typescript
interface ShoppingList {
  id: string;
  name: string;
  items: ShoppingListItem[];
  generatedFrom: string[]; // MealPlan IDs
  status: ShoppingListStatus;
  estimatedTotal: number;
  householdId: string;
  createdBy: string; // User ID
  createdAt: Date;
  updatedAt: Date;
}

interface ShoppingListItem {
  id: string;
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  category: StoreCategory;
  purchased: boolean;
  estimatedPrice?: number;
  actualPrice?: number;
  notes?: string;
}

type ShoppingListStatus = 'active' | 'completed' | 'archived';
type StoreCategory = 'produce' | 'dairy' | 'meat' | 'frozen' | 'pantry' | 'bakery' | 'other';
```

### Relationships

- Belongs to one Household
- Generated from MealPlans
- Created by one User
- Updates InventoryItems when marked purchased
