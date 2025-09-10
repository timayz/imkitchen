import { ChangeHistoryTracker } from '../../src/services/ChangeHistoryTracker';
import type { MealPlanSnapshot, MealPlanChange } from '../../src/services/ChangeHistoryTracker';

// Mock localStorage for testing
const localStorageMock = (() => {
  let store: { [key: string]: string } = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('ChangeHistoryTracker', () => {
  let tracker: ChangeHistoryTracker;
  const mealPlanId = 'test-meal-plan-123';

  const createMockSnapshot = (version: number = Date.now()): MealPlanSnapshot => ({
    mealPlanId,
    version,
    timestamp: new Date(),
    meals: {
      monday: {
        breakfast: {
          recipeId: 'recipe-1',
          recipeName: 'Test Recipe',
          servings: 2,
          isLocked: false,
        },
      },
    },
    totalEstimatedTime: 30,
    completionPercentage: 0,
  });

  const createMockChange = (
    type: MealPlanChange['type'] = 'substitution',
    beforeSnapshot?: MealPlanSnapshot,
    afterSnapshot?: MealPlanSnapshot
  ): Omit<MealPlanChange, 'id' | 'timestamp'> => ({
    type,
    description: `Test ${type} change`,
    beforeState: beforeSnapshot || createMockSnapshot(1),
    afterState: afterSnapshot || createMockSnapshot(2),
    metadata: {
      reason: 'Test change',
      userInitiated: true,
    },
  });

  beforeEach(() => {
    localStorageMock.clear();
    tracker = new ChangeHistoryTracker(mealPlanId, {
      maxHistoryLength: 5, // Small limit for testing
      enableBatching: true,
      batchTimeout: 100,
      persistToStorage: false, // Disable for most tests
    });
  });

  describe('Basic Change Recording', () => {
    test('should record a change successfully', () => {
      const change = createMockChange();
      const changeId = tracker.recordChange(change);

      expect(changeId).toBeDefined();
      expect(changeId).toMatch(/^change_test-meal-plan-123_/);

      const history = tracker.getHistory();
      expect(history).toHaveLength(1);
      expect(history[0].id).toBe(changeId);
      expect(history[0].type).toBe('substitution');
    });

    test('should maintain current index correctly', () => {
      const change1 = createMockChange('substitution');
      const change2 = createMockChange('swap');

      tracker.recordChange(change1);
      tracker.recordChange(change2);

      const state = tracker.getState();
      expect(state.currentIndex).toBe(1);
      expect(state.totalChanges).toBe(2);
      expect(state.canUndo).toBe(true);
      expect(state.canRedo).toBe(false);
    });

    test('should generate unique change IDs', () => {
      const change1 = createMockChange();
      const change2 = createMockChange();

      const id1 = tracker.recordChange(change1);
      const id2 = tracker.recordChange(change2);

      expect(id1).not.toBe(id2);
    });
  });

  describe('Undo/Redo Operations', () => {
    test('should undo changes correctly', () => {
      const beforeSnapshot = createMockSnapshot(1);
      const afterSnapshot = createMockSnapshot(2);
      const change = createMockChange('substitution', beforeSnapshot, afterSnapshot);

      tracker.recordChange(change);

      // Should be able to undo
      expect(tracker.canUndo()).toBe(true);
      expect(tracker.canRedo()).toBe(false);

      const undoResult = tracker.undo();
      expect(undoResult).toBeDefined();
      expect(undoResult!.change.type).toBe('substitution');
      expect(undoResult!.newState).toBe(beforeSnapshot);

      // After undo, should be able to redo but not undo
      expect(tracker.canUndo()).toBe(false);
      expect(tracker.canRedo()).toBe(true);
    });

    test('should redo changes correctly', () => {
      const beforeSnapshot = createMockSnapshot(1);
      const afterSnapshot = createMockSnapshot(2);
      const change = createMockChange('substitution', beforeSnapshot, afterSnapshot);

      tracker.recordChange(change);
      tracker.undo();

      const redoResult = tracker.redo();
      expect(redoResult).toBeDefined();
      expect(redoResult!.change.type).toBe('substitution');
      expect(redoResult!.newState).toBe(afterSnapshot);

      expect(tracker.canUndo()).toBe(true);
      expect(tracker.canRedo()).toBe(false);
    });

    test('should handle multiple undo/redo operations', () => {
      const changes = [
        createMockChange('substitution', createMockSnapshot(1), createMockSnapshot(2)),
        createMockChange('lock', createMockSnapshot(2), createMockSnapshot(3)),
        createMockChange('swap', createMockSnapshot(3), createMockSnapshot(4)),
      ];

      changes.forEach(change => tracker.recordChange(change));

      // Undo all changes
      expect(tracker.getState().currentIndex).toBe(2);
      
      tracker.undo();
      expect(tracker.getState().currentIndex).toBe(1);
      
      tracker.undo();
      expect(tracker.getState().currentIndex).toBe(0);
      
      tracker.undo();
      expect(tracker.getState().currentIndex).toBe(-1);
      expect(tracker.canUndo()).toBe(false);

      // Redo all changes
      tracker.redo();
      expect(tracker.getState().currentIndex).toBe(0);
      
      tracker.redo();
      expect(tracker.getState().currentIndex).toBe(1);
      
      tracker.redo();
      expect(tracker.getState().currentIndex).toBe(2);
      expect(tracker.canRedo()).toBe(false);
    });

    test('should return null when no undo/redo available', () => {
      // No changes recorded
      expect(tracker.undo()).toBeNull();
      expect(tracker.redo()).toBeNull();

      const change = createMockChange();
      tracker.recordChange(change);

      // Cannot redo when at latest change
      expect(tracker.redo()).toBeNull();

      tracker.undo();

      // Cannot undo when at beginning
      expect(tracker.undo()).toBeNull();
    });
  });

  describe('History Management', () => {
    test('should enforce history limit', () => {
      const maxLength = 5;
      
      // Add more changes than the limit
      for (let i = 0; i < maxLength + 3; i++) {
        tracker.recordChange(createMockChange('substitution', createMockSnapshot(i), createMockSnapshot(i + 1)));
      }

      const history = tracker.getHistory();
      expect(history).toHaveLength(maxLength);

      // Should still be able to undo up to the limit
      let undoCount = 0;
      while (tracker.canUndo()) {
        tracker.undo();
        undoCount++;
      }
      
      expect(undoCount).toBe(maxLength);
    });

    test('should clear history correctly', () => {
      tracker.recordChange(createMockChange());
      tracker.recordChange(createMockChange());

      expect(tracker.getHistory()).toHaveLength(2);

      tracker.clear();

      expect(tracker.getHistory()).toHaveLength(0);
      expect(tracker.canUndo()).toBe(false);
      expect(tracker.canRedo()).toBe(false);
      expect(tracker.getState().currentIndex).toBe(-1);
    });

    test('should truncate future history when recording after undo', () => {
      const change1 = createMockChange('substitution');
      const change2 = createMockChange('lock');
      const change3 = createMockChange('swap');

      tracker.recordChange(change1);
      tracker.recordChange(change2);
      tracker.undo(); // Now at change1

      // Recording new change should remove change2 from history
      tracker.recordChange(change3);

      const history = tracker.getHistory();
      expect(history).toHaveLength(2);
      expect(history[1].type).toBe('swap');
      expect(tracker.canRedo()).toBe(false);
    });

    test('should get recent changes correctly', () => {
      for (let i = 0; i < 10; i++) {
        tracker.recordChange(createMockChange('substitution'));
      }

      const recentChanges = tracker.getRecentChanges(3);
      expect(recentChanges).toHaveLength(3);

      // Should be in reverse order (most recent first)
      expect(recentChanges[0]).toBe(tracker.getHistory()[9]);
      expect(recentChanges[1]).toBe(tracker.getHistory()[8]);
      expect(recentChanges[2]).toBe(tracker.getHistory()[7]);
    });
  });

  describe('Batch Operations', () => {
    test('should handle batch operations', async () => {
      const batchId = tracker.startBatch('Test batch operation');
      expect(batchId).toBeDefined();

      const change1 = createMockChange('lock');
      const change2 = createMockChange('unlock');

      // Add changes to batch (would be triggered by batching logic)
      tracker.recordChange(change1);
      tracker.recordChange(change2);

      const batchChange = tracker.endBatch();
      expect(batchChange).toBeDefined();
      expect(batchChange!.type).toBe('batch');
      expect(batchChange!.description).toBe('Test batch operation');
    });

    test('should handle empty batch', () => {
      tracker.startBatch('Empty batch');
      const batchChange = tracker.endBatch();
      expect(batchChange).toBeNull();
    });

    test('should handle nested batches', () => {
      tracker.startBatch('First batch');
      tracker.recordChange(createMockChange());
      
      // Starting second batch should end first
      tracker.startBatch('Second batch');
      tracker.recordChange(createMockChange());
      
      const secondBatchChange = tracker.endBatch();
      expect(secondBatchChange!.description).toBe('Second batch');
      
      const history = tracker.getHistory();
      // Should have both individual changes plus the batch change
      expect(history.length).toBeGreaterThan(0);
    });
  });

  describe('Snapshot Creation', () => {
    test('should create snapshots correctly', () => {
      const mockMeals = {
        monday: [
          {
            id: 'slot-1',
            mealType: 'breakfast' as const,
            day: 'monday' as const,
            recipeId: 'recipe-1',
            servings: 2,
            isLocked: false,
            isCompleted: true,
            notes: 'Test notes',
            recipe: {
              id: 'recipe-1',
              title: 'Test Recipe',
              description: 'A test recipe',
              preparationTimeMinutes: 30,
              cookingTimeMinutes: 15,
              totalTimeMinutes: 45,
              servings: 4,
              ingredients: [],
              instructions: [],
              nutritionInfo: {
                calories: 250,
                protein: 15,
                carbs: 30,
                fat: 10,
              },
              tags: [],
              difficulty: 'easy' as const,
              cuisineType: 'american' as const,
              imageUrl: 'test-image.jpg',
              createdAt: new Date(),
              updatedAt: new Date(),
            },
          },
        ],
      };

      const snapshot = tracker.createSnapshot(mockMeals, {
        totalEstimatedTime: 45,
        completionPercentage: 50,
      });

      expect(snapshot.mealPlanId).toBe(mealPlanId);
      expect(snapshot.totalEstimatedTime).toBe(45);
      expect(snapshot.completionPercentage).toBe(50);
      expect(snapshot.meals.monday?.breakfast).toEqual({
        recipeId: 'recipe-1',
        recipeName: 'Test Recipe',
        servings: 2,
        isLocked: false,
        isCompleted: true,
        notes: 'Test notes',
      });
    });
  });

  describe('Description Generation', () => {
    test('should generate descriptions for different change types', () => {
      const testCases = [
        {
          type: 'substitution' as const,
          expected: 'Replaced Monday Breakfast with Test Recipe',
        },
        {
          type: 'swap' as const,
          expected: 'Swapped Monday Breakfast to Test Recipe',
        },
        {
          type: 'reorder' as const,
          expected: 'Moved Monday Breakfast to Tuesday Lunch',
        },
        {
          type: 'lock' as const,
          expected: 'Locked Monday Breakfast',
        },
        {
          type: 'unlock' as const,
          expected: 'Unlocked Monday Breakfast',
        },
        {
          type: 'add' as const,
          expected: 'Added Test Recipe to Monday Breakfast',
        },
        {
          type: 'remove' as const,
          expected: 'Removed Test Recipe from Monday Breakfast',
        },
      ];

      testCases.forEach(({ type, expected }) => {
        const description = tracker.generateDescription(
          type,
          'monday',
          'breakfast',
          'Test Recipe',
          'tuesday',
          'lunch'
        );
        expect(description).toBe(expected);
      });
    });
  });

  describe('Persistence', () => {
    test('should persist history to localStorage', () => {
      const persistentTracker = new ChangeHistoryTracker(mealPlanId, {
        persistToStorage: true,
      });

      persistentTracker.recordChange(createMockChange());
      
      const stored = localStorage.getItem(`meal_plan_history_${mealPlanId}`);
      expect(stored).toBeDefined();
      
      const parsedData = JSON.parse(stored!);
      expect(parsedData.mealPlanId).toBe(mealPlanId);
      expect(parsedData.history).toHaveLength(1);
    });

    test('should load persisted history', () => {
      // Create and persist some history
      const tracker1 = new ChangeHistoryTracker(mealPlanId, {
        persistToStorage: true,
      });
      
      tracker1.recordChange(createMockChange('substitution'));
      tracker1.recordChange(createMockChange('lock'));

      // Create new tracker instance - should load persisted history
      const tracker2 = new ChangeHistoryTracker(mealPlanId, {
        persistToStorage: true,
      });

      expect(tracker2.getHistory()).toHaveLength(2);
      expect(tracker2.getState().currentIndex).toBe(1);
    });
  });
});