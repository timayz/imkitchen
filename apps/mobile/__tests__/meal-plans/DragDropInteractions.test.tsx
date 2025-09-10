import React from 'react';
import { render, fireEvent, act } from '@testing-library/react-native';
import { PanResponder, Animated } from 'react-native';
import { MealCardDraggable } from '../../src/components/meal-plans/MealCardDraggable';
import { MealCalendarDropZone } from '../../src/components/meal-plans/MealCalendarDropZone';
import { MealSlot } from '../../src/components/molecules/MealSlot';
import { MealLockToggle } from '../../src/components/atoms/MealLockToggle';
import type { MealSlotWithRecipe, DayOfWeek, MealType } from '@imkitchen/shared-types';

// Mock React Native components that need special handling for tests
jest.mock('react-native', () => {
  const RN = jest.requireActual('react-native');
  return {
    ...RN,
    PanResponder: {
      create: jest.fn(() => ({
        panHandlers: {
          onStartShouldSetPanResponder: jest.fn(),
          onMoveShouldSetPanResponder: jest.fn(),
          onPanResponderGrant: jest.fn(),
          onPanResponderMove: jest.fn(),
          onPanResponderRelease: jest.fn(),
        },
      })),
    },
    Animated: {
      ...RN.Animated,
      spring: jest.fn(() => ({ start: jest.fn() })),
      timing: jest.fn(() => ({ start: jest.fn() })),
      parallel: jest.fn((animations) => ({ start: jest.fn() })),
      event: jest.fn(),
      ValueXY: jest.fn(() => ({ x: { setValue: jest.fn() }, y: { setValue: jest.fn() } })),
      Value: jest.fn(() => ({ setValue: jest.fn() })),
    },
    Dimensions: {
      get: jest.fn(() => ({ width: 375, height: 812 })),
    },
  };
});

describe('Drag and Drop Interactions', () => {
  const mockMealWithRecipe: MealSlotWithRecipe = {
    day: 'monday',
    mealType: 'breakfast' as MealType,
    servings: 2,
    isCompleted: false,
    isLocked: false,
    recipe: {
      id: 'recipe-1',
      title: 'Pancakes with Berries',
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
      imageUrl: 'https://example.com/pancakes.jpg',
      createdAt: new Date(),
      updatedAt: new Date(),
    },
  };

  const mockLockedMeal: MealSlotWithRecipe = {
    ...mockMealWithRecipe,
    isLocked: true,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('MealCardDraggable', () => {
    const defaultProps = {
      day: 'monday' as DayOfWeek,
      mealType: 'breakfast' as MealType,
      meal: mockMealWithRecipe,
      mealTypeLabel: 'Breakfast',
      mealTypeIcon: '🌅',
      onDragStart: jest.fn(),
      onDragEnd: jest.fn(),
      onPress: jest.fn(),
      onLongPress: jest.fn(),
    };

    it('renders draggable meal card with recipe information', () => {
      const { getByText } = render(<MealCardDraggable {...defaultProps} />);
      
      expect(getByText('🌅 Breakfast')).toBeTruthy();
      expect(getByText('Pancakes with Berries')).toBeTruthy();
      expect(getByText('25m')).toBeTruthy();
      expect(getByText('2 servings')).toBeTruthy();
    });

    it('shows drag handle when not locked', () => {
      const { getByTestId } = render(<MealCardDraggable {...defaultProps} />);
      
      const dragHandle = getByTestId('drag-handle');
      expect(dragHandle).toBeTruthy();
    });

    it('shows lock icon instead of drag handle when locked', () => {
      const lockedProps = { ...defaultProps, isLocked: true, meal: mockLockedMeal };
      const { getByText, queryByTestId } = render(<MealCardDraggable {...lockedProps} />);
      
      expect(getByText('🔒')).toBeTruthy();
      expect(queryByTestId('drag-handle')).toBeNull();
    });

    it('applies locked styling when meal is locked', () => {
      const lockedProps = { ...defaultProps, isLocked: true, meal: mockLockedMeal };
      const { getByTestId } = render(<MealCardDraggable {...lockedProps} />);
      
      const container = getByTestId('meal-card-container');
      expect(container.props.style).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            borderColor: '#FFC107',
            backgroundColor: '#FFFBF0',
          }),
        ])
      );
    });

    it('applies dragging styling when isDragging is true', () => {
      const draggingProps = { ...defaultProps, isDragging: true };
      const { getByTestId } = render(<MealCardDraggable {...draggingProps} />);
      
      const container = getByTestId('meal-card-container');
      expect(container.props.style).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            borderColor: '#2196F3',
            elevation: 8,
          }),
        ])
      );
    });

    it('calls onPress when meal card is pressed', () => {
      const { getByTestId } = render(<MealCardDraggable {...defaultProps} />);
      
      const touchable = getByTestId('meal-card-touchable');
      fireEvent.press(touchable);
      
      expect(defaultProps.onPress).toHaveBeenCalled();
    });

    it('calls onLongPress when meal card is long pressed', () => {
      const { getByTestId } = render(<MealCardDraggable {...defaultProps} />);
      
      const touchable = getByTestId('meal-card-touchable');
      fireEvent(touchable, 'longPress');
      
      expect(defaultProps.onLongPress).toHaveBeenCalled();
    });

    it('does not attach pan handlers when meal is locked', () => {
      const lockedProps = { ...defaultProps, isLocked: true, meal: mockLockedMeal };
      render(<MealCardDraggable {...lockedProps} />);
      
      // PanResponder should not be created for locked meals
      const panResponderCalls = (PanResponder.create as jest.Mock).mock.calls;
      const lockedPanResponder = panResponderCalls.find(call => 
        call[0].onMoveShouldSetPanResponder && 
        !call[0].onMoveShouldSetPanResponder({}, { dx: 10, dy: 10 })
      );
      expect(lockedPanResponder).toBeTruthy();
    });
  });

  describe('MealCalendarDropZone', () => {
    const defaultDropZoneProps = {
      day: 'tuesday' as DayOfWeek,
      mealType: 'lunch' as MealType,
      mealTypeLabel: 'Lunch',
      mealTypeIcon: '☀️',
      isActive: false,
      isValidTarget: true,
      onDrop: jest.fn(),
    };

    it('renders empty drop zone with meal type information', () => {
      const { getByText } = render(<MealCalendarDropZone {...defaultDropZoneProps} />);
      
      expect(getByText('☀️')).toBeTruthy();
      expect(getByText('Lunch')).toBeTruthy();
    });

    it('shows drop hint when active and valid target', () => {
      const activeProps = { ...defaultDropZoneProps, isActive: true, isValidTarget: true };
      const { getByText } = render(<MealCalendarDropZone {...activeProps} />);
      
      expect(getByText('Drop here')).toBeTruthy();
      expect(getByText('↓ Drop Here')).toBeTruthy();
    });

    it('shows invalid drop hint when active but not valid target', () => {
      const invalidProps = { ...defaultDropZoneProps, isActive: true, isValidTarget: false };
      const { getByText } = render(<MealCalendarDropZone {...invalidProps} />);
      
      expect(getByText('Cannot drop here')).toBeTruthy();
      expect(getByText('✕ Invalid')).toBeTruthy();
    });

    it('applies active styling when drop zone is active', () => {
      const activeProps = { ...defaultDropZoneProps, isActive: true };
      const { getByTestId } = render(<MealCalendarDropZone {...activeProps} />);
      
      const container = getByTestId('drop-zone-container');
      expect(container.props.style).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            borderWidth: 2,
          }),
        ])
      );
    });

    it('renders children when provided', () => {
      const { getByText } = render(
        <MealCalendarDropZone {...defaultDropZoneProps}>
          <div>Custom content</div>
        </MealCalendarDropZone>
      );
      
      expect(getByText('Custom content')).toBeTruthy();
    });
  });

  describe('MealLockToggle', () => {
    const defaultToggleProps = {
      isLocked: false,
      onToggle: jest.fn(),
    };

    it('renders unlock icon when not locked', () => {
      const { getByText } = render(<MealLockToggle {...defaultToggleProps} />);
      
      expect(getByText('🔓')).toBeTruthy();
    });

    it('renders lock icon when locked', () => {
      const lockedProps = { ...defaultToggleProps, isLocked: true };
      const { getByText } = render(<MealLockToggle {...lockedProps} />);
      
      expect(getByText('🔒')).toBeTruthy();
    });

    it('calls onToggle when pressed', () => {
      const { getByRole } = render(<MealLockToggle {...defaultToggleProps} />);
      
      const button = getByRole('button');
      fireEvent.press(button);
      
      expect(defaultToggleProps.onToggle).toHaveBeenCalled();
    });

    it('animates on press', () => {
      const { getByRole } = render(<MealLockToggle {...defaultToggleProps} />);
      
      const button = getByRole('button');
      
      act(() => {
        fireEvent.press(button);
      });
      
      // Check that animation was triggered
      expect(Animated.timing).toHaveBeenCalled();
    });

    it('does not call onToggle when disabled', () => {
      const disabledProps = { ...defaultToggleProps, disabled: true };
      const { getByRole } = render(<MealLockToggle {...disabledProps} />);
      
      const button = getByRole('button');
      fireEvent.press(button);
      
      expect(defaultToggleProps.onToggle).not.toHaveBeenCalled();
    });

    it('applies different sizes correctly', () => {
      const sizes = ['small', 'medium', 'large'] as const;
      
      sizes.forEach(size => {
        const { getByRole } = render(
          <MealLockToggle {...defaultToggleProps} size={size} />
        );
        
        const button = getByRole('button');
        const expectedSizes = {
          small: { width: 20, height: 20 },
          medium: { width: 28, height: 28 },
          large: { width: 36, height: 36 },
        };
        
        expect(button.props.style).toMatchObject(
          expect.arrayContaining([
            expect.objectContaining(expectedSizes[size]),
          ])
        );
      });
    });

    it('has proper accessibility labels', () => {
      const { getByRole } = render(<MealLockToggle {...defaultToggleProps} />);
      
      const button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Lock meal');
      expect(button.props.accessibilityHint).toBe(
        'Locks this meal to prevent changes during regeneration'
      );
    });

    it('updates accessibility labels when locked', () => {
      const lockedProps = { ...defaultToggleProps, isLocked: true };
      const { getByRole } = render(<MealLockToggle {...lockedProps} />);
      
      const button = getByRole('button');
      expect(button.props.accessibilityLabel).toBe('Unlock meal');
      expect(button.props.accessibilityHint).toBe(
        'Unlocks this meal so it can be moved or changed during regeneration'
      );
    });
  });

  describe('MealSlot Integration', () => {
    const defaultSlotProps = {
      day: 'monday' as DayOfWeek,
      mealType: 'breakfast' as MealType,
      meal: mockMealWithRecipe,
      mealTypeLabel: 'Breakfast',
      mealTypeIcon: '🌅',
      isEmpty: false,
      isEditable: true,
      dragEnabled: true,
      dropEnabled: true,
      showLockToggle: true,
      onPress: jest.fn(),
      onLongPress: jest.fn(),
      onDragStart: jest.fn(),
      onDragEnd: jest.fn(),
      onLockToggle: jest.fn(),
    };

    it('renders draggable component for meals that can be dragged', () => {
      const { getByTestId } = render(<MealSlot {...defaultSlotProps} />);
      
      expect(getByTestId('meal-card-container')).toBeTruthy();
      expect(getByTestId('drag-handle')).toBeTruthy();
    });

    it('renders drop zone for empty slots', () => {
      const emptySlotProps = { ...defaultSlotProps, meal: undefined, isEmpty: true };
      const { getByTestId } = render(<MealSlot {...emptySlotProps} />);
      
      expect(getByTestId('drop-zone-container')).toBeTruthy();
    });

    it('renders drop zone for locked meals (non-draggable)', () => {
      const lockedSlotProps = { 
        ...defaultSlotProps, 
        meal: mockLockedMeal,
        dragEnabled: false
      };
      const { getByTestId } = render(<MealSlot {...lockedSlotProps} />);
      
      expect(getByTestId('drop-zone-container')).toBeTruthy();
    });

    it('shows lock toggle when enabled', () => {
      const { getByRole } = render(<MealSlot {...defaultSlotProps} />);
      
      const lockToggle = getByRole('button');
      expect(lockToggle).toBeTruthy();
    });

    it('calls onLockToggle when lock toggle is pressed', () => {
      const { getByRole } = render(<MealSlot {...defaultSlotProps} />);
      
      const lockToggle = getByRole('button');
      fireEvent.press(lockToggle);
      
      expect(defaultSlotProps.onLockToggle).toHaveBeenCalledWith(
        'monday',
        'breakfast',
        true
      );
    });

    it('prevents dragging when meal is locked', () => {
      const lockedSlotProps = {
        ...defaultSlotProps,
        meal: { ...mockMealWithRecipe, isLocked: true },
      };
      
      const { queryByTestId } = render(<MealSlot {...lockedSlotProps} />);
      
      // Should not render draggable component
      expect(queryByTestId('drag-handle')).toBeNull();
    });

    it('shows locked visual feedback', () => {
      const lockedSlotProps = {
        ...defaultSlotProps,
        meal: mockLockedMeal,
      };
      
      const { getByTestId } = render(<MealSlot {...lockedSlotProps} />);
      
      const container = getByTestId('drop-zone-container').children[0];
      expect(container.props.style).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            borderColor: '#FFC107',
            backgroundColor: '#FFFBF0',
          }),
        ])
      );
    });
  });

  describe('Drag and Drop Flow Integration', () => {
    it('maintains proper state during drag operations', () => {
      const dragProps = { 
        ...defaultSlotProps, 
        isDragging: true,
        isDragTarget: false 
      };
      
      const { getByTestId } = render(<MealSlot {...dragProps} />);
      
      const container = getByTestId('meal-card-container');
      expect(container.props.style).toMatchObject(
        expect.arrayContaining([
          expect.objectContaining({
            borderColor: '#2196F3',
          }),
        ])
      );
    });

    it('shows drop target visual feedback', () => {
      const dropTargetProps = {
        ...defaultSlotProps,
        meal: undefined,
        isEmpty: true,
        isDragTarget: true,
      };
      
      const { getByTestId } = render(<MealSlot {...dropTargetProps} />);
      
      const dropZone = getByTestId('drop-zone-container');
      expect(dropZone.props.isActive).toBe(true);
      expect(dropZone.props.isValidTarget).toBe(true);
    });

    it('prevents drops on locked slots', () => {
      const lockedDropTargetProps = {
        ...defaultSlotProps,
        meal: mockLockedMeal,
        isDragTarget: true,
      };
      
      const { getByTestId } = render(<MealSlot {...lockedDropTargetProps} />);
      
      const dropZone = getByTestId('drop-zone-container');
      expect(dropZone.props.isValidTarget).toBe(false);
    });
  });

  describe('Performance and Edge Cases', () => {
    it('handles rapid lock/unlock toggles gracefully', () => {
      const { getByRole } = render(<MealSlot {...defaultSlotProps} />);
      
      const lockToggle = getByRole('button');
      
      // Rapidly toggle lock state
      act(() => {
        fireEvent.press(lockToggle);
        fireEvent.press(lockToggle);
        fireEvent.press(lockToggle);
      });
      
      expect(defaultSlotProps.onLockToggle).toHaveBeenCalledTimes(3);
    });

    it('handles missing recipe data gracefully', () => {
      const mealWithoutRecipe = { 
        ...mockMealWithRecipe, 
        recipe: undefined 
      };
      const propsWithoutRecipe = { 
        ...defaultSlotProps, 
        meal: mealWithoutRecipe,
        isEmpty: true 
      };
      
      const { getByTestId } = render(<MealSlot {...propsWithoutRecipe} />);
      
      expect(getByTestId('drop-zone-container')).toBeTruthy();
    });

    it('maintains component stability during prop changes', () => {
      const { rerender } = render(<MealSlot {...defaultSlotProps} />);
      
      // Change props that shouldn't cause crashes
      const updatedProps = {
        ...defaultSlotProps,
        isDragging: true,
        isDragTarget: false,
        meal: { ...mockMealWithRecipe, isLocked: true },
      };
      
      expect(() => {
        rerender(<MealSlot {...updatedProps} />);
      }).not.toThrow();
    });
  });
});