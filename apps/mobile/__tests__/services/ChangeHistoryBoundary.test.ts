import { ChangeHistoryTracker } from '../../src/services/ChangeHistoryTracker';
import { MealPlanStateService } from '../../src/services/MealPlanStateService';
import type { MealPlanSnapshot, MealPlanChange } from '../../src/services/ChangeHistoryTracker';
import type { MealPlanResponse } from '@imkitchen/shared-types';

describe('Change History Boundary Tests', () => {
  let tracker: ChangeHistoryTracker;
  let stateService: MealPlanStateService;
  const mealPlanId = 'boundary-test-plan';

  const createMockSnapshot = (version: number): MealPlanSnapshot => ({
    mealPlanId,
    version,
    timestamp: new Date(Date.now() + version * 1000), // Different timestamps
    meals: {
      monday: {
        breakfast: {
          recipeId: `recipe-${version}`,
          recipeName: `Recipe ${version}`,
          servings: version % 5 + 1, // Vary servings 1-5
          isLocked: version % 2 === 0, // Alternate locked state
          isCompleted: false,
        },
      },
    },
    totalEstimatedTime: version * 10,
    completionPercentage: Math.min(version * 5, 100),
  });

  const createMockChange = (
    version: number,
    type: MealPlanChange['type'] = 'substitution'
  ): Omit<MealPlanChange, 'id' | 'timestamp'> => ({
    type,
    description: `${type} change ${version}`,
    beforeState: createMockSnapshot(version - 1),
    afterState: createMockSnapshot(version),
    metadata: {
      reason: `Boundary test change ${version}`,
      userInitiated: true,
    },
  });

  beforeEach(() => {
    // Use minimal config for boundary testing
    tracker = new ChangeHistoryTracker(mealPlanId, {
      maxHistoryLength: 3, // Very small for boundary testing
      enableBatching: false,
      persistToStorage: false,
    });
    stateService = new MealPlanStateService();
  });

  describe('History Length Boundary Conditions', () => {
    test('should handle exactly max length history', () => {
      const maxLength = 3;
      
      // Fill to exactly max length
      for (let i = 1; i <= maxLength; i++) {
        tracker.recordChange(createMockChange(i));
      }

      const state = tracker.getState();
      expect(state.totalChanges).toBe(maxLength);
      expect(state.currentIndex).toBe(maxLength - 1);
      expect(tracker.canUndo()).toBe(true);
      expect(tracker.canRedo()).toBe(false);

      // Should be able to undo all changes
      for (let i = maxLength - 1; i >= 0; i--) {
        expect(tracker.canUndo()).toBe(true);
        expect(tracker.getState().currentIndex).toBe(i);
        tracker.undo();
      }

      expect(tracker.canUndo()).toBe(false);
      expect(tracker.getState().currentIndex).toBe(-1);
    });

    test('should handle overflow by one', () => {
      const maxLength = 3;
      
      // Fill beyond max length by one
      for (let i = 1; i <= maxLength + 1; i++) {
        tracker.recordChange(createMockChange(i));
      }

      const state = tracker.getState();
      expect(state.totalChanges).toBe(maxLength); // Should be capped
      expect(state.currentIndex).toBe(maxLength - 1);

      // Should still only contain the latest changes
      const history = tracker.getHistory();
      expect(history[0].description).toBe('substitution change 2'); // First should be change 2
      expect(history[2].description).toBe('substitution change 4'); // Last should be change 4
    });

    test('should handle massive overflow', () => {
      const maxLength = 3;
      const overflowAmount = 100;
      
      // Fill way beyond max length
      for (let i = 1; i <= maxLength + overflowAmount; i++) {
        tracker.recordChange(createMockChange(i));
      }

      const state = tracker.getState();
      expect(state.totalChanges).toBe(maxLength); // Should still be capped
      expect(state.currentIndex).toBe(maxLength - 1);

      // Should contain only the latest changes
      const history = tracker.getHistory();
      const expectedStart = overflowAmount + 1;
      expect(history[0].description).toBe(`substitution change ${expectedStart}`);
      expect(history[2].description).toBe(`substitution change ${expectedStart + 2}`);
    });

    test('should handle zero max length', () => {
      const zeroTracker = new ChangeHistoryTracker(mealPlanId, {
        maxHistoryLength: 0,
        enableBatching: false,
        persistToStorage: false,
      });

      zeroTracker.recordChange(createMockChange(1));

      const state = zeroTracker.getState();
      expect(state.totalChanges).toBe(0);
      expect(state.currentIndex).toBe(-1);
      expect(zeroTracker.canUndo()).toBe(false);
      expect(zeroTracker.canRedo()).toBe(false);
    });

    test('should handle single item max length', () => {
      const singleTracker = new ChangeHistoryTracker(mealPlanId, {
        maxHistoryLength: 1,
        enableBatching: false,
        persistToStorage: false,
      });

      singleTracker.recordChange(createMockChange(1));
      singleTracker.recordChange(createMockChange(2));

      const state = singleTracker.getState();
      expect(state.totalChanges).toBe(1);
      expect(state.currentIndex).toBe(0);

      // Should only contain the latest change
      const history = singleTracker.getHistory();
      expect(history[0].description).toBe('substitution change 2');
    });
  });

  describe('Undo/Redo Boundary Conditions', () => {
    test('should handle undo at empty history', () => {
      expect(tracker.canUndo()).toBe(false);
      expect(tracker.undo()).toBeNull();
      
      const state = tracker.getState();
      expect(state.currentIndex).toBe(-1);
      expect(state.totalChanges).toBe(0);
    });

    test('should handle redo at empty history', () => {
      expect(tracker.canRedo()).toBe(false);
      expect(tracker.redo()).toBeNull();
      
      const state = tracker.getState();
      expect(state.currentIndex).toBe(-1);
      expect(state.totalChanges).toBe(0);
    });

    test('should handle undo/redo at single item', () => {
      tracker.recordChange(createMockChange(1));

      // At latest change - can undo but not redo
      expect(tracker.canUndo()).toBe(true);
      expect(tracker.canRedo()).toBe(false);
      expect(tracker.getState().currentIndex).toBe(0);

      // After undo - can redo but not undo
      tracker.undo();
      expect(tracker.canUndo()).toBe(false);
      expect(tracker.canRedo()).toBe(true);
      expect(tracker.getState().currentIndex).toBe(-1);

      // After redo - back to can undo but not redo
      tracker.redo();
      expect(tracker.canUndo()).toBe(true);
      expect(tracker.canRedo()).toBe(false);
      expect(tracker.getState().currentIndex).toBe(0);
    });

    test('should handle rapid undo/redo switching', () => {
      tracker.recordChange(createMockChange(1));
      tracker.recordChange(createMockChange(2));
      tracker.recordChange(createMockChange(3));

      // Rapid switching
      for (let i = 0; i < 10; i++) {
        tracker.undo();
        tracker.redo();
      }

      // Should end up in consistent state
      const state = tracker.getState();
      expect(state.currentIndex).toBe(2); // At latest change
      expect(tracker.canUndo()).toBe(true);
      expect(tracker.canRedo()).toBe(false);
    });

    test('should handle undo all then redo all', () => {
      const numChanges = 3;
      for (let i = 1; i <= numChanges; i++) {
        tracker.recordChange(createMockChange(i));
      }

      // Undo all
      for (let i = 0; i < numChanges; i++) {
        expect(tracker.canUndo()).toBe(true);
        tracker.undo();
      }
      
      expect(tracker.canUndo()).toBe(false);
      expect(tracker.getState().currentIndex).toBe(-1);

      // Redo all
      for (let i = 0; i < numChanges; i++) {
        expect(tracker.canRedo()).toBe(true);
        tracker.redo();
      }
      
      expect(tracker.canRedo()).toBe(false);
      expect(tracker.getState().currentIndex).toBe(numChanges - 1);
    });
  });

  describe('Edge Case Scenarios', () => {
    test('should handle change recording after partial undo with overflow', () => {
      const maxLength = 3;
      
      // Fill to max
      for (let i = 1; i <= maxLength; i++) {
        tracker.recordChange(createMockChange(i));
      }

      // Undo one
      tracker.undo();
      expect(tracker.getState().currentIndex).toBe(1);

      // Add more changes than remaining space
      for (let i = 10; i <= 15; i++) {
        tracker.recordChange(createMockChange(i));
      }

      // Should handle this gracefully
      const state = tracker.getState();
      expect(state.totalChanges).toBe(maxLength);
      expect(state.currentIndex).toBe(maxLength - 1);
      expect(tracker.canRedo()).toBe(false); // Future history should be cleared
    });

    test('should handle interleaved operations with history limits', () => {
      tracker.recordChange(createMockChange(1));
      tracker.recordChange(createMockChange(2));
      
      tracker.undo(); // At change 1
      tracker.recordChange(createMockChange(10)); // Branch from change 1
      tracker.undo(); // Back to change 1
      tracker.undo(); // Back to beginning
      
      tracker.recordChange(createMockChange(20)); // New branch from beginning
      tracker.recordChange(createMockChange(21));
      tracker.recordChange(createMockChange(22));

      // Should maintain consistency
      const state = tracker.getState();
      expect(state.currentIndex).toBeLessThan(state.totalChanges);
      expect(tracker.canUndo()).toBe(true);

      // Should be able to undo to beginning
      while (tracker.canUndo()) {
        tracker.undo();
      }
      expect(tracker.getState().currentIndex).toBe(-1);
    });

    test('should handle concurrent-like operations simulation', () => {
      // Simulate what might happen with rapid user interactions
      const operations = [
        () => tracker.recordChange(createMockChange(1)),
        () => tracker.undo(),
        () => tracker.recordChange(createMockChange(2)),
        () => tracker.recordChange(createMockChange(3)),
        () => tracker.undo(),
        () => tracker.undo(),
        () => tracker.redo(),
        () => tracker.recordChange(createMockChange(4)),
      ];

      // Execute all operations
      operations.forEach(op => op());

      // Should maintain valid state
      const state = tracker.getState();
      expect(state.currentIndex).toBeGreaterThanOrEqual(-1);
      expect(state.currentIndex).toBeLessThan(state.totalChanges);
      
      // Undo/redo capabilities should be consistent
      if (state.currentIndex === -1) {
        expect(tracker.canUndo()).toBe(false);
      } else {
        expect(tracker.canUndo()).toBe(true);
      }

      if (state.currentIndex === state.totalChanges - 1) {
        expect(tracker.canRedo()).toBe(false);
      } else if (state.totalChanges > 0) {
        expect(tracker.canRedo()).toBe(true);
      }
    });
  });

  describe('Data Consistency Boundary Tests', () => {
    test('should maintain referential integrity with extreme changes', () => {
      // Create changes with complex state differences
      const complexChange1: Omit<MealPlanChange, 'id' | 'timestamp'> = {
        type: 'batch',
        description: 'Complex batch operation',
        beforeState: {
          mealPlanId,
          version: 1,
          timestamp: new Date(),
          meals: {},
          totalEstimatedTime: 0,
          completionPercentage: 0,
        },
        afterState: {
          mealPlanId,
          version: 2,
          timestamp: new Date(),
          meals: {
            monday: { breakfast: { recipeId: 'r1', recipeName: 'R1', servings: 1, isLocked: false } },
            tuesday: { lunch: { recipeId: 'r2', recipeName: 'R2', servings: 2, isLocked: true } },
            wednesday: { dinner: { recipeId: 'r3', recipeName: 'R3', servings: 3, isLocked: false } },
            thursday: { breakfast: { recipeId: 'r4', recipeName: 'R4', servings: 4, isLocked: true } },
            friday: { lunch: { recipeId: 'r5', recipeName: 'R5', servings: 5, isLocked: false } },
            saturday: { dinner: { recipeId: 'r6', recipeName: 'R6', servings: 1, isLocked: true } },
            sunday: { breakfast: { recipeId: 'r7', recipeName: 'R7', servings: 2, isLocked: false } },
          },
          totalEstimatedTime: 420, // 7 hours
          completionPercentage: 100,
        },
        metadata: {
          batchId: 'complex-batch-1',
          reason: 'Complex state transformation',
          userInitiated: true,
        },
      };

      tracker.recordChange(complexChange1);

      const result = tracker.undo();
      expect(result).toBeDefined();
      expect(result!.newState).toBe(complexChange1.beforeState);
      expect(result!.change.type).toBe('batch');

      // State should be consistent
      const state = tracker.getState();
      expect(state.currentIndex).toBe(-1);
      expect(tracker.canRedo()).toBe(true);
    });

    test('should handle null and undefined values gracefully', () => {
      const edgeCaseChange: Omit<MealPlanChange, 'id' | 'timestamp'> = {
        type: 'remove',
        description: 'Remove with null values',
        beforeState: {
          mealPlanId,
          version: 1,
          timestamp: new Date(),
          meals: {
            monday: {
              breakfast: {
                recipeId: 'recipe-1',
                recipeName: 'Recipe 1',
                servings: 1,
                isLocked: false,
              },
            },
          },
          totalEstimatedTime: 30,
          completionPercentage: 50,
        },
        afterState: {
          mealPlanId,
          version: 2,
          timestamp: new Date(),
          meals: {
            monday: {
              breakfast: {
                recipeId: undefined,
                recipeName: undefined,
                servings: 0,
                isLocked: false,
                notes: undefined,
              },
            },
          },
          totalEstimatedTime: 0,
          completionPercentage: 0,
        },
        metadata: undefined,
      };

      expect(() => tracker.recordChange(edgeCaseChange)).not.toThrow();
      
      const history = tracker.getHistory();
      expect(history).toHaveLength(1);
      expect(history[0].metadata).toBeUndefined();
    });

    test('should handle very long strings and large numbers', () => {
      const longString = 'A'.repeat(10000);
      const largeNumber = Number.MAX_SAFE_INTEGER;

      const extremeChange: Omit<MealPlanChange, 'id' | 'timestamp'> = {
        type: 'substitution',
        description: longString,
        beforeState: createMockSnapshot(1),
        afterState: {
          ...createMockSnapshot(2),
          version: largeNumber,
          totalEstimatedTime: largeNumber,
        },
        metadata: {
          reason: longString,
          userInitiated: true,
        },
      };

      expect(() => tracker.recordChange(extremeChange)).not.toThrow();
      
      const history = tracker.getHistory();
      expect(history[0].description).toBe(longString);
      expect(history[0].afterState.version).toBe(largeNumber);
    });
  });

  describe('Memory and Performance Boundaries', () => {
    test('should handle minimum viable operations', () => {
      // Test with absolute minimum data
      const minimalChange: Omit<MealPlanChange, 'id' | 'timestamp'> = {
        type: 'add',
        description: '',
        beforeState: {
          mealPlanId,
          version: 1,
          timestamp: new Date(),
          meals: {},
        },
        afterState: {
          mealPlanId,
          version: 2,
          timestamp: new Date(),
          meals: {},
        },
      };

      tracker.recordChange(minimalChange);
      
      expect(tracker.getHistory()).toHaveLength(1);
      expect(tracker.canUndo()).toBe(true);
    });

    test('should maintain performance with rapid operations', () => {
      const startTime = Date.now();
      const numOperations = 1000;

      // Rapid fire operations within history limit
      for (let i = 0; i < numOperations; i++) {
        tracker.recordChange(createMockChange(i % 3 + 1)); // Cycle through 3 changes
        if (i % 10 === 0 && tracker.canUndo()) {
          tracker.undo();
        }
        if (i % 15 === 0 && tracker.canRedo()) {
          tracker.redo();
        }
      }

      const duration = Date.now() - startTime;
      
      // Should complete within reasonable time (adjust threshold as needed)
      expect(duration).toBeLessThan(1000); // 1 second
      
      // Should still maintain correct state
      const state = tracker.getState();
      expect(state.currentIndex).toBeGreaterThanOrEqual(-1);
      expect(state.currentIndex).toBeLessThan(state.totalChanges);
    });
  });
});