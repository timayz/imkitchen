# Data Models

## User

**Purpose:** Central user entity supporting authentication, preferences, and personalized meal planning automation

**Key Attributes:**
- id: UUID - Primary identifier for cross-service references
- email: string - Authentication and communication
- displayName: string - User interface personalization  
- dietaryRestrictions: string[] - Array of dietary constraints for meal planning
- cookingSkillLevel: enum - Beginner, Intermediate, Advanced for complexity filtering
- weeklyAvailability: JSON - Time availability patterns for intelligent scheduling
- preferredMealComplexity: enum - Simple, Moderate, Complex for automation preferences
- rotationResetCount: integer - Tracking recipe rotation cycles

### TypeScript Interface

```typescript
interface User {
  id: string;
  email: string;
  displayName: string;
  dietaryRestrictions: string[];
  cookingSkillLevel: 'beginner' | 'intermediate' | 'advanced';
  weeklyAvailability: {
    [day: string]: {
      morningEnergy: 'low' | 'medium' | 'high';
      availableTime: number; // minutes
    };
  };
  preferredMealComplexity: 'simple' | 'moderate' | 'complex';
  rotationResetCount: number;
  createdAt: Date;
  updatedAt: Date;
}
```

### Relationships
- User → many Recipes (recipe collection)
- User → many MealPlans (generated meal plans)
- User → many RecipeRatings (community participation)

## Recipe

**Purpose:** Core recipe entity with metadata essential for intelligent meal planning automation and community features

**Key Attributes:**
- id: UUID - Primary identifier
- title: string - Recipe display name
- description: text - Recipe overview for community features
- ingredients: Ingredient[] - Structured ingredient list for shopping automation
- instructions: Step[] - Cooking steps with time estimates
- prepTime: integer - Total preparation time in minutes for scheduling
- cookTime: integer - Active cooking time for availability matching  
- complexity: enum - Simple, Moderate, Complex for automation filtering
- cuisineType: string - Cultural categorization for variety optimization
- mealType: enum - Breakfast, Lunch, Dinner, Snack for calendar assignment
- servings: integer - Portion size for ingredient scaling
- imageUrl: string - Recipe photo URL for visual appeal

### TypeScript Interface

```typescript
interface Recipe {
  id: string;
  userId: string;
  title: string;
  description: string;
  ingredients: {
    name: string;
    amount: number;
    unit: string;
    category: 'produce' | 'dairy' | 'pantry' | 'protein' | 'other';
  }[];
  instructions: {
    stepNumber: number;
    instruction: string;
    estimatedMinutes?: number;
  }[];
  prepTime: number;
  cookTime: number;
  complexity: 'simple' | 'moderate' | 'complex';
  cuisineType: string;
  mealType: 'breakfast' | 'lunch' | 'dinner' | 'snack';
  servings: number;
  imageUrl?: string;
  isPublic: boolean;
  createdAt: Date;
  updatedAt: Date;
}
```

### Relationships
- Recipe → one User (owner)
- Recipe → many MealPlanEntries (usage in meal plans)
- Recipe → many RecipeRatings (community feedback)

## MealPlan

**Purpose:** Weekly meal planning container with automation metadata for user meal organization and rotation tracking

**Key Attributes:**
- id: UUID - Primary identifier
- userId: UUID - Plan owner reference
- weekStartDate: Date - Monday of the target week for calendar alignment
- generationType: enum - Automated, Manual, Mixed for generation tracking
- generatedAt: DateTime - Automation timestamp for performance monitoring
- totalEstimatedTime: integer - Weekly cooking time for user planning
- isActive: boolean - Current active plan for user interface

### TypeScript Interface

```typescript
interface MealPlan {
  id: string;
  userId: string;
  weekStartDate: Date;
  generationType: 'automated' | 'manual' | 'mixed';
  generatedAt: Date;
  totalEstimatedTime: number; // minutes
  isActive: boolean;
  entries: MealPlanEntry[];
  createdAt: Date;
  updatedAt: Date;
}
```

### Relationships
- MealPlan → one User (owner)
- MealPlan → many MealPlanEntries (daily meal assignments)

## MealPlanEntry

**Purpose:** Individual meal assignment within weekly plan supporting manual overrides and scheduling optimization

**Key Attributes:**
- id: UUID - Primary identifier
- mealPlanId: UUID - Parent plan reference
- recipeId: UUID - Assigned recipe reference  
- date: Date - Specific day assignment
- mealType: enum - Breakfast, Lunch, Dinner for calendar positioning
- isManualOverride: boolean - Track user modifications vs automation
- scheduledPrepTime: DateTime - Optimal prep timing based on recipe requirements
- isCompleted: boolean - User cooking completion tracking

### TypeScript Interface

```typescript
interface MealPlanEntry {
  id: string;
  mealPlanId: string;
  recipeId: string;
  date: Date;
  mealType: 'breakfast' | 'lunch' | 'dinner';
  isManualOverride: boolean;
  scheduledPrepTime?: Date;
  isCompleted: boolean;
  notes?: string;
  createdAt: Date;
  updatedAt: Date;
}
```

### Relationships
- MealPlanEntry → one MealPlan (parent plan)
- MealPlanEntry → one Recipe (assigned recipe)

## RecipeRating

**Purpose:** Community feedback system enabling recipe quality validation and social engagement features

**Key Attributes:**
- id: UUID - Primary identifier
- recipeId: UUID - Rated recipe reference
- userId: UUID - Rating contributor
- rating: integer - 1-5 star rating scale
- review: text - Optional written feedback
- difficulty: enum - Community difficulty assessment vs original recipe rating
- wouldCookAgain: boolean - Recipe recommendation indicator

### TypeScript Interface

```typescript
interface RecipeRating {
  id: string;
  recipeId: string;
  userId: string;
  rating: number; // 1-5
  review?: string;
  difficulty: 'easier' | 'as_expected' | 'harder';
  wouldCookAgain: boolean;
  createdAt: Date;
  updatedAt: Date;
}
```

### Relationships
- RecipeRating → one Recipe (rated recipe)  
- RecipeRating → one User (rating author)
