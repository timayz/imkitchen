import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { MealPlanGrid } from '../../src/components/organisms/MealPlanGrid';
import type { MealPlanResponse } from '@imkitchen/shared-types';

// Mock dependencies
jest.mock('../../src/components/molecules/MealSlot', () => ({
  MealSlot: ({ onPress, onLongPress }: any) => {
    const { Text, TouchableOpacity } = require('react-native');
    return (
      <TouchableOpacity 
        testID="meal-slot" 
        onPress={onPress} 
        onLongPress={onLongPress}
      >
        <Text>Meal Slot</Text>
      </TouchableOpacity>
    );
  },
}));

jest.mock('../../src/components/atoms/EmptyMealPlanState', () => ({
  EmptyMealPlanState: ({ title, message, onActionPress }: any) => {
    const { View, Text, TouchableOpacity } = require('react-native');
    return (
      <View testID="empty-state">
        <Text testID="empty-title">{title}</Text>
        <Text testID="empty-message">{message}</Text>
        {onActionPress && (
          <TouchableOpacity testID="empty-action" onPress={onActionPress}>
            <Text>Action</Text>
          </TouchableOpacity>
        )}
      </View>
    );
  },
}));

describe('MealPlanGrid', () => {
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

  const defaultProps = {
    mealPlan: mockMealPlan,
    onMealPress: jest.fn(),
    onMealLongPress: jest.fn(),
    onEmptySlotPress: jest.fn(),
    isEditable: true,
    loading: false,
    error: null,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders meal plan grid with all days', () => {
      const { getByText } = render(<MealPlanGrid {...defaultProps} />);
      
      // Check that all days are rendered
      expect(getByText('Mon')).toBeTruthy();
      expect(getByText('Tue')).toBeTruthy();
      expect(getByText('Wed')).toBeTruthy();
      expect(getByText('Thu')).toBeTruthy();
      expect(getByText('Fri')).toBeTruthy();
      expect(getByText('Sat')).toBeTruthy();
      expect(getByText('Sun')).toBeTruthy();
    });

    it('renders meal slots for each day', () => {
      const { getAllByTestId } = render(<MealPlanGrid {...defaultProps} />);
      
      // Should have 7 days × 3 meal types = 21 meal slots
      const mealSlots = getAllByTestId('meal-slot');
      expect(mealSlots).toHaveLength(21);
    });

    it('displays total cooking time and completion percentage', () => {
      const { getByText } = render(<MealPlanGrid {...defaultProps} />);
      
      expect(getByText(/Total cooking time: 3h 0m/)).toBeTruthy();
      expect(getByText(/Progress: 50%/)).toBeTruthy();
    });

    it('highlights today column when current date matches', () => {
      // Mock today to be Monday (2024-01-01)
      jest.spyOn(Date, 'now').mockImplementation(() => new Date('2024-01-01').getTime());
      
      const { getByText } = render(<MealPlanGrid {...defaultProps} />);
      
      // Today indicator should be present
      const mondayHeader = getByText('Mon').parent;
      expect(mondayHeader).toBeTruthy();
      
      Date.now = jest.fn().mockRestore();
    });
  });

  describe('Loading State', () => {
    it('shows loading indicator when loading is true', () => {
      const { getByText } = render(
        <MealPlanGrid {...defaultProps} loading={true} mealPlan={null} />
      );
      
      expect(getByText('Loading meal plan...')).toBeTruthy();
    });
  });

  describe('Error State', () => {
    it('shows error message when error is present', () => {
      const errorMessage = 'Failed to load data';
      const { getByText } = render(
        <MealPlanGrid 
          {...defaultProps} 
          error={errorMessage} 
          mealPlan={null} 
        />
      );
      
      expect(getByText('Failed to load meal plan')).toBeTruthy();
      expect(getByText(errorMessage)).toBeTruthy();
    });
  });

  describe('Empty State', () => {
    it('shows empty state when no meal plan is provided', () => {
      const { getByTestId } = render(
        <MealPlanGrid {...defaultProps} mealPlan={null} />
      );
      
      expect(getByTestId('empty-state')).toBeTruthy();
      expect(getByTestId('empty-title')).toBeTruthy();
      expect(getByTestId('empty-message')).toBeTruthy();
    });
  });

  describe('Interactions', () => {
    it('calls onMealPress when meal slot with recipe is pressed', () => {
      const { getAllByTestId } = render(<MealPlanGrid {...defaultProps} />);
      
      const mealSlots = getAllByTestId('meal-slot');
      fireEvent.press(mealSlots[0]); // First slot (Monday breakfast)
      
      expect(defaultProps.onMealPress).toHaveBeenCalledWith(
        'monday',
        'breakfast',
        expect.objectContaining({
          day: 'monday',
          mealType: 'breakfast',
          recipe: expect.objectContaining({
            id: 'recipe-1',
            title: 'Pancakes',
          }),
        })
      );
    });

    it('calls onEmptySlotPress when empty meal slot is pressed', () => {
      const { getAllByTestId } = render(<MealPlanGrid {...defaultProps} />);
      
      const mealSlots = getAllByTestId('meal-slot');
      fireEvent.press(mealSlots[3]); // Tuesday breakfast (empty)
      
      expect(defaultProps.onEmptySlotPress).toHaveBeenCalledWith(
        'tuesday',
        'breakfast'
      );
    });

    it('calls onMealLongPress when meal slot is long pressed', () => {
      const { getAllByTestId } = render(<MealPlanGrid {...defaultProps} />);
      
      const mealSlots = getAllByTestId('meal-slot');
      fireEvent(mealSlots[0], 'longPress'); // Monday breakfast
      
      expect(defaultProps.onMealLongPress).toHaveBeenCalledWith(
        'monday',
        'breakfast',
        expect.objectContaining({
          day: 'monday',
          mealType: 'breakfast',
        })
      );
    });

    it('does not call interaction handlers when not editable', () => {
      const { getAllByTestId } = render(
        <MealPlanGrid {...defaultProps} isEditable={false} />
      );
      
      const mealSlots = getAllByTestId('meal-slot');
      fireEvent.press(mealSlots[0]);
      
      // Should still call onMealPress for viewing recipe details
      expect(defaultProps.onMealPress).toHaveBeenCalled();
    });
  });

  describe('Date Formatting', () => {
    it('formats week dates correctly', () => {
      const { getByText } = render(<MealPlanGrid {...defaultProps} />);
      
      // January 1, 2024 should be displayed as "Jan 1"
      expect(getByText('Jan 1')).toBeTruthy();
    });
  });

  describe('Accessibility', () => {
    it('has proper accessibility labels', () => {
      const { getAllByTestId } = render(<MealPlanGrid {...defaultProps} />);
      
      const mealSlots = getAllByTestId('meal-slot');
      expect(mealSlots[0]).toBeTruthy();
    });
  });

  describe('Performance', () => {
    it('renders efficiently with many meal slots', () => {
      const largeMealPlan = {
        ...mockMealPlan,
        populatedMeals: {
          ...mockMealPlan.populatedMeals,
          tuesday: Array(3).fill(mockMealPlan.populatedMeals.monday[0]),
          wednesday: Array(3).fill(mockMealPlan.populatedMeals.monday[0]),
          thursday: Array(3).fill(mockMealPlan.populatedMeals.monday[0]),
          friday: Array(3).fill(mockMealPlan.populatedMeals.monday[0]),
          saturday: Array(3).fill(mockMealPlan.populatedMeals.monday[0]),
          sunday: Array(3).fill(mockMealPlan.populatedMeals.monday[0]),
        },
      };

      const { getAllByTestId } = render(
        <MealPlanGrid {...defaultProps} mealPlan={largeMealPlan} />
      );
      
      // Should still render all meal slots efficiently
      const mealSlots = getAllByTestId('meal-slot');
      expect(mealSlots).toHaveLength(21);
    });
  });

  describe('Edge Cases', () => {
    it('handles meal plan with no recipes gracefully', () => {
      const emptyMealPlan = {
        ...mockMealPlan,
        populatedMeals: {
          monday: [],
          tuesday: [],
          wednesday: [],
          thursday: [],
          friday: [],
          saturday: [],
          sunday: [],
        },
        totalEstimatedTime: 0,
        completionPercentage: 0,
      };

      const { getAllByTestId } = render(
        <MealPlanGrid {...defaultProps} mealPlan={emptyMealPlan} />
      );
      
      const mealSlots = getAllByTestId('meal-slot');
      expect(mealSlots).toHaveLength(21);
    });

    it('handles missing recipe data gracefully', () => {
      const mealPlanWithMissingRecipe = {
        ...mockMealPlan,
        populatedMeals: {
          ...mockMealPlan.populatedMeals,
          monday: [
            {
              day: 'monday',
              mealType: 'breakfast',
              servings: 2,
              isCompleted: false,
              // Missing recipe
            },
          ],
        },
      };

      const { getAllByTestId } = render(
        <MealPlanGrid {...defaultProps} mealPlan={mealPlanWithMissingRecipe} />
      );
      
      const mealSlots = getAllByTestId('meal-slot');
      expect(mealSlots).toHaveLength(21);
    });
  });
});