// Meal plan-related TypeScript interfaces and types
// Shared between mobile app and backend API

import { Recipe } from './recipe';
import type { MealType } from './rating.types';

export interface MealSlot {
  day: string; // monday, tuesday, etc.
  mealType: MealType;
  recipe?: Recipe;
  recipeId?: string;
  servings: number;
  notes?: string;
  isCompleted: boolean;
  isLocked?: boolean;
}

export interface MealSlotWithRecipe extends MealSlot {
  recipe?: Recipe;
}

export interface WeeklyMeals {
  monday: MealSlot[];
  tuesday: MealSlot[];
  wednesday: MealSlot[];
  thursday: MealSlot[];
  friday: MealSlot[];
  saturday: MealSlot[];
  sunday: MealSlot[];
}

export interface WeeklyMealsWithRecipes {
  monday: MealSlotWithRecipe[];
  tuesday: MealSlotWithRecipe[];
  wednesday: MealSlotWithRecipe[];
  thursday: MealSlotWithRecipe[];
  friday: MealSlotWithRecipe[];
  saturday: MealSlotWithRecipe[];
  sunday: MealSlotWithRecipe[];
}

export interface MealPlan {
  id: string;
  userId: string;
  weekStartDate: Date;
  generationType: 'automated' | 'manual' | 'mixed';
  generatedAt: Date;
  totalEstimatedTime: number; // minutes
  isActive: boolean;
  status: 'draft' | 'active' | 'archived' | 'deleted';
  completionPercentage?: number;
  createdAt: Date;
  updatedAt: Date;
  archivedAt?: Date;
}

export interface MealPlanResponse {
  id: string;
  userId: string;
  weekStartDate: Date;
  generationType: 'automated' | 'manual' | 'mixed';
  generatedAt: Date;
  totalEstimatedTime: number; // minutes
  isActive: boolean;
  status: 'draft' | 'active' | 'archived' | 'deleted';
  completionPercentage?: number;
  populatedMeals: WeeklyMealsWithRecipes;
  createdAt: Date;
  updatedAt: Date;
  archivedAt?: Date;
}

// Input DTOs for meal plan creation and updates
export interface CreateMealPlanInput {
  weekStartDate: Date;
  generationType: 'automated' | 'manual' | 'mixed';
  meals: WeeklyMeals;
}

export interface UpdateMealPlanInput {
  meals?: WeeklyMeals;
  status?: 'draft' | 'active' | 'archived' | 'deleted';
  completionPercentage?: number;
}

export interface UpdateMealSlotInput {
  recipeId?: string;
  servings?: number;
  notes?: string;
  isCompleted?: boolean;
}

// "Fill My Week" generation types
export interface GenerateMealPlanInput {
  weekStartDate?: Date;
  maxPrepTimePerMeal?: number;
  preferredComplexityLevel?: 'simple' | 'moderate' | 'complex';
  avoidRecipeIDs?: string[];
  cuisinePreferences?: string[];
}

export interface MealPlanGenerationResponse {
  mealPlan: MealPlanResponse;
  generationTimeMs: number;
  varietyScore: number;
  recipesUsed: number;
  rotationCycle: number;
  warnings?: string[];
}

// Filter and search interfaces
export interface MealPlanFilters {
  weekStart?: Date;
  weekEnd?: Date;
  status?: 'draft' | 'active' | 'archived' | 'deleted';
}

// Calendar helper types
export type DayOfWeek = 'monday' | 'tuesday' | 'wednesday' | 'thursday' | 'friday' | 'saturday' | 'sunday';
// MealType is imported from rating.types.ts to avoid duplication

export interface CalendarWeek {
  weekStart: Date;
  weekEnd: Date;
  days: {
    date: Date;
    dayName: DayOfWeek;
    meals: {
      breakfast?: MealSlotWithRecipe;
      lunch?: MealSlotWithRecipe;
      dinner?: MealSlotWithRecipe;
    };
  }[];
}

// Color coding system for complexity and preparation
export interface ComplexityColors {
  simple: string;    // Green
  moderate: string;  // Orange
  complex: string;   // Red
}

export interface PreparationColors {
  sameDay: string;   // Default
  prepAhead: string; // Blue
  makeAhead: string; // Purple
}

// Drag and drop types
export interface DragDropMealData {
  recipeId: string;
  recipe: Recipe;
  sourceMealPlanId?: string;
  sourceDay?: DayOfWeek;
  sourceMealType?: MealType;
}

export interface DropTargetData {
  mealPlanId: string;
  day: DayOfWeek;
  mealType: MealType;
  accepts: 'recipe' | 'meal';
}

// Empty state types
export interface EmptyStateConfig {
  title: string;
  message: string;
  actionText?: string;
  actionHandler?: () => void;
  icon?: string;
}

// Week navigation types
export interface WeekNavigationProps {
  currentWeek: Date;
  onWeekChange: (weekStart: Date) => void;
  canNavigateBack?: boolean;
  canNavigateForward?: boolean;
}

// Mobile-specific props
export interface MealPlanGridProps {
  mealPlan: MealPlanResponse | null;
  onMealPress?: (day: DayOfWeek, mealType: MealType, meal?: MealSlotWithRecipe) => void;
  onMealLongPress?: (day: DayOfWeek, mealType: MealType, meal?: MealSlotWithRecipe) => void;
  onEmptySlotPress?: (day: DayOfWeek, mealType: MealType) => void;
  onMealPlanUpdate?: (updatedMealPlan: MealPlanResponse) => void;
  isEditable?: boolean;
  loading?: boolean;
  error?: string | null;
}

export interface MealSlotProps {
  day: DayOfWeek;
  mealType: MealType;
  meal?: MealSlotWithRecipe;
  onPress?: () => void;
  onLongPress?: () => void;
  isEditable?: boolean;
  isEmpty?: boolean;
  dragEnabled?: boolean;
  dropEnabled?: boolean;
}

export interface WeekNavigatorProps {
  currentWeek: Date;
  onPreviousWeek: () => void;
  onNextWeek: () => void;
  onWeekSelect?: (date: Date) => void;
  showWeekSelector?: boolean;
  minDate?: Date;
  maxDate?: Date;
}