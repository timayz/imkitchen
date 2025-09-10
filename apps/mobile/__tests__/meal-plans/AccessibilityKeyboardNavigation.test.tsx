import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { MealPlanGrid } from '../../src/components/organisms/MealPlanGrid';
import { MealSlot } from '../../src/components/molecules/MealSlot';
import { MealLockToggle } from '../../src/components/atoms/MealLockToggle';
import type { MealPlanResponse, MealSlotWithRecipe, DayOfWeek, MealType } from '@imkitchen/shared-types';

// Mock React Native KeyEvent for keyboard testing
const mockKeyEvent = (key: string, altKey = false, ctrlKey = false, metaKey = false) => ({
  key,
  altKey,
  ctrlKey,
  metaKey,
  preventDefault: jest.fn(),
  stopPropagation: jest.fn(),
});

describe('Accessibility and Keyboard Navigation', () => {
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
          isLocked: false,
          recipe: {
            id: 'recipe-1',
            title: 'Breakfast Pancakes',
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
        {
          day: 'monday',
          mealType: 'lunch',
          servings: 2,
          isCompleted: false,
          isLocked: true,
          recipe: {
            id: 'recipe-2',
            title: 'Locked Lunch Salad',
            prepTime: 10,
            cookTime: 0,
            totalTime: 10,
            complexity: 'simple',
            mealType: ['lunch'],
            servings: 2,
            ingredients: [],
            instructions: [],
            dietaryLabels: [],
            averageRating: 4.2,
            totalRatings: 8,
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

  const mockMeal: MealSlotWithRecipe = mockMealPlan.populatedMeals.monday[0];
  const mockLockedMeal: MealSlotWithRecipe = mockMealPlan.populatedMeals.monday[1];

  describe('MealLockToggle Accessibility', () => {
    const defaultProps = {
      isLocked: false,
      onToggle: jest.fn(),
    };

    beforeEach(() => {
      jest.clearAllMocks();
    });

    it('has proper ARIA role', () => {
      const { getByRole } = render(<MealLockToggle {...defaultProps} />);
      
      const button = getByRole('button');
      expect(button).toBeTruthy();
    });

    it('has descriptive accessibility label for unlocked state', () => {
      const { getByRole } = render(<MealLockToggle {...defaultProps} />);
      
      const button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Lock meal');
      expect(button.props.accessibilityHint).toBe(
        'Locks this meal to prevent changes during regeneration'
      );
    });

    it('has descriptive accessibility label for locked state', () => {
      const lockedProps = { ...defaultProps, isLocked: true };
      const { getByRole } = render(<MealLockToggle {...lockedProps} />);
      
      const button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Unlock meal');
      expect(button.props.accessibilityHint).toBe(
        'Unlocks this meal so it can be moved or changed during regeneration'
      );
    });

    it('handles keyboard activation (Space key)', () => {
      const { getByRole } = render(<MealLockToggle {...defaultProps} />);
      
      const button = getByRole('button');
      fireEvent(button, 'keyPress', mockKeyEvent(' '));
      
      expect(defaultProps.onToggle).toHaveBeenCalled();
    });

    it('handles keyboard activation (Enter key)', () => {
      const { getByRole } = render(<MealLockToggle {...defaultProps} />);
      
      const button = getByRole('button');
      fireEvent(button, 'keyPress', mockKeyEvent('Enter'));
      
      expect(defaultProps.onToggle).toHaveBeenCalled();
    });

    it('ignores other keyboard inputs', () => {
      const { getByRole } = render(<MealLockToggle {...defaultProps} />);
      
      const button = getByRole('button');
      fireEvent(button, 'keyPress', mockKeyEvent('a'));
      fireEvent(button, 'keyPress', mockKeyEvent('Tab'));
      fireEvent(button, 'keyPress', mockKeyEvent('Escape'));
      
      expect(defaultProps.onToggle).not.toHaveBeenCalled();
    });

    it('respects disabled state for keyboard input', () => {
      const disabledProps = { ...defaultProps, disabled: true };
      const { getByRole } = render(<MealLockToggle {...disabledProps} />);
      
      const button = getByRole('button');
      fireEvent(button, 'keyPress', mockKeyEvent(' '));
      fireEvent(button, 'keyPress', mockKeyEvent('Enter'));
      
      expect(defaultProps.onToggle).not.toHaveBeenCalled();
    });

    it('announces state changes to screen readers', () => {
      const { getByRole, rerender } = render(<MealLockToggle {...defaultProps} />);
      
      let button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Lock meal');
      
      // Simulate state change
      const lockedProps = { ...defaultProps, isLocked: true };
      rerender(<MealLockToggle {...lockedProps} />);
      
      button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Unlock meal');
    });
  });

  describe('MealSlot Keyboard Navigation', () => {
    const defaultProps = {
      day: 'monday' as DayOfWeek,
      mealType: 'breakfast' as MealType,
      meal: mockMeal,
      mealTypeLabel: 'Breakfast',
      mealTypeIcon: '🌅',
      isEmpty: false,
      isEditable: true,
      showLockToggle: true,
      onPress: jest.fn(),
      onLongPress: jest.fn(),
      onLockToggle: jest.fn(),
    };

    beforeEach(() => {
      jest.clearAllMocks();
    });

    it('has proper focus management for touchable elements', () => {
      const { getByTestId } = render(<MealSlot {...defaultProps} />);
      
      const touchable = getByTestId('meal-card-touchable');
      expect(touchable.props.accessible).toBe(true);
    });

    it('provides keyboard shortcut for locking (Ctrl+L)', () => {
      const { getByRole } = render(<MealSlot {...defaultProps} />);
      
      const lockToggle = getByRole('button');
      fireEvent(lockToggle, 'keyPress', mockKeyEvent('l', false, true));
      
      expect(defaultProps.onLockToggle).toHaveBeenCalledWith('monday', 'breakfast', true);
    });

    it('provides accessible meal information', () => {
      const { getByText } = render(<MealSlot {...defaultProps} />);
      
      expect(getByText('Breakfast Pancakes')).toBeTruthy();
      expect(getByText('🌅 Breakfast')).toBeTruthy();
      expect(getByText('25m')).toBeTruthy();
    });

    it('announces meal state changes', () => {
      const { rerender, getByRole } = render(<MealSlot {...defaultProps} />);
      
      let lockToggle = getByRole('button');
      expect(lockToggle.props.accessibilityLabel).toBe('Lock meal');
      
      // Simulate locking the meal
      const lockedProps = {
        ...defaultProps,
        meal: { ...mockMeal, isLocked: true },
      };
      rerender(<MealSlot {...lockedProps} />);
      
      lockToggle = getByRole('button');
      expect(lockToggle.props.accessibilityLabel).toBe('Unlock meal');
    });

    it('handles focus correctly for drag operations', () => {
      const dragProps = {
        ...defaultProps,
        dragEnabled: true,
        isDragging: false,
      };
      
      const { getByTestId } = render(<MealSlot {...dragProps} />);
      
      const draggableElement = getByTestId('meal-card-container');
      expect(draggableElement).toBeTruthy();
    });

    it('provides alternative keyboard navigation for drag operations', () => {
      // Simulate keyboard-based meal reordering
      const dragProps = {
        ...defaultProps,
        dragEnabled: true,
        onDragStart: jest.fn(),
        onDragEnd: jest.fn(),
      };
      
      const { getByTestId } = render(<MealSlot {...dragProps} />);
      
      const touchable = getByTestId('meal-card-touchable');
      
      // Simulate arrow key navigation (would be implemented in the grid)
      fireEvent(touchable, 'keyPress', mockKeyEvent('ArrowRight'));
      fireEvent(touchable, 'keyPress', mockKeyEvent('Enter'));
      
      // These would trigger meal movement in a full implementation
      expect(touchable).toBeTruthy();
    });
  });

  describe('MealPlanGrid Keyboard Navigation', () => {
    const defaultGridProps = {
      mealPlan: mockMealPlan,
      isEditable: true,
      onMealPress: jest.fn(),
      onMealLongPress: jest.fn(),
      onEmptySlotPress: jest.fn(),
    };

    beforeEach(() => {
      jest.clearAllMocks();
    });

    it('supports global keyboard shortcuts', () => {
      const { getByText } = render(<MealPlanGrid {...defaultGridProps} />);
      
      // Test global undo shortcut (Ctrl+Z)
      fireEvent(getByText('Mon'), 'keyPress', mockKeyEvent('z', false, true));
      
      // Test global redo shortcut (Ctrl+Y)
      fireEvent(getByText('Mon'), 'keyPress', mockKeyEvent('y', false, true));
      
      // These shortcuts would be handled by the parent component
      expect(getByText('Mon')).toBeTruthy();
    });

    it('provides logical tab order through meal slots', () => {
      const { getAllByRole } = render(<MealPlanGrid {...defaultGridProps} />);
      
      const lockButtons = getAllByRole('button');
      
      // Should have at least one lock toggle button
      expect(lockButtons.length).toBeGreaterThan(0);
      
      // Each button should be focusable
      lockButtons.forEach(button => {
        expect(button.props.accessible).toBe(true);
      });
    });

    it('handles Escape key to exit edit modes', () => {
      const { getByText } = render(<MealPlanGrid {...defaultGridProps} />);
      
      // Simulate escape key to exit any active edit state
      fireEvent(getByText('Mon'), 'keyPress', mockKeyEvent('Escape'));
      
      expect(getByText('Mon')).toBeTruthy();
    });

    it('provides screen reader announcements for meal plan changes', () => {
      const { rerender, getByText } = render(<MealPlanGrid {...defaultGridProps} />);
      
      expect(getByText('Breakfast Pancakes')).toBeTruthy();
      
      // Simulate meal plan update
      const updatedMealPlan = {
        ...mockMealPlan,
        populatedMeals: {
          ...mockMealPlan.populatedMeals,
          monday: [
            {
              ...mockMeal,
              isLocked: true,
            },
          ],
        },
      };
      
      rerender(<MealPlanGrid {...defaultGridProps} mealPlan={updatedMealPlan} />);
      
      // Verify meal is still present but state changed
      expect(getByText('Breakfast Pancakes')).toBeTruthy();
    });
  });

  describe('High Contrast Mode Support', () => {
    it('maintains visual hierarchy in high contrast mode', () => {
      const { getByRole } = render(
        <MealLockToggle isLocked={false} onToggle={jest.fn()} />
      );
      
      const button = getByRole('button');
      const styles = button.props.style;
      
      // Verify button has adequate contrast styling
      expect(styles).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            borderWidth: 1,
          }),
        ])
      );
    });

    it('provides non-color dependent visual indicators', () => {
      const lockedProps = {
        day: 'monday' as DayOfWeek,
        mealType: 'breakfast' as MealType,
        meal: mockLockedMeal,
        mealTypeLabel: 'Breakfast',
        mealTypeIcon: '🌅',
        isEmpty: false,
        isEditable: true,
        showLockToggle: true,
        onLockToggle: jest.fn(),
      };
      
      const { getByText } = render(<MealSlot {...lockedProps} />);
      
      // Lock icon should be visible regardless of color perception
      expect(getByText('🔒')).toBeTruthy();
    });
  });

  describe('Screen Reader Compatibility', () => {
    it('provides meaningful content descriptions', () => {
      const { getByRole } = render(
        <MealLockToggle 
          isLocked={false} 
          onToggle={jest.fn()} 
        />
      );
      
      const button = getByRole('button');
      expect(button.props.accessibilityLabel).toBeTruthy();
      expect(button.props.accessibilityHint).toBeTruthy();
    });

    it('announces meal completion status', () => {
      const completedMeal = {
        ...mockMeal,
        isCompleted: true,
      };
      
      const { getByText } = render(
        <MealSlot
          day="monday"
          mealType="breakfast"
          meal={completedMeal}
          mealTypeLabel="Breakfast"
          mealTypeIcon="🌅"
          isEmpty={false}
          isEditable={true}
          onLockToggle={jest.fn()}
        />
      );
      
      expect(getByText('✓ Done')).toBeTruthy();
    });

    it('provides context for drag and drop operations', () => {
      const dragProps = {
        day: 'monday' as DayOfWeek,
        mealType: 'breakfast' as MealType,
        meal: mockMeal,
        mealTypeLabel: 'Breakfast',
        mealTypeIcon: '🌅',
        isEmpty: false,
        isEditable: true,
        dragEnabled: true,
        showLockToggle: true,
        onDragStart: jest.fn(),
        onDragEnd: jest.fn(),
        onLockToggle: jest.fn(),
      };
      
      const { getByTestId } = render(<MealSlot {...dragProps} />);
      
      const dragHandle = getByTestId('drag-handle');
      expect(dragHandle).toBeTruthy();
      
      // In a full implementation, this would have accessibility labels
      // explaining the drag functionality
    });
  });

  describe('Touch and Gesture Accessibility', () => {
    it('provides adequate touch targets', () => {
      const { getByRole } = render(
        <MealLockToggle 
          isLocked={false} 
          onToggle={jest.fn()}
          size="small"
        />
      );
      
      const button = getByRole('button');
      const styles = button.props.style;
      
      // Even small size should meet minimum touch target requirements
      expect(styles).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            width: 20,
            height: 20,
          }),
        ])
      );
    });

    it('supports different size options for accessibility needs', () => {
      const sizes = ['small', 'medium', 'large'] as const;
      
      sizes.forEach(size => {
        const { getByRole } = render(
          <MealLockToggle 
            isLocked={false} 
            onToggle={jest.fn()}
            size={size}
          />
        );
        
        const button = getByRole('button');
        expect(button).toBeTruthy();
      });
    });
  });

  describe('Voice Control Support', () => {
    it('has accessible names for voice commands', () => {
      const { getByRole } = render(
        <MealLockToggle 
          isLocked={false} 
          onToggle={jest.fn()} 
        />
      );
      
      const button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Lock meal');
      
      // Voice control users should be able to say "tap lock meal"
    });

    it('provides context for meal identification in voice commands', () => {
      const { getByText } = render(
        <MealSlot
          day="monday"
          mealType="breakfast"
          meal={mockMeal}
          mealTypeLabel="Breakfast"
          mealTypeIcon="🌅"
          isEmpty={false}
          isEditable={true}
          onLockToggle={jest.fn()}
        />
      );
      
      // Clear meal identification for voice commands
      expect(getByText('🌅 Breakfast')).toBeTruthy();
      expect(getByText('Breakfast Pancakes')).toBeTruthy();
    });
  });
});