import { MealPlanStateService } from '../../src/services/MealPlanStateService';
import { mealPlanService } from '../../src/services/meal_plan_service';
import type { MealPlanResponse } from '@imkitchen/shared-types';
import type { MealPlanSnapshot } from '../../src/services/ChangeHistoryTracker';

// Mock the meal plan service
jest.mock('../../src/services/meal_plan_service', () => ({
  mealPlanService: {
    getMealPlan: jest.fn(),
    updateMealSlot: jest.fn(),
    toggleMealLock: jest.fn(),
  },
}));

const mockMealPlanService = mealPlanService as jest.Mocked<typeof mealPlanService>;

describe('MealPlanStateService', () => {
  let stateService: MealPlanStateService;
  const mealPlanId = 'test-meal-plan-123';

  const createMockMealPlan = (): MealPlanResponse => ({
    id: mealPlanId,
    name: 'Test Meal Plan',
    weekStartDate: '2024-01-01',
    populatedMeals: {
      monday: [
        {
          id: 'slot-1',
          mealType: 'breakfast',
          day: 'monday',
          recipeId: 'recipe-1',
          servings: 2,
          isLocked: false,
          isCompleted: false,
          notes: 'Original meal',
          recipe: {
            id: 'recipe-1',
            title: 'Original Recipe',
            description: 'Test recipe',
            preparationTimeMinutes: 15,
            cookingTimeMinutes: 20,
            totalTimeMinutes: 35,
            servings: 4,
            ingredients: [],
            instructions: [],
            nutritionInfo: { calories: 300, protein: 20, carbs: 25, fat: 12 },
            tags: [],
            difficulty: 'easy',
            cuisineType: 'american',
            imageUrl: '',
            createdAt: new Date(),
            updatedAt: new Date(),
          },
        },
      ],
      tuesday: [
        {
          id: 'slot-2',
          mealType: 'lunch',
          day: 'tuesday',
          recipeId: 'recipe-2',
          servings: 1,
          isLocked: true,
          isCompleted: false,
          recipe: {
            id: 'recipe-2',
            title: 'Locked Recipe',
            description: 'A locked test recipe',
            preparationTimeMinutes: 10,
            cookingTimeMinutes: 15,
            totalTimeMinutes: 25,
            servings: 2,
            ingredients: [],
            instructions: [],
            nutritionInfo: { calories: 200, protein: 15, carbs: 20, fat: 8 },
            tags: [],
            difficulty: 'easy',
            cuisineType: 'italian',
            imageUrl: '',
            createdAt: new Date(),
            updatedAt: new Date(),
          },
        },
      ],
    },
    totalEstimatedTime: 60,
    completionPercentage: 25,
    createdAt: new Date(),
    updatedAt: new Date(),
  });

  const createMockSnapshot = (overrides: Partial<MealPlanSnapshot> = {}): MealPlanSnapshot => ({
    mealPlanId,
    version: Date.now(),
    timestamp: new Date(),
    meals: {
      monday: {
        breakfast: {
          recipeId: 'recipe-1',
          recipeName: 'Original Recipe',
          servings: 2,
          isLocked: false,
          isCompleted: false,
          notes: 'Original meal',
        },
      },
      tuesday: {
        lunch: {
          recipeId: 'recipe-2',
          recipeName: 'Locked Recipe',
          servings: 1,
          isLocked: true,
          isCompleted: false,
        },
      },
    },
    totalEstimatedTime: 60,
    completionPercentage: 25,
    ...overrides,
  });

  beforeEach(() => {
    stateService = new MealPlanStateService();
    jest.clearAllMocks();

    // Default mock implementations
    mockMealPlanService.getMealPlan.mockResolvedValue(createMockMealPlan());
    mockMealPlanService.updateMealSlot.mockResolvedValue(createMockMealPlan());
    mockMealPlanService.toggleMealLock.mockResolvedValue(createMockMealPlan());
  });

  describe('Snapshot Application', () => {
    test('should apply snapshot with recipe changes successfully', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-3', // Changed recipe
              recipeName: 'New Recipe',
              servings: 3, // Changed servings
              isLocked: false,
              isCompleted: false,
            },
          },
          tuesday: {
            lunch: {
              recipeId: 'recipe-2',
              recipeName: 'Locked Recipe',
              servings: 1,
              isLocked: true,
              isCompleted: false,
            },
          },
        },
      });

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true);
      expect(result.changesApplied).toHaveLength(1);
      expect(result.changesApplied![0]).toEqual({
        day: 'monday',
        mealType: 'breakfast',
        action: 'update',
        details: 'Change from Original Recipe to New Recipe',
      });

      expect(mockMealPlanService.updateMealSlot).toHaveBeenCalledWith(mealPlanId, {
        day: 'monday',
        mealType: 'breakfast',
        recipeId: 'recipe-3',
        servings: 3,
      });
    });

    test('should apply snapshot with lock changes successfully', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-1',
              recipeName: 'Original Recipe',
              servings: 2,
              isLocked: true, // Changed to locked
              isCompleted: false,
            },
          },
          tuesday: {
            lunch: {
              recipeId: 'recipe-2',
              recipeName: 'Locked Recipe',
              servings: 1,
              isLocked: false, // Changed to unlocked
              isCompleted: false,
            },
          },
        },
      });

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true);
      expect(result.changesApplied).toHaveLength(2);

      // Should unlock first (higher priority), then lock
      expect(result.changesApplied![0].action).toBe('unlock');
      expect(result.changesApplied![1].action).toBe('lock');

      expect(mockMealPlanService.toggleMealLock).toHaveBeenCalledWith(
        mealPlanId,
        'tuesday',
        'lunch',
        false
      );
      expect(mockMealPlanService.toggleMealLock).toHaveBeenCalledWith(
        mealPlanId,
        'monday',
        'breakfast',
        true
      );
    });

    test('should add new meals correctly', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-1',
              recipeName: 'Original Recipe',
              servings: 2,
              isLocked: false,
              isCompleted: false,
            },
            dinner: { // New meal
              recipeId: 'recipe-4',
              recipeName: 'Dinner Recipe',
              servings: 4,
              isLocked: false,
              isCompleted: false,
            },
          },
          tuesday: {
            lunch: {
              recipeId: 'recipe-2',
              recipeName: 'Locked Recipe',
              servings: 1,
              isLocked: true,
              isCompleted: false,
            },
          },
        },
      });

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true);
      expect(result.changesApplied).toHaveLength(1);
      expect(result.changesApplied![0]).toEqual({
        day: 'monday',
        mealType: 'dinner',
        action: 'add',
        details: 'Add Dinner Recipe',
      });

      expect(mockMealPlanService.updateMealSlot).toHaveBeenCalledWith(mealPlanId, {
        day: 'monday',
        mealType: 'dinner',
        recipeId: 'recipe-4',
        servings: 4,
      });
    });

    test('should remove meals correctly', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-1',
              recipeName: 'Original Recipe',
              servings: 2,
              isLocked: false,
              isCompleted: false,
            },
          },
          // Tuesday lunch is removed from target
        },
      });

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true);
      expect(result.changesApplied).toHaveLength(1);
      expect(result.changesApplied![0]).toEqual({
        day: 'tuesday',
        mealType: 'lunch',
        action: 'remove',
        details: 'Remove Locked Recipe',
      });

      expect(mockMealPlanService.updateMealSlot).toHaveBeenCalledWith(mealPlanId, {
        day: 'tuesday',
        mealType: 'lunch',
        recipeId: null,
      });
    });

    test('should handle no changes needed', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot(); // Same as current

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true);
      expect(result.changesApplied).toHaveLength(0);
      expect(mockMealPlanService.updateMealSlot).not.toHaveBeenCalled();
      expect(mockMealPlanService.toggleMealLock).not.toHaveBeenCalled();
    });

    test('should handle service errors gracefully', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-error',
              recipeName: 'Error Recipe',
              servings: 2,
              isLocked: false,
              isCompleted: false,
            },
          },
          tuesday: {
            lunch: {
              recipeId: 'recipe-2',
              recipeName: 'Locked Recipe',
              servings: 1,
              isLocked: true,
              isCompleted: false,
            },
          },
        },
      });

      mockMealPlanService.updateMealSlot.mockRejectedValueOnce(new Error('Service error'));

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true); // Should continue with other changes
      expect(result.changesApplied).toHaveLength(0); // The failed change is not included
    });

    test('should handle complete failure', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot();

      mockMealPlanService.getMealPlan.mockRejectedValueOnce(new Error('Complete failure'));

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(false);
      expect(result.error).toBe('Complete failure');
    });
  });

  describe('Change Prioritization', () => {
    test('should apply changes in correct priority order', async () => {
      const currentMealPlan = createMockMealPlan();
      const targetSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-new',
              recipeName: 'New Recipe',
              servings: 2,
              isLocked: true, // Will be locked after update
              isCompleted: false,
            },
            lunch: { // Will be added
              recipeId: 'recipe-lunch',
              recipeName: 'Lunch Recipe',
              servings: 2,
              isLocked: false,
              isCompleted: false,
            },
          },
          tuesday: {
            // Lunch will be removed and was locked
          },
        },
      });

      // Mock the calls to track order
      const callOrder: string[] = [];
      mockMealPlanService.updateMealSlot.mockImplementation(async (id, input) => {
        if (input.recipeId === null) {
          callOrder.push('remove');
        } else if (input.recipeId === 'recipe-new') {
          callOrder.push('update');
        } else {
          callOrder.push('add');
        }
        return createMockMealPlan();
      });

      mockMealPlanService.toggleMealLock.mockImplementation(async (id, day, meal, locked) => {
        callOrder.push(locked ? 'lock' : 'unlock');
        return createMockMealPlan();
      });

      const result = await stateService.applySnapshot(currentMealPlan, targetSnapshot);

      expect(result.success).toBe(true);

      // Expected order: remove, unlock, add, update, lock
      expect(callOrder).toEqual(['remove', 'add', 'update', 'lock']);
    });
  });

  describe('Snapshot Validation', () => {
    test('should validate valid snapshots', () => {
      const validSnapshot = createMockSnapshot();
      const validation = stateService.validateSnapshot(validSnapshot);

      expect(validation.valid).toBe(true);
      expect(validation.errors).toHaveLength(0);
    });

    test('should detect missing meal plan ID', () => {
      const invalidSnapshot = createMockSnapshot({ mealPlanId: '' });
      const validation = stateService.validateSnapshot(invalidSnapshot);

      expect(validation.valid).toBe(false);
      expect(validation.errors).toContain('Snapshot missing meal plan ID');
    });

    test('should detect missing meals data', () => {
      const invalidSnapshot = createMockSnapshot();
      // @ts-ignore - Intentionally creating invalid snapshot for testing
      delete invalidSnapshot.meals;

      const validation = stateService.validateSnapshot(invalidSnapshot);

      expect(validation.valid).toBe(false);
      expect(validation.errors).toContain('Snapshot missing meals data');
    });

    test('should detect invalid servings count', () => {
      const invalidSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-1',
              recipeName: 'Test Recipe',
              servings: -1, // Invalid
              isLocked: false,
              isCompleted: false,
            },
          },
        },
      });

      const validation = stateService.validateSnapshot(invalidSnapshot);

      expect(validation.valid).toBe(false);
      expect(validation.errors).toContain('Invalid servings count for monday breakfast');
    });

    test('should detect recipe ID without name', () => {
      const invalidSnapshot = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: 'recipe-1',
              recipeName: '', // Missing name
              servings: 2,
              isLocked: false,
              isCompleted: false,
            },
          },
        },
      });

      const validation = stateService.validateSnapshot(invalidSnapshot);

      expect(validation.valid).toBe(false);
      expect(validation.errors).toContain('Recipe ID without name for monday breakfast');
    });
  });

  describe('Edge Cases', () => {
    test('should handle empty current meal plan', async () => {
      const emptyMealPlan = {
        ...createMockMealPlan(),
        populatedMeals: {},
      };
      
      const targetSnapshot = createMockSnapshot();
      const result = await stateService.applySnapshot(emptyMealPlan, targetSnapshot);

      expect(result.success).toBe(true);
      expect(result.changesApplied!.length).toBeGreaterThan(0);
    });

    test('should handle empty target snapshot', async () => {
      const currentMealPlan = createMockMealPlan();
      const emptySnapshot = createMockSnapshot({ meals: {} });

      const result = await stateService.applySnapshot(currentMealPlan, emptySnapshot);

      expect(result.success).toBe(true);
      // Should remove all existing meals
      expect(result.changesApplied!.every(change => change.action === 'remove')).toBe(true);
    });

    test('should handle snapshot with null/undefined values', async () => {
      const currentMealPlan = createMockMealPlan();
      const snapshotWithNulls = createMockSnapshot({
        meals: {
          monday: {
            breakfast: {
              recipeId: null,
              recipeName: null,
              servings: 0,
              isLocked: false,
              isCompleted: false,
            },
          },
        },
      });

      const result = await stateService.applySnapshot(currentMealPlan, snapshotWithNulls);

      expect(result.success).toBe(true);
      // Should handle null values appropriately
    });
  });
});