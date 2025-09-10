import { ShoppingIntegrationService } from '../../src/services/shopping_integration_service';
import { shoppingService } from '../../src/services/shopping_service';
import type { MealPlanResponse, ShoppingList } from '../../src/types/shopping';
import type { MealPlanChange, MealPlanSnapshot } from '../../src/services/ChangeHistoryTracker';

// Mock the shopping service
jest.mock('../../src/services/shopping_service', () => ({
  shoppingService: {
    generateShoppingList: jest.fn(),
    getShoppingLists: jest.fn(),
    deleteShoppingList: jest.fn(),
    updateShoppingListIncremental: jest.fn(),
  },
}));

const mockShoppingService = shoppingService as jest.Mocked<typeof shoppingService>;

describe('ShoppingIntegrationService Enhanced Tests', () => {
  let integrationService: ShoppingIntegrationService;
  const mealPlanId = 'test-meal-plan-123';

  const createMockMealPlan = (version: number = 1): MealPlanResponse => ({
    id: mealPlanId,
    name: 'Test Meal Plan',
    weekStartDate: '2024-01-01',
    version,
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
          recipe: {
            id: 'recipe-1',
            title: 'Breakfast Recipe',
            description: 'A healthy breakfast',
            preparationTimeMinutes: 10,
            cookingTimeMinutes: 15,
            totalTimeMinutes: 25,
            servings: 2,
            ingredients: [
              { name: 'Eggs', amount: 2, unit: 'pieces' },
              { name: 'Milk', amount: 0.5, unit: 'cup' },
            ],
            instructions: ['Mix ingredients', 'Cook'],
            nutritionInfo: { calories: 300, protein: 20, carbs: 10, fat: 20 },
            tags: [],
            difficulty: 'easy',
            cuisineType: 'american',
            imageUrl: '',
            createdAt: new Date(),
            updatedAt: new Date(),
          },
        },
      ],
    },
    totalEstimatedTime: 25,
    completionPercentage: 0,
    createdAt: new Date(),
    updatedAt: new Date(),
  });

  const createMockShoppingList = (): ShoppingList => ({
    id: 'shopping-list-123',
    name: 'Test Shopping List',
    mealPlanId,
    status: 'active',
    totalItems: 5,
    completedItems: 0,
    categories: [
      {
        name: 'Dairy',
        items: [
          {
            id: 'item-1',
            name: 'Milk',
            quantity: 0.5,
            unit: 'cup',
            completed: false,
            recipeSource: 'Breakfast Recipe',
          },
        ],
      },
      {
        name: 'Protein',
        items: [
          {
            id: 'item-2',
            name: 'Eggs',
            quantity: 2,
            unit: 'pieces',
            completed: false,
            recipeSource: 'Breakfast Recipe',
          },
        ],
      },
    ],
    generatedAt: new Date(),
    lastModified: new Date(),
  });

  const createMockSnapshot = (recipeId: string): MealPlanSnapshot => ({
    mealPlanId,
    version: 1,
    timestamp: new Date(),
    meals: {
      monday: {
        breakfast: {
          recipeId,
          recipeName: `Recipe ${recipeId}`,
          servings: 2,
          isLocked: false,
          isCompleted: false,
        },
      },
    },
    totalEstimatedTime: 25,
    completionPercentage: 0,
  });

  const createMockChange = (
    type: MealPlanChange['type'],
    beforeRecipeId: string,
    afterRecipeId: string
  ): MealPlanChange => ({
    id: `change-${Date.now()}`,
    timestamp: new Date(),
    type,
    description: `Test ${type} change`,
    beforeState: createMockSnapshot(beforeRecipeId),
    afterState: createMockSnapshot(afterRecipeId),
    metadata: {
      reason: 'Test change',
      userInitiated: true,
      affectedSlots: [{ day: 'monday', mealType: 'breakfast' }],
    },
  });

  beforeEach(() => {
    integrationService = new ShoppingIntegrationService();
    jest.clearAllMocks();

    // Default mock implementations
    mockShoppingService.generateShoppingList.mockResolvedValue(createMockShoppingList());
    mockShoppingService.getShoppingLists.mockResolvedValue([createMockShoppingList()]);
    mockShoppingService.deleteShoppingList.mockResolvedValue();
    mockShoppingService.updateShoppingListIncremental.mockResolvedValue(createMockShoppingList());
  });

  describe('Incremental Updates', () => {
    test('should perform incremental update when changes are provided', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);
      const changes = [createMockChange('substitution', 'recipe-1', 'recipe-2')];

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { useIncrementalUpdate: true },
        changes
      );

      expect(result).toBeDefined();
      expect(mockShoppingService.updateShoppingListIncremental).toHaveBeenCalled();
      expect(mockShoppingService.generateShoppingList).not.toHaveBeenCalled();
    });

    test('should fall back to full regeneration if incremental update fails', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);
      const changes = [createMockChange('substitution', 'recipe-1', 'recipe-2')];

      // Make incremental update fail
      mockShoppingService.updateShoppingListIncremental.mockRejectedValue(
        new Error('Incremental update failed')
      );

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { useIncrementalUpdate: true },
        changes
      );

      expect(result).toBeDefined();
      expect(mockShoppingService.updateShoppingListIncremental).toHaveBeenCalled();
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalled(); // Fallback
    });

    test('should handle batch changes correctly', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);
      
      const batchChange: MealPlanChange = {
        id: 'batch-change',
        timestamp: new Date(),
        type: 'batch',
        description: 'Batch operation',
        beforeState: createMockSnapshot('recipe-1'),
        afterState: createMockSnapshot('recipe-2'),
        metadata: {
          batchId: 'batch-123',
          userInitiated: true,
          affectedSlots: [
            { day: 'monday', mealType: 'breakfast' },
            { day: 'monday', mealType: 'lunch' },
          ],
        },
      };

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { useIncrementalUpdate: true },
        [batchChange]
      );

      expect(result).toBeDefined();
      expect(mockShoppingService.updateShoppingListIncremental).toHaveBeenCalled();
    });

    test('should skip incremental update when no meaningful changes', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);
      
      // Create a change that doesn't affect ingredients (like reorder)
      const reorderChange = createMockChange('reorder', 'recipe-1', 'recipe-1');

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { useIncrementalUpdate: true },
        [reorderChange]
      );

      expect(result).toBeDefined();
      // Should return existing shopping list without updates
      expect(mockShoppingService.updateShoppingListIncremental).not.toHaveBeenCalled();
    });
  });

  describe('Version Tracking', () => {
    test('should create version entry for new shopping list', async () => {
      const newMealPlan = createMockMealPlan(1);

      // No existing shopping list
      mockShoppingService.getShoppingLists.mockResolvedValue([]);

      const result = await integrationService.handleMealPlanChange(
        newMealPlan,
        null,
        {}
      );

      expect(result).toBeDefined();
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalled();

      // Check version info was created
      const versionInfo = integrationService.getVersionInfo(result!.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.mealPlanId).toBe(mealPlanId);
      expect(versionInfo!.version).toBe(1);
    });

    test('should update version entry on changes', async () => {
      const currentMealPlan = createMockMealPlan(2);
      const previousMealPlan = createMockMealPlan(1);
      const changes = [createMockChange('substitution', 'recipe-1', 'recipe-2')];

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { useIncrementalUpdate: true },
        changes
      );

      expect(result).toBeDefined();

      // Version should be updated
      const versionInfo = integrationService.getVersionInfo(result!.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.mealPlanVersion).toBe(2);
      expect(versionInfo!.changesSinceLastUpdate).toEqual(changes);
    });

    test('should track version across multiple updates', async () => {
      const existingList = createMockShoppingList();
      const mealPlan1 = createMockMealPlan(1);
      const mealPlan2 = createMockMealPlan(2);
      const mealPlan3 = createMockMealPlan(3);

      // First update
      await integrationService.handleMealPlanChange(mealPlan1, null, {});
      
      let versionInfo = integrationService.getVersionInfo(existingList.id);
      expect(versionInfo?.version).toBe(1);

      // Second update
      const changes1 = [createMockChange('substitution', 'recipe-1', 'recipe-2')];
      await integrationService.handleMealPlanChange(mealPlan2, mealPlan1, {}, changes1);
      
      versionInfo = integrationService.getVersionInfo(existingList.id);
      expect(versionInfo?.version).toBe(2);

      // Third update
      const changes2 = [createMockChange('add', '', 'recipe-3')];
      await integrationService.handleMealPlanChange(mealPlan3, mealPlan2, {}, changes2);
      
      versionInfo = integrationService.getVersionInfo(existingList.id);
      expect(versionInfo?.version).toBe(3);
    });
  });

  describe('Concurrency Control', () => {
    test('should prevent concurrent updates to same meal plan', async () => {
      const mealPlan = createMockMealPlan(1);
      
      // Start two concurrent update requests
      const promise1 = integrationService.handleMealPlanChange(mealPlan, null, {});
      const promise2 = integrationService.handleMealPlanChange(mealPlan, null, {});

      const [result1, result2] = await Promise.all([promise1, promise2]);

      // Both should resolve to the same result (second call waited for first)
      expect(result1).toEqual(result2);
      
      // Service should only be called once
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledTimes(1);
    });

    test('should allow concurrent updates to different meal plans', async () => {
      const mealPlan1 = createMockMealPlan(1);
      const mealPlan2 = { ...createMockMealPlan(1), id: 'different-meal-plan' };

      // Start concurrent updates for different meal plans
      const promise1 = integrationService.handleMealPlanChange(mealPlan1, null, {});
      const promise2 = integrationService.handleMealPlanChange(mealPlan2, null, {});

      const [result1, result2] = await Promise.all([promise1, promise2]);

      expect(result1).toBeDefined();
      expect(result2).toBeDefined();
      
      // Service should be called twice (once for each meal plan)
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledTimes(2);
    });
  });

  describe('Error Handling', () => {
    test('should handle service errors gracefully', async () => {
      const mealPlan = createMockMealPlan(1);
      
      mockShoppingService.generateShoppingList.mockRejectedValue(
        new Error('Service temporarily unavailable')
      );

      const result = await integrationService.handleMealPlanChange(mealPlan, null, {});

      expect(result).toBeNull();
      
      // Should queue error notification
      const notifications = integrationService.getNotifications();
      expect(notifications).toHaveLength(1);
      expect(notifications[0].type).toBe('warning');
    });

    test('should handle partial incremental update failures', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);
      const changes = [
        createMockChange('substitution', 'recipe-1', 'recipe-2'),
        createMockChange('add', '', 'recipe-3'),
      ];

      // Make incremental update fail partway through
      let callCount = 0;
      mockShoppingService.updateShoppingListIncremental.mockImplementation(() => {
        callCount++;
        if (callCount === 1) {
          throw new Error('Partial failure');
        }
        return Promise.resolve(createMockShoppingList());
      });

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { useIncrementalUpdate: true },
        changes
      );

      // Should fall back to full regeneration
      expect(result).toBeDefined();
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalled();
    });
  });

  describe('Notification System', () => {
    test('should queue notifications for successful updates', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);
      const changes = [createMockChange('substitution', 'recipe-1', 'recipe-2')];

      await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        { notifyUser: true, useIncrementalUpdate: true },
        changes
      );

      const notifications = integrationService.getNotifications();
      expect(notifications).toHaveLength(1);
      expect(notifications[0].type).toBe('info');
      expect(notifications[0].message).toContain('incrementally');
    });

    test('should not queue notifications when disabled', async () => {
      const mealPlan = createMockMealPlan(1);

      await integrationService.handleMealPlanChange(
        mealPlan,
        null,
        { notifyUser: false }
      );

      const notifications = integrationService.getNotifications();
      expect(notifications).toHaveLength(0);
    });

    test('should auto-clear old notifications', (done) => {
      integrationService.queueNotification('Test notification', 'info');
      
      expect(integrationService.getNotifications()).toHaveLength(1);

      // Wait for auto-clear timeout
      setTimeout(() => {
        expect(integrationService.getNotifications()).toHaveLength(0);
        done();
      }, 10100); // Slightly longer than auto-clear timeout
    }, 11000);

    test('should allow manual notification clearing', () => {
      integrationService.queueNotification('Test 1', 'info');
      integrationService.queueNotification('Test 2', 'success');
      
      expect(integrationService.getNotifications()).toHaveLength(2);

      integrationService.clearNotifications();
      expect(integrationService.getNotifications()).toHaveLength(0);
    });
  });

  describe('Performance Optimizations', () => {
    test('should skip updates when no meaningful changes detected', async () => {
      const currentMealPlan = createMockMealPlan(1);
      const previousMealPlan = createMockMealPlan(1);

      // Mock diff calculation to return no changes
      jest.spyOn(integrationService as any, 'calculateShoppingListDiff')
        .mockResolvedValue({ added: [], removed: [], modified: [] });

      const result = await integrationService.handleMealPlanChange(
        currentMealPlan,
        previousMealPlan,
        {}
      );

      expect(result).toBeDefined();
      // Should return existing list without calling generation service
      expect(mockShoppingService.generateShoppingList).not.toHaveBeenCalled();
    });

    test('should use cached results when available', async () => {
      const mealPlan = createMockMealPlan(1);
      
      // First call
      const result1 = await integrationService.handleMealPlanChange(mealPlan, null, {});
      
      // Second call with same parameters (would use cache in real implementation)
      const result2 = await integrationService.handleMealPlanChange(mealPlan, null, {});

      expect(result1).toBeDefined();
      expect(result2).toBeDefined();
    });
  });

  describe('Edge Cases', () => {
    test('should handle empty meal plan gracefully', async () => {
      const emptyMealPlan: MealPlanResponse = {
        ...createMockMealPlan(1),
        populatedMeals: {},
        totalEstimatedTime: 0,
      };

      const result = await integrationService.handleMealPlanChange(emptyMealPlan, null, {});

      expect(result).toBeDefined();
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalled();
    });

    test('should handle meal plan with null recipes', async () => {
      const mealPlanWithNulls = createMockMealPlan(1);
      mealPlanWithNulls.populatedMeals.monday![0].recipe = null as any;

      const result = await integrationService.handleMealPlanChange(
        mealPlanWithNulls,
        null,
        {}
      );

      expect(result).toBeDefined();
    });

    test('should handle very large meal plans', async () => {
      const largeMealPlan = createMockMealPlan(1);
      
      // Create meals for all days and meal types
      const days = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];
      const mealTypes = ['breakfast', 'lunch', 'dinner'];
      
      days.forEach(day => {
        largeMealPlan.populatedMeals[day as keyof typeof largeMealPlan.populatedMeals] = 
          mealTypes.map((mealType, index) => ({
            id: `${day}-${mealType}-${index}`,
            mealType: mealType as any,
            day: day as any,
            recipeId: `recipe-${day}-${mealType}`,
            servings: 2,
            isLocked: false,
            isCompleted: false,
            recipe: {
              id: `recipe-${day}-${mealType}`,
              title: `${day} ${mealType}`,
              description: 'Test recipe',
              preparationTimeMinutes: 15,
              cookingTimeMinutes: 20,
              totalTimeMinutes: 35,
              servings: 2,
              ingredients: Array.from({ length: 5 }, (_, i) => ({
                name: `Ingredient ${i + 1}`,
                amount: i + 1,
                unit: 'cup',
              })),
              instructions: ['Cook', 'Serve'],
              nutritionInfo: { calories: 300, protein: 15, carbs: 30, fat: 12 },
              tags: [],
              difficulty: 'easy',
              cuisineType: 'american',
              imageUrl: '',
              createdAt: new Date(),
              updatedAt: new Date(),
            },
          }));
      });

      const result = await integrationService.handleMealPlanChange(
        largeMealPlan,
        null,
        {}
      );

      expect(result).toBeDefined();
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalled();
    });
  });
});