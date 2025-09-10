import { act } from '@testing-library/react-native';
import type { MealPlanResponse } from '@imkitchen/shared-types';
import { useMealPlanStore } from '../../src/store/meal_plan_store';
import { mealPlanService } from '../../src/services/meal_plan_service';

// Mock the meal plan service
jest.mock('../../src/services/meal_plan_service', () => ({
  mealPlanService: {
    getMealPlanByWeek: jest.fn(),
    createMealPlan: jest.fn(),
    updateMealPlan: jest.fn(),
    updateMealSlot: jest.fn(),
    deleteMealSlot: jest.fn(),
    moveMeal: jest.fn(),
    deleteMealPlan: jest.fn(),
    getWeekStart: jest.fn((date: Date) => {
      const weekStart = new Date(date);
      const day = weekStart.getDay();
      const diff = weekStart.getDate() - day + (day === 0 ? -6 : 1);
      weekStart.setDate(diff);
      weekStart.setHours(0, 0, 0, 0);
      return weekStart;
    }),
  },
}));

const mockMealPlanService = mealPlanService as jest.Mocked<typeof mealPlanService>;

describe('MealPlanStore', () => {
  const mockMealPlan: MealPlanResponse = {
    id: 'test-meal-plan-1',
    userId: 'user-1',
    weekStartDate: new Date('2024-01-01'),
    generationType: 'manual',
    generatedAt: new Date('2024-01-01'),
    totalEstimatedTime: 180,
    isActive: true,
    status: 'active',
    completionPercentage: 50,
    populatedMeals: {
      monday: [
        {
          day: 'monday',
          mealType: 'breakfast',
          servings: 2,
          isCompleted: false,
          recipe: {
            id: 'recipe-1',
            title: 'Pancakes',
            prepTime: 15,
            cookTime: 10,
            totalTime: 25,
            complexity: 'simple',
            mealType: ['breakfast'],
            servings: 4,
            ingredients: [],
            instructions: [],
            dietaryLabels: [],
            averageRating: 4.5,
            totalRatings: 10,
            createdAt: new Date(),
            updatedAt: new Date(),
          },
        },
      ],
      tuesday: [],
      wednesday: [],
      thursday: [],
      friday: [],
      saturday: [],
      sunday: [],
    },
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01'),
  };

  beforeEach(() => {
    // Reset store state
    useMealPlanStore.getState().reset();
    jest.clearAllMocks();
  });

  describe('Initial State', () => {
    it('has correct initial state', () => {
      const state = useMealPlanStore.getState();
      
      expect(state.currentMealPlan).toBeNull();
      expect(state.mealPlans).toEqual({});
      expect(state.loading).toBe(false);
      expect(state.refreshing).toBe(false);
      expect(state.error).toBeNull();
      expect(state.optimisticUpdates).toEqual([]);
    });

    it('sets current week to start of current week', () => {
      const state = useMealPlanStore.getState();
      const today = new Date();
      const expectedWeekStart = mealPlanService.getWeekStart(today);
      
      expect(state.currentWeek).toEqual(expectedWeekStart);
    });
  });

  describe('Week Navigation', () => {
    it('sets current week correctly', () => {
      const testDate = new Date('2024-01-15');
      const expectedWeekStart = new Date('2024-01-15');
      expectedWeekStart.setHours(0, 0, 0, 0);
      
      act(() => {
        useMealPlanStore.getState().setCurrentWeek(testDate);
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentWeek).toEqual(expectedWeekStart);
    });

    it('loads cached meal plan when changing weeks', () => {
      const weekKey = '2024-01-01';
      
      // Pre-populate cache
      act(() => {
        useMealPlanStore.setState({
          mealPlans: {
            [weekKey]: mockMealPlan,
          },
        });
      });
      
      // Set week to cached week
      act(() => {
        useMealPlanStore.getState().setCurrentWeek(new Date('2024-01-01'));
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
    });
  });

  describe('Loading Meal Plans', () => {
    it('loads meal plan successfully', async () => {
      mockMealPlanService.getMealPlanByWeek.mockResolvedValue(mockMealPlan);
      
      const weekStart = new Date('2024-01-01');
      
      await act(async () => {
        await useMealPlanStore.getState().loadMealPlan(weekStart);
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
      expect(mockMealPlanService.getMealPlanByWeek).toHaveBeenCalledWith(weekStart);
    });

    it('handles meal plan not found gracefully', async () => {
      const notFoundError = new Error('Meal plan not found');
      mockMealPlanService.getMealPlanByWeek.mockRejectedValue(notFoundError);
      
      const weekStart = new Date('2024-01-01');
      
      await act(async () => {
        await useMealPlanStore.getState().loadMealPlan(weekStart);
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull(); // No error for not found
    });

    it('handles other errors correctly', async () => {
      const serverError = new Error('Server error');
      mockMealPlanService.getMealPlanByWeek.mockRejectedValue(serverError);
      
      const weekStart = new Date('2024-01-01');
      
      await act(async () => {
        await useMealPlanStore.getState().loadMealPlan(weekStart);
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.error).toBe('Server error');
    });

    it('prevents duplicate requests for same week', async () => {
      mockMealPlanService.getMealPlanByWeek.mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve(mockMealPlan), 100))
      );
      
      const weekStart = new Date('2024-01-01');
      
      // Start two concurrent requests
      const promise1 = useMealPlanStore.getState().loadMealPlan(weekStart);
      const promise2 = useMealPlanStore.getState().loadMealPlan(weekStart);
      
      await act(async () => {
        await Promise.all([promise1, promise2]);
      });
      
      // Should only call the service once
      expect(mockMealPlanService.getMealPlanByWeek).toHaveBeenCalledTimes(1);
    });

    it('uses cached data when force refresh is false', async () => {
      const weekKey = '2024-01-01';
      
      // Pre-populate cache
      act(() => {
        useMealPlanStore.setState({
          mealPlans: {
            [weekKey]: mockMealPlan,
          },
        });
      });
      
      await act(async () => {
        await useMealPlanStore.getState().loadMealPlan(new Date('2024-01-01'), false);
      });
      
      // Should not call service when using cached data
      expect(mockMealPlanService.getMealPlanByWeek).not.toHaveBeenCalled();
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
    });
  });

  describe('Creating Meal Plans', () => {
    it('creates meal plan successfully', async () => {
      const input = {
        weekStartDate: new Date('2024-01-01'),
        generationType: 'manual' as const,
        meals: {
          monday: [],
          tuesday: [],
          wednesday: [],
          thursday: [],
          friday: [],
          saturday: [],
          sunday: [],
        },
      };
      
      mockMealPlanService.createMealPlan.mockResolvedValue(mockMealPlan);
      
      await act(async () => {
        await useMealPlanStore.getState().createMealPlan(input);
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
      expect(state.error).toBeNull();
      expect(mockMealPlanService.createMealPlan).toHaveBeenCalledWith(input);
    });

    it('handles creation errors correctly', async () => {
      const input = {
        weekStartDate: new Date('2024-01-01'),
        generationType: 'manual' as const,
        meals: {
          monday: [],
          tuesday: [],
          wednesday: [],
          thursday: [],
          friday: [],
          saturday: [],
          sunday: [],
        },
      };
      
      const error = new Error('Creation failed');
      mockMealPlanService.createMealPlan.mockRejectedValue(error);
      
      await act(async () => {
        try {
          await useMealPlanStore.getState().createMealPlan(input);
        } catch (e) {
          // Expected to throw
        }
      });
      
      const state = useMealPlanStore.getState();
      expect(state.error).toBe('Creation failed');
    });
  });

  describe('Updating Meal Slots', () => {
    it('updates meal slot successfully', async () => {
      const updatedMealPlan = { ...mockMealPlan, id: 'updated-meal-plan' };
      mockMealPlanService.updateMealSlot.mockResolvedValue(updatedMealPlan);
      
      await act(async () => {
        await useMealPlanStore.getState().updateMealSlot(
          'meal-plan-1',
          'monday',
          'breakfast',
          { recipeId: 'new-recipe-id' }
        );
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(updatedMealPlan);
      expect(state.error).toBeNull();
      expect(mockMealPlanService.updateMealSlot).toHaveBeenCalledWith(
        'meal-plan-1',
        'monday',
        'breakfast',
        { recipeId: 'new-recipe-id' }
      );
    });
  });

  describe('Deleting Meal Slots', () => {
    it('deletes meal slot successfully', async () => {
      mockMealPlanService.deleteMealSlot.mockResolvedValue(mockMealPlan);
      
      await act(async () => {
        await useMealPlanStore.getState().deleteMealSlot(
          'meal-plan-1',
          'monday',
          'breakfast'
        );
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
      expect(mockMealPlanService.deleteMealSlot).toHaveBeenCalledWith(
        'meal-plan-1',
        'monday',
        'breakfast'
      );
    });
  });

  describe('Moving Meals', () => {
    it('moves meal successfully', async () => {
      mockMealPlanService.moveMeal.mockResolvedValue(mockMealPlan);
      
      await act(async () => {
        await useMealPlanStore.getState().moveMeal(
          'meal-plan-1',
          'monday',
          'breakfast',
          'tuesday',
          'lunch'
        );
      });
      
      const state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
      expect(mockMealPlanService.moveMeal).toHaveBeenCalledWith(
        'meal-plan-1',
        'monday',
        'breakfast',
        'tuesday',
        'lunch'
      );
    });
  });

  describe('Refreshing', () => {
    it('refreshes current meal plan', async () => {
      const weekStart = new Date('2024-01-01');
      
      // Set current week
      act(() => {
        useMealPlanStore.getState().setCurrentWeek(weekStart);
      });
      
      mockMealPlanService.getMealPlanByWeek.mockResolvedValue(mockMealPlan);
      
      await act(async () => {
        await useMealPlanStore.getState().refreshCurrentMealPlan();
      });
      
      const state = useMealPlanStore.getState();
      expect(state.refreshing).toBe(false);
      expect(state.currentMealPlan).toEqual(mockMealPlan);
      expect(mockMealPlanService.getMealPlanByWeek).toHaveBeenCalledWith(weekStart);
    });
  });

  describe('Optimistic Updates', () => {
    it('adds and removes optimistic updates', () => {
      const update = {
        id: 'test-update',
        type: 'update' as const,
        timestamp: Date.now(),
        newData: { test: 'data' },
      };
      
      act(() => {
        useMealPlanStore.getState().addOptimisticUpdate(update);
      });
      
      let state = useMealPlanStore.getState();
      expect(state.optimisticUpdates).toContain(update);
      
      act(() => {
        useMealPlanStore.getState().removeOptimisticUpdate('test-update');
      });
      
      state = useMealPlanStore.getState();
      expect(state.optimisticUpdates).not.toContain(update);
    });
  });

  describe('Error Handling', () => {
    it('clears error correctly', () => {
      act(() => {
        useMealPlanStore.setState({ error: 'Test error' });
      });
      
      let state = useMealPlanStore.getState();
      expect(state.error).toBe('Test error');
      
      act(() => {
        useMealPlanStore.getState().clearError();
      });
      
      state = useMealPlanStore.getState();
      expect(state.error).toBeNull();
    });
  });

  describe('Reset', () => {
    it('resets store to initial state', () => {
      act(() => {
        useMealPlanStore.setState({
          currentMealPlan: mockMealPlan,
          error: 'Test error',
          loading: true,
        });
      });
      
      let state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toEqual(mockMealPlan);
      expect(state.error).toBe('Test error');
      expect(state.loading).toBe(true);
      
      act(() => {
        useMealPlanStore.getState().reset();
      });
      
      state = useMealPlanStore.getState();
      expect(state.currentMealPlan).toBeNull();
      expect(state.error).toBeNull();
      expect(state.loading).toBe(false);
    });
  });
});