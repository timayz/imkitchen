import React from 'react';
import { render, fireEvent, waitFor, act } from '@testing-library/react-native';
import { QuickSwapModal } from '../../src/components/meal-plans/QuickSwapModal';
import type { MealSlotWithRecipe, Recipe } from '@imkitchen/shared-types';
import type { SwapSuggestion, SwapFilters } from '../../src/components/meal-plans/QuickSwapModal';

// Mock React Native components
jest.mock('react-native', () => {
  const RN = jest.requireActual('react-native');
  return {
    ...RN,
    Modal: ({ visible, children, onRequestClose }: any) => 
      visible ? <div data-testid="modal" onKeyPress={onRequestClose}>{children}</div> : null,
    SafeAreaView: ({ children }: any) => <div data-testid="safe-area">{children}</div>,
    ActivityIndicator: ({ testID }: any) => <div data-testid={testID || "activity-indicator"}>Loading...</div>,
  };
});

describe('QuickSwapModal', () => {
  const mockMeal: MealSlotWithRecipe = {
    day: 'monday',
    mealType: 'breakfast',
    servings: 2,
    isCompleted: false,
    isLocked: false,
    recipe: {
      id: 'original-recipe',
      title: 'Original Pancakes',
      prepTime: 15,
      cookTime: 10,
      totalTime: 25,
      complexity: 'simple',
      mealType: ['breakfast'],
      servings: 4,
      ingredients: [
        { name: 'Flour', amount: '2 cups', unit: 'cups' },
        { name: 'Eggs', amount: '2', unit: 'pieces' },
      ],
      instructions: ['Mix ingredients', 'Cook'],
      dietaryLabels: ['vegetarian'],
      averageRating: 4.2,
      totalRatings: 50,
      createdAt: new Date(),
      updatedAt: new Date(),
    },
  };

  const mockSuggestions: SwapSuggestion[] = [
    {
      recipe: {
        id: 'suggestion-1',
        title: 'Blueberry Pancakes',
        prepTime: 18,
        cookTime: 12,
        totalTime: 30,
        complexity: 'simple',
        mealType: ['breakfast'],
        servings: 4,
        ingredients: [
          { name: 'Flour', amount: '2 cups', unit: 'cups' },
          { name: 'Blueberries', amount: '1 cup', unit: 'cups' },
        ],
        instructions: ['Mix ingredients', 'Add blueberries', 'Cook'],
        dietaryLabels: ['vegetarian'],
        averageRating: 4.5,
        totalRatings: 35,
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      compatibilityScore: 0.85,
      reasons: ['Same breakfast type', 'Similar complexity', 'Shared ingredients'],
      timeDifference: 5,
      complexityMatch: true,
      cuisineMatch: true,
      shoppingListImpact: {
        itemsAdded: 1,
        itemsRemoved: 0,
        estimatedCostChange: 2.50,
      },
    },
    {
      recipe: {
        id: 'suggestion-2',
        title: 'French Toast',
        prepTime: 10,
        cookTime: 8,
        totalTime: 18,
        complexity: 'simple',
        mealType: ['breakfast'],
        servings: 4,
        ingredients: [
          { name: 'Bread', amount: '8 slices', unit: 'slices' },
          { name: 'Eggs', amount: '3', unit: 'pieces' },
        ],
        instructions: ['Beat eggs', 'Dip bread', 'Cook'],
        dietaryLabels: ['vegetarian'],
        averageRating: 4.1,
        totalRatings: 42,
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      compatibilityScore: 0.72,
      reasons: ['Great for breakfast', 'Faster preparation', 'Uses eggs'],
      timeDifference: -7,
      complexityMatch: true,
      cuisineMatch: false,
      shoppingListImpact: {
        itemsAdded: 2,
        itemsRemoved: 1,
        estimatedCostChange: 1.25,
      },
    },
  ];

  const defaultProps = {
    visible: true,
    currentMeal: mockMeal,
    day: 'monday' as const,
    mealType: 'breakfast' as const,
    onClose: jest.fn(),
    onSwapConfirmed: jest.fn(),
    onGetSuggestions: jest.fn().mockResolvedValue(mockSuggestions),
    onPreviewShoppingListChanges: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Modal Display', () => {
    it('renders modal when visible', () => {
      const { getByTestId } = render(<QuickSwapModal {...defaultProps} />);
      
      expect(getByTestId('modal')).toBeTruthy();
      expect(getByTestId('safe-area')).toBeTruthy();
    });

    it('does not render when not visible', () => {
      const { queryByTestId } = render(
        <QuickSwapModal {...defaultProps} visible={false} />
      );
      
      expect(queryByTestId('modal')).toBeNull();
    });

    it('displays current meal information', () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      expect(getByText('Quick Swap')).toBeTruthy();
      expect(getByText(/Replace Original Pancakes/)).toBeTruthy();
      expect(getByText(/monday breakfast/)).toBeTruthy();
    });

    it('shows loading state while fetching suggestions', () => {
      const { getByText, getByTestId } = render(
        <QuickSwapModal 
          {...defaultProps} 
          onGetSuggestions={jest.fn().mockImplementation(() => new Promise(() => {}))} 
        />
      );

      expect(getByTestId('activity-indicator')).toBeTruthy();
      expect(getByText('Finding similar recipes...')).toBeTruthy();
    });
  });

  describe('Header Controls', () => {
    it('calls onClose when close button is pressed', () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      fireEvent.press(getByText('✕'));
      
      expect(defaultProps.onClose).toHaveBeenCalled();
    });

    it('shows filters button', () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      expect(getByText('Filters')).toBeTruthy();
    });

    it('toggles filters when filters button is pressed', () => {
      const { getByText, queryByText } = render(<QuickSwapModal {...defaultProps} />);
      
      // Filters should not be visible initially
      expect(queryByText('Max prep time:')).toBeNull();
      
      // Press filters button
      fireEvent.press(getByText('Filters'));
      
      // Filters should now be visible
      expect(getByText('Max prep time:')).toBeTruthy();
    });
  });

  describe('Search Functionality', () => {
    it('renders search input', () => {
      const { getByPlaceholderText } = render(<QuickSwapModal {...defaultProps} />);
      
      expect(getByPlaceholderText('Search recipes...')).toBeTruthy();
    });

    it('filters suggestions based on search query', async () => {
      const { getByPlaceholderText, getByText, queryByText } = render(
        <QuickSwapModal {...defaultProps} />
      );

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
        expect(getByText('French Toast')).toBeTruthy();
      });

      // Search for "blueberry"
      const searchInput = getByPlaceholderText('Search recipes...');
      fireEvent.changeText(searchInput, 'blueberry');

      // Should show only blueberry pancakes
      expect(getByText('Blueberry Pancakes')).toBeTruthy();
      expect(queryByText('French Toast')).toBeNull();
    });

    it('shows empty state when no matches found', async () => {
      const { getByPlaceholderText, getByText } = render(
        <QuickSwapModal {...defaultProps} />
      );

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      const searchInput = getByPlaceholderText('Search recipes...');
      fireEvent.changeText(searchInput, 'nonexistent recipe');

      expect(getByText('No matching recipes found')).toBeTruthy();
      expect(getByText('Try adjusting your filters or search terms')).toBeTruthy();
    });
  });

  describe('Filters', () => {
    it('shows time filter buttons', () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      // Open filters
      fireEvent.press(getByText('Filters'));
      
      expect(getByText('15m')).toBeTruthy();
      expect(getByText('30m')).toBeTruthy();
      expect(getByText('1h')).toBeTruthy();
      expect(getByText('2h')).toBeTruthy();
    });

    it('shows complexity filter buttons', () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      fireEvent.press(getByText('Filters'));
      
      expect(getByText('simple')).toBeTruthy();
      expect(getByText('moderate')).toBeTruthy();
      expect(getByText('complex')).toBeTruthy();
    });

    it('applies filters and calls onGetSuggestions', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      fireEvent.press(getByText('Filters'));
      fireEvent.press(getByText('30m'));
      fireEvent.press(getByText('simple'));
      fireEvent.press(getByText('Apply Filters'));

      await waitFor(() => {
        expect(defaultProps.onGetSuggestions).toHaveBeenCalledWith(
          expect.objectContaining({
            maxPrepTime: 30,
            complexity: 'simple',
          })
        );
      });
    });
  });

  describe('Suggestion Cards', () => {
    it('displays suggestion information correctly', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        // Recipe title
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
        
        // Compatibility score
        expect(getByText('85%')).toBeTruthy();
        
        // Time and complexity
        expect(getByText('30m')).toBeTruthy();
        expect(getByText('simple')).toBeTruthy();
        
        // Shopping list impact
        expect(getByText('+1 items')).toBeTruthy();
        expect(getByText('-0 items')).toBeTruthy();
        expect(getByText('+$2.50')).toBeTruthy();
      });
    });

    it('shows compatibility reasons', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        expect(getByText('Why this match:')).toBeTruthy();
        expect(getByText('• Same breakfast type')).toBeTruthy();
        expect(getByText('• Similar complexity')).toBeTruthy();
      });
    });

    it('handles suggestion selection', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      // Select the first suggestion
      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        expect(getByText('✓ Selected')).toBeTruthy();
      });
    });

    it('calls preview shopping list changes when suggestion is selected', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        expect(defaultProps.onPreviewShoppingListChanges).toHaveBeenCalledWith('suggestion-1');
      });
    });
  });

  describe('Swap Confirmation', () => {
    it('shows swap button when suggestion is selected', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        expect(getByText('Swap to Blueberry Pancakes')).toBeTruthy();
      });
    });

    it('calls onSwapConfirmed when swap button is pressed', async () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        expect(getByText('Swap to Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Swap to Blueberry Pancakes'));

      expect(defaultProps.onSwapConfirmed).toHaveBeenCalledWith('suggestion-1');
    });

    it('shows loading state during swap', async () => {
      const mockSwapConfirmed = jest.fn().mockImplementation(
        () => new Promise(() => {}) // Never resolves
      );

      const { getByText, getByTestId } = render(
        <QuickSwapModal {...defaultProps} onSwapConfirmed={mockSwapConfirmed} />
      );

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        expect(getByText('Swap to Blueberry Pancakes')).toBeTruthy();
      });

      act(() => {
        fireEvent.press(getByText('Swap to Blueberry Pancakes'));
      });

      expect(getByTestId('activity-indicator')).toBeTruthy();
    });
  });

  describe('Error Handling', () => {
    it('handles missing recipe gracefully', () => {
      const mealWithoutRecipe = { ...mockMeal, recipe: undefined };
      
      const { queryByTestId } = render(
        <QuickSwapModal {...defaultProps} currentMeal={mealWithoutRecipe} />
      );
      
      expect(queryByTestId('modal')).toBeNull();
    });

    it('shows empty state when suggestions fail to load', async () => {
      const failingGetSuggestions = jest.fn().mockRejectedValue(new Error('Network error'));
      
      const { getByText } = render(
        <QuickSwapModal {...defaultProps} onGetSuggestions={failingGetSuggestions} />
      );

      await waitFor(() => {
        expect(getByText('No matching recipes found')).toBeTruthy();
      });
    });

    it('handles swap confirmation errors gracefully', async () => {
      const failingSwapConfirmed = jest.fn().mockRejectedValue(new Error('Swap failed'));
      
      const { getByText } = render(
        <QuickSwapModal {...defaultProps} onSwapConfirmed={failingSwapConfirmed} />
      );

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        expect(getByText('Swap to Blueberry Pancakes')).toBeTruthy();
      });

      await act(async () => {
        fireEvent.press(getByText('Swap to Blueberry Pancakes'));
      });

      // Error should be thrown and handled by the component
      expect(failingSwapConfirmed).toHaveBeenCalled();
    });
  });

  describe('Accessibility', () => {
    it('has proper accessibility labels on close button', () => {
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      const closeButton = getByText('✕');
      expect(closeButton.props.accessibilityRole).toBe('button');
    });

    it('has accessible search input', () => {
      const { getByPlaceholderText } = render(<QuickSwapModal {...defaultProps} />);
      
      const searchInput = getByPlaceholderText('Search recipes...');
      expect(searchInput.props.accessibilityLabel).toBe('Search recipes');
      expect(searchInput.props.accessibilityHint).toBe('Search through suggested recipe replacements');
    });

    it('has accessible suggestion cards', async () => {
      const { getByLabelText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        const suggestionCard = getByLabelText('Replace with Blueberry Pancakes');
        expect(suggestionCard).toBeTruthy();
        expect(suggestionCard.props.accessibilityHint).toBe('85% match, 30m cooking time');
      });
    });

    it('has accessible swap confirmation button', async () => {
      const { getByText, getByLabelText } = render(<QuickSwapModal {...defaultProps} />);

      await waitFor(() => {
        expect(getByText('Blueberry Pancakes')).toBeTruthy();
      });

      fireEvent.press(getByText('Blueberry Pancakes'));

      await waitFor(() => {
        const swapButton = getByLabelText('Confirm swap to Blueberry Pancakes');
        expect(swapButton).toBeTruthy();
        expect(swapButton.props.accessibilityRole).toBe('button');
      });
    });
  });

  describe('Performance', () => {
    it('debounces suggestion loading', async () => {
      jest.useFakeTimers();
      
      const { getByText } = render(<QuickSwapModal {...defaultProps} />);
      
      // Open filters
      fireEvent.press(getByText('Filters'));
      
      // Rapidly change filters
      fireEvent.press(getByText('30m'));
      fireEvent.press(getByText('1h'));
      fireEvent.press(getByText('15m'));
      
      // Should not call suggestions multiple times immediately
      expect(defaultProps.onGetSuggestions).toHaveBeenCalledTimes(1); // Initial load
      
      jest.useRealTimers();
    });

    it('handles large suggestion lists efficiently', async () => {
      const largeSuggestionList = Array(50).fill(null).map((_, i) => ({
        ...mockSuggestions[0],
        recipe: {
          ...mockSuggestions[0].recipe,
          id: `suggestion-${i}`,
          title: `Recipe ${i}`,
        },
      }));

      const { getAllByText } = render(
        <QuickSwapModal 
          {...defaultProps} 
          onGetSuggestions={jest.fn().mockResolvedValue(largeSuggestionList)} 
        />
      );

      // Should render all suggestions without performance issues
      await waitFor(() => {
        const recipeTitles = getAllByText(/Recipe \d+/);
        expect(recipeTitles.length).toBe(50);
      });
    });
  });
});