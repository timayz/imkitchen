import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { ShoppingItemCard } from '../ShoppingItemCard';
import type { ShoppingItem } from '../../../types/shopping';

// Mock data
const mockShoppingItem: ShoppingItem = {
  id: 'item-1',
  shoppingListId: 'list-1',
  ingredientName: 'Chicken Breast',
  amount: 2,
  unit: 'lbs',
  category: 'protein',
  isCompleted: false,
  notes: undefined,
  recipeSources: ['recipe-1', 'recipe-2'],
  estimatedCost: 12.99,
  createdAt: new Date('2024-01-01'),
  updatedAt: new Date('2024-01-01'),
};

const mockCompletedItem: ShoppingItem = {
  ...mockShoppingItem,
  id: 'item-2',
  isCompleted: true,
  notes: 'Get organic if available',
  completedAt: new Date('2024-01-01'),
};

describe('ShoppingItemCard', () => {
  const mockOnToggleCompleted = jest.fn();
  const mockOnShowRecipeSources = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders shopping item correctly', () => {
      const { getByText } = render(
        <ShoppingItemCard
          item={mockShoppingItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      expect(getByText('Chicken Breast')).toBeTruthy();
      expect(getByText('2 lbs')).toBeTruthy();
      expect(getByText('~$12.99')).toBeTruthy();
    });

    it('renders completed item with different styling', () => {
      const { getByText } = render(
        <ShoppingItemCard
          item={mockCompletedItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      // Completed items should show notes
      expect(getByText('📝 Get organic if available')).toBeTruthy();
    });

    it('shows recipe sources button when sources exist', () => {
      const { getByText } = render(
        <ShoppingItemCard
          item={mockShoppingItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      expect(getByText('🍽️ 2')).toBeTruthy();
    });

    it('hides recipe sources button when no sources', () => {
      const itemWithoutSources = { ...mockShoppingItem, recipeSources: [] };
      const { queryByText } = render(
        <ShoppingItemCard
          item={itemWithoutSources}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      expect(queryByText('🍽️ 0')).toBeFalsy();
    });
  });

  describe('Interactions', () => {
    it('calls onToggleCompleted when item is pressed', async () => {
      const { getByTestId } = render(
        <ShoppingItemCard
          item={mockShoppingItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      const itemCard = getByTestId('shopping-item-card') || getByTestId('shopping-item-container');
      fireEvent.press(itemCard);

      await waitFor(() => {
        expect(mockOnToggleCompleted).toHaveBeenCalledWith(true, undefined);
      });
    });

    it('calls onShowRecipeSources when recipe sources button is pressed', async () => {
      const { getByText } = render(
        <ShoppingItemCard
          item={mockShoppingItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      const recipeButton = getByText('🍽️ 2');
      fireEvent.press(recipeButton);

      await waitFor(() => {
        expect(mockOnShowRecipeSources).toHaveBeenCalled();
      });
    });

    it('does not call actions when disabled', async () => {
      const { getByTestId } = render(
        <ShoppingItemCard
          item={mockShoppingItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
          disabled={true}
        />
      );

      const itemCard = getByTestId('shopping-item-card') || getByTestId('shopping-item-container');
      fireEvent.press(itemCard);

      expect(mockOnToggleCompleted).not.toHaveBeenCalled();
    });
  });

  describe('Amount formatting', () => {
    it('formats whole numbers without decimals', () => {
      const item = { ...mockShoppingItem, amount: 3, unit: 'cups' };
      const { getByText } = render(
        <ShoppingItemCard
          item={item}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      expect(getByText('3 cups')).toBeTruthy();
    });

    it('formats decimal numbers with appropriate precision', () => {
      const item = { ...mockShoppingItem, amount: 1.5, unit: 'cups' };
      const { getByText } = render(
        <ShoppingItemCard
          item={item}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      expect(getByText('1.50 cups')).toBeTruthy();
    });
  });

  describe('Category icons', () => {
    const categories = [
      { category: 'produce', expectedIcon: '🥕' },
      { category: 'dairy', expectedIcon: '🥛' },
      { category: 'protein', expectedIcon: '🍗' },
      { category: 'pantry', expectedIcon: '🏺' },
      { category: 'other', expectedIcon: '📦' },
    ] as const;

    categories.forEach(({ category, expectedIcon }) => {
      it(`shows correct icon for ${category} category`, () => {
        const item = { ...mockShoppingItem, category };
        const { getByText } = render(
          <ShoppingItemCard
            item={item}
            onToggleCompleted={mockOnToggleCompleted}
            onShowRecipeSources={mockOnShowRecipeSources}
          />
        );

        expect(getByText(expectedIcon)).toBeTruthy();
      });
    });
  });

  describe('Notes functionality', () => {
    it('opens notes modal when notes button is pressed', async () => {
      const { getByText } = render(
        <ShoppingItemCard
          item={mockShoppingItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      const notesButton = getByText('📝+');
      fireEvent.press(notesButton);

      await waitFor(() => {
        expect(getByText('Notes for Chicken Breast')).toBeTruthy();
      });
    });

    it('shows existing notes in modal', async () => {
      const { getByText, getByDisplayValue } = render(
        <ShoppingItemCard
          item={mockCompletedItem}
          onToggleCompleted={mockOnToggleCompleted}
          onShowRecipeSources={mockOnShowRecipeSources}
        />
      );

      const notesButton = getByText('📝');
      fireEvent.press(notesButton);

      await waitFor(() => {
        expect(getByDisplayValue('Get organic if available')).toBeTruthy();
      });
    });
  });
});