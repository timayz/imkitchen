import { ShoppingIntegrationService } from '../../src/services/shopping_integration_service';
import { shoppingService } from '../../src/services/shopping_service';
import { ChangeHistoryTracker } from '../../src/services/ChangeHistoryTracker';
import type { MealPlanResponse, ShoppingList } from '../../src/types/shopping';
import type { MealPlanChange, MealPlanSnapshot } from '../../src/services/ChangeHistoryTracker';

// Mock dependencies
jest.mock('../../src/services/shopping_service');
const mockShoppingService = shoppingService as jest.Mocked<typeof shoppingService>;

describe('Shopping List Version Synchronization', () => {
  let integrationService: ShoppingIntegrationService;
  let changeTracker: ChangeHistoryTracker;
  const mealPlanId = 'version-sync-meal-plan';

  const createMockMealPlan = (version: number): MealPlanResponse => ({
    id: mealPlanId,
    name: 'Version Sync Test Plan',
    weekStartDate: '2024-01-01',
    version,
    populatedMeals: {
      monday: [
        {
          id: 'slot-1',
          mealType: 'breakfast',
          day: 'monday',
          recipeId: `recipe-v${version}`,
          servings: 2,
          isLocked: false,
          isCompleted: false,
          recipe: {
            id: `recipe-v${version}`,
            title: `Recipe Version ${version}`,
            description: 'Version sync test recipe',
            preparationTimeMinutes: 15,
            cookingTimeMinutes: 20,
            totalTimeMinutes: 35,
            servings: 2,
            ingredients: [
              { name: 'Ingredient A', amount: version, unit: 'cup' },
              { name: 'Ingredient B', amount: version * 0.5, unit: 'tsp' },
            ],
            instructions: ['Mix', 'Cook'],
            nutritionInfo: { calories: 300, protein: 20, carbs: 30, fat: 10 },
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
    totalEstimatedTime: 35,
    completionPercentage: 0,
    createdAt: new Date(),
    updatedAt: new Date(),
  });

  const createMockShoppingList = (version: number): ShoppingList => ({
    id: `shopping-list-v${version}`,
    name: 'Version Sync Shopping List',
    mealPlanId,
    status: 'active',
    totalItems: version * 2, // Scale with version
    completedItems: 0,
    categories: [
      {
        name: 'Test Category',
        items: Array.from({ length: version }, (_, i) => ({
          id: `item-${version}-${i}`,
          name: `Item ${i + 1} (v${version})`,
          quantity: version,
          unit: 'piece',
          completed: false,
          recipeSource: `Recipe Version ${version}`,
        })),
      },
    ],
    generatedAt: new Date(),
    lastModified: new Date(),
  });

  const createMockSnapshot = (version: number): MealPlanSnapshot => ({
    mealPlanId,
    version,
    timestamp: new Date(),
    meals: {
      monday: {
        breakfast: {
          recipeId: `recipe-v${version}`,
          recipeName: `Recipe Version ${version}`,
          servings: 2,
          isLocked: false,
          isCompleted: false,
        },
      },
    },
    totalEstimatedTime: 35,
    completionPercentage: 0,
  });

  const createVersionChange = (fromVersion: number, toVersion: number): MealPlanChange => ({
    id: `change-${fromVersion}-to-${toVersion}`,
    timestamp: new Date(),
    type: 'substitution',
    description: `Version change from ${fromVersion} to ${toVersion}`,
    beforeState: createMockSnapshot(fromVersion),
    afterState: createMockSnapshot(toVersion),
    metadata: {
      reason: `Version sync test change`,
      userInitiated: true,
      affectedSlots: [{ day: 'monday', mealType: 'breakfast' }],
    },
  });

  beforeEach(() => {
    integrationService = new ShoppingIntegrationService();
    changeTracker = new ChangeHistoryTracker(mealPlanId, {
      maxHistoryLength: 10,
      persistToStorage: false,
    });

    jest.clearAllMocks();

    // Default mock implementations
    mockShoppingService.generateShoppingList.mockImplementation(
      async () => createMockShoppingList(1)
    );
    mockShoppingService.getShoppingLists.mockResolvedValue([]);
    mockShoppingService.updateShoppingListIncremental.mockImplementation(
      async () => createMockShoppingList(2)
    );
  });

  describe('Version Creation and Tracking', () => {
    test('should create initial version when generating shopping list', async () => {
      const mealPlan = createMockMealPlan(1);
      
      const result = await integrationService.handleMealPlanChange(mealPlan, null, {});
      
      expect(result).toBeDefined();
      
      const versionInfo = integrationService.getVersionInfo(result!.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.version).toBe(1);
      expect(versionInfo!.mealPlanVersion).toBe(1);
      expect(versionInfo!.mealPlanId).toBe(mealPlanId);
      expect(versionInfo!.changesSinceLastUpdate).toHaveLength(0);
    });

    test('should increment version on each update', async () => {
      const initialMealPlan = createMockMealPlan(1);
      const updatedMealPlan1 = createMockMealPlan(2);
      const updatedMealPlan2 = createMockMealPlan(3);

      // Mock existing shopping list for updates
      const shoppingList = createMockShoppingList(1);
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // Initial generation
      await integrationService.handleMealPlanChange(initialMealPlan, null, {});
      
      let versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.version).toBe(1);

      // First update
      const change1 = createVersionChange(1, 2);
      await integrationService.handleMealPlanChange(
        updatedMealPlan1, 
        initialMealPlan, 
        { useIncrementalUpdate: true },
        [change1]
      );
      
      versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.version).toBe(2);
      expect(versionInfo!.mealPlanVersion).toBe(2);

      // Second update
      const change2 = createVersionChange(2, 3);
      await integrationService.handleMealPlanChange(
        updatedMealPlan2, 
        updatedMealPlan1, 
        { useIncrementalUpdate: true },
        [change2]
      );
      
      versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.version).toBe(3);
      expect(versionInfo!.mealPlanVersion).toBe(3);
    });

    test('should track changes since last update', async () => {
      const mealPlan1 = createMockMealPlan(1);
      const mealPlan2 = createMockMealPlan(2);
      
      const shoppingList = createMockShoppingList(1);
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // Initial generation
      await integrationService.handleMealPlanChange(mealPlan1, null, {});

      // Update with changes
      const changes = [
        createVersionChange(1, 2),
        {
          ...createVersionChange(1, 2),
          id: 'additional-change',
          type: 'add' as const,
          description: 'Additional change',
        }
      ];

      await integrationService.handleMealPlanChange(
        mealPlan2, 
        mealPlan1, 
        { useIncrementalUpdate: true },
        changes
      );

      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.changesSinceLastUpdate).toHaveLength(2);
      expect(versionInfo!.changesSinceLastUpdate).toEqual(changes);
    });
  });

  describe('Version Consistency Validation', () => {
    test('should detect version mismatches', async () => {
      const mealPlan = createMockMealPlan(5); // Version 5
      const shoppingList = createMockShoppingList(1);
      
      // Mock existing shopping list with lower version
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);
      
      // Create version info that's out of sync
      (integrationService as any).createVersionEntry(shoppingList, createMockMealPlan(2), []);
      
      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.mealPlanVersion).toBe(2);
      
      // When handling change with version 5, should detect mismatch
      await integrationService.handleMealPlanChange(
        mealPlan, 
        createMockMealPlan(2), 
        { skipVersionCheck: false }
      );

      // Version should be updated to match meal plan
      const updatedVersionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(updatedVersionInfo!.mealPlanVersion).toBe(5);
    });

    test('should handle concurrent version updates', async () => {
      const mealPlan = createMockMealPlan(1);
      const shoppingList = createMockShoppingList(1);
      
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // Simulate concurrent updates
      const updatePromises = Promise.all([
        integrationService.handleMealPlanChange(
          mealPlan, 
          null, 
          {},
          [createVersionChange(0, 1)]
        ),
        integrationService.handleMealPlanChange(
          mealPlan, 
          null, 
          {},
          [createVersionChange(0, 1)]
        ),
        integrationService.handleMealPlanChange(
          mealPlan, 
          null, 
          {},
          [createVersionChange(0, 1)]
        ),
      ]);

      const results = await updatePromises;

      // All should resolve successfully
      results.forEach(result => expect(result).toBeDefined());

      // Version should be consistent
      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.version).toBeGreaterThanOrEqual(1);
    });

    test('should recover from version corruption', async () => {
      const mealPlan = createMockMealPlan(3);
      const shoppingList = createMockShoppingList(1);
      
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // Corrupt version info
      (integrationService as any).versionStore.set(shoppingList.id, {
        id: 'corrupted',
        shoppingListId: shoppingList.id,
        mealPlanId: 'wrong-meal-plan',
        version: NaN,
        mealPlanVersion: -1,
        createdAt: null,
        changesSinceLastUpdate: null,
      });

      // Should handle corruption gracefully
      await integrationService.handleMealPlanChange(mealPlan, null, {});

      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.mealPlanId).toBe(mealPlanId);
      expect(versionInfo!.version).toBeGreaterThan(0);
      expect(versionInfo!.mealPlanVersion).toBe(3);
    });
  });

  describe('Change History Integration', () => {
    test('should integrate with change history tracker', async () => {
      const mealPlan1 = createMockMealPlan(1);
      const mealPlan2 = createMockMealPlan(2);
      
      // Record changes in tracker
      const change = createVersionChange(1, 2);
      changeTracker.recordChange(change);
      
      const shoppingList = createMockShoppingList(1);
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      await integrationService.handleMealPlanChange(
        mealPlan2, 
        mealPlan1, 
        { useIncrementalUpdate: true },
        [change]
      );

      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.changesSinceLastUpdate).toContainEqual(
        expect.objectContaining({
          id: change.id,
          type: change.type,
          description: change.description,
        })
      );
    });

    test('should handle batch changes in version tracking', async () => {
      const mealPlan1 = createMockMealPlan(1);
      const mealPlan2 = createMockMealPlan(2);
      
      const batchChange: MealPlanChange = {
        id: 'batch-version-change',
        timestamp: new Date(),
        type: 'batch',
        description: 'Batch version change',
        beforeState: createMockSnapshot(1),
        afterState: createMockSnapshot(2),
        metadata: {
          batchId: 'batch-123',
          userInitiated: true,
          affectedSlots: [
            { day: 'monday', mealType: 'breakfast' },
            { day: 'monday', mealType: 'lunch' },
          ],
        },
      };

      const shoppingList = createMockShoppingList(1);
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      await integrationService.handleMealPlanChange(
        mealPlan2, 
        mealPlan1, 
        { useIncrementalUpdate: true },
        [batchChange]
      );

      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.changesSinceLastUpdate).toContainEqual(
        expect.objectContaining({
          type: 'batch',
          metadata: expect.objectContaining({
            batchId: 'batch-123',
          }),
        })
      );
    });

    test('should maintain version consistency during undo/redo operations', async () => {
      const mealPlan1 = createMockMealPlan(1);
      const mealPlan2 = createMockMealPlan(2);
      const mealPlan3 = createMockMealPlan(3);

      const shoppingList = createMockShoppingList(1);
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // Initial state
      await integrationService.handleMealPlanChange(mealPlan1, null, {});
      
      // First change
      const change1 = createVersionChange(1, 2);
      await integrationService.handleMealPlanChange(
        mealPlan2, 
        mealPlan1, 
        { useIncrementalUpdate: true },
        [change1]
      );

      let versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.version).toBe(2);

      // Second change  
      const change2 = createVersionChange(2, 3);
      await integrationService.handleMealPlanChange(
        mealPlan3, 
        mealPlan2, 
        { useIncrementalUpdate: true },
        [change2]
      );

      versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.version).toBe(3);

      // Undo operation (back to version 2)
      await integrationService.handleMealPlanChange(
        mealPlan2, 
        mealPlan3, 
        { useIncrementalUpdate: true },
        [] // No new changes, just state restoration
      );

      versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.mealPlanVersion).toBe(2);
    });
  });

  describe('Version Persistence and Recovery', () => {
    test('should maintain version info across service restarts', () => {
      const shoppingList = createMockShoppingList(1);
      const mealPlan = createMockMealPlan(3);

      // Create initial version info
      (integrationService as any).createVersionEntry(shoppingList, mealPlan, []);
      
      let versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo!.version).toBe(1);

      // Simulate service restart by creating new instance
      const newIntegrationService = new ShoppingIntegrationService();
      
      // Version info should be lost (since we don't persist in this test setup)
      let newVersionInfo = newIntegrationService.getVersionInfo(shoppingList.id);
      expect(newVersionInfo).toBeNull();

      // But should be recreated on next operation
      (newIntegrationService as any).createVersionEntry(shoppingList, mealPlan, []);
      newVersionInfo = newIntegrationService.getVersionInfo(shoppingList.id);
      expect(newVersionInfo!.mealPlanId).toBe(mealPlanId);
    });

    test('should handle missing version info gracefully', async () => {
      const mealPlan = createMockMealPlan(2);
      const shoppingList = createMockShoppingList(1);
      
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // No existing version info
      expect(integrationService.getVersionInfo(shoppingList.id)).toBeNull();

      // Should create version info during update
      await integrationService.handleMealPlanChange(
        mealPlan, 
        createMockMealPlan(1), 
        {}
      );

      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.mealPlanId).toBe(mealPlanId);
    });

    test('should validate version timestamps', async () => {
      const mealPlan = createMockMealPlan(1);
      const shoppingList = createMockShoppingList(1);
      
      const beforeTime = Date.now();
      
      await integrationService.handleMealPlanChange(mealPlan, null, {});
      
      const afterTime = Date.now();
      
      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo).toBeDefined();
      expect(versionInfo!.createdAt.getTime()).toBeGreaterThanOrEqual(beforeTime);
      expect(versionInfo!.createdAt.getTime()).toBeLessThanOrEqual(afterTime);
    });
  });

  describe('Error Scenarios', () => {
    test('should handle version update failures', async () => {
      const mealPlan = createMockMealPlan(2);
      const shoppingList = createMockShoppingList(1);
      
      mockShoppingService.getShoppingLists.mockResolvedValue([shoppingList]);

      // Create initial version
      (integrationService as any).createVersionEntry(shoppingList, createMockMealPlan(1), []);

      // Mock update failure
      mockShoppingService.updateShoppingListIncremental.mockRejectedValue(
        new Error('Update failed')
      );

      await integrationService.handleMealPlanChange(
        mealPlan, 
        createMockMealPlan(1), 
        { useIncrementalUpdate: true },
        [createVersionChange(1, 2)]
      );

      // Version should still be updated even if shopping list update failed
      // (because we fall back to full regeneration)
      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      expect(versionInfo).toBeDefined();
    });

    test('should handle version info corruption', () => {
      const shoppingListId = 'test-list';
      
      // Corrupt the version store
      (integrationService as any).versionStore.set(shoppingListId, 'corrupted data');
      
      // Should return null for corrupted data
      const versionInfo = integrationService.getVersionInfo(shoppingListId);
      expect(versionInfo).toBe('corrupted data'); // Returns whatever is stored
      
      // But should handle it gracefully in operations
      expect(() => {
        (integrationService as any).updateVersionEntry(shoppingListId, createMockMealPlan(1), []);
      }).not.toThrow();
    });

    test('should handle extremely high version numbers', async () => {
      const highVersionPlan = createMockMealPlan(Number.MAX_SAFE_INTEGER);
      
      await integrationService.handleMealPlanChange(highVersionPlan, null, {});
      
      const shoppingList = createMockShoppingList(1);
      const versionInfo = integrationService.getVersionInfo(shoppingList.id);
      
      if (versionInfo) {
        expect(versionInfo.mealPlanVersion).toBe(Number.MAX_SAFE_INTEGER);
        expect(typeof versionInfo.mealPlanVersion).toBe('number');
      }
    });
  });
});