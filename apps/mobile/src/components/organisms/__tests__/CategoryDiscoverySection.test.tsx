import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { CategoryDiscoverySection } from '../CategoryDiscoverySection';
import type { RecipeCategory } from '@imkitchen/shared-types';

describe('CategoryDiscoverySection', () => {
  const mockOnCategoryToggle = jest.fn();
  const mockOnClearCategories = jest.fn();

  beforeEach(() => {
    mockOnCategoryToggle.mockClear();
    mockOnClearCategories.mockClear();
  });

  it('renders correctly with no selected categories', () => {
    const { getByText, queryByText } = render(
      <CategoryDiscoverySection
        selectedCategories={[]}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    expect(getByText('Discover by Category')).toBeTruthy();
    expect(queryByText('Clear All')).toBeFalsy();
    expect(queryByText(/selected/)).toBeFalsy();
  });

  it('renders correctly with selected categories', () => {
    const selectedCategories: RecipeCategory[] = ['vegetarian', 'quick_meals'];
    const { getByText } = render(
      <CategoryDiscoverySection
        selectedCategories={selectedCategories}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    expect(getByText('Discover by Category')).toBeTruthy();
    expect(getByText('Clear All')).toBeTruthy();
    expect(getByText('2 categories selected')).toBeTruthy();
  });

  it('shows singular form for single selected category', () => {
    const selectedCategories: RecipeCategory[] = ['vegetarian'];
    const { getByText } = render(
      <CategoryDiscoverySection
        selectedCategories={selectedCategories}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    expect(getByText('1 category selected')).toBeTruthy();
  });

  it('calls onClearCategories when Clear All is pressed', () => {
    const selectedCategories: RecipeCategory[] = ['vegetarian', 'vegan'];
    const { getByText } = render(
      <CategoryDiscoverySection
        selectedCategories={selectedCategories}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    fireEvent.press(getByText('Clear All'));
    expect(mockOnClearCategories).toHaveBeenCalledTimes(1);
  });

  it('renders all category chips with correct labels', () => {
    const { getByText } = render(
      <CategoryDiscoverySection
        selectedCategories={[]}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    // Check that all expected categories are rendered
    expect(getByText(/🥬.*Vegetarian/)).toBeTruthy();
    expect(getByText(/🌱.*Vegan/)).toBeTruthy();
    expect(getByText(/⚡.*Quick Meals/)).toBeTruthy();
    expect(getByText(/🍲.*Comfort Food/)).toBeTruthy();
    expect(getByText(/💚.*Healthy/)).toBeTruthy();
    expect(getByText(/💰.*Budget Friendly/)).toBeTruthy();
    expect(getByText(/👨‍👩‍👧‍👦.*Family Friendly/)).toBeTruthy();
  });

  it('calls onCategoryToggle with correct category when chip is pressed', () => {
    const { getByLabelText } = render(
      <CategoryDiscoverySection
        selectedCategories={[]}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    // Find and press the vegetarian category chip
    const vegetarianChip = getByLabelText('🥬 Vegetarian category filter');
    fireEvent.press(vegetarianChip);

    expect(mockOnCategoryToggle).toHaveBeenCalledTimes(1);
    expect(mockOnCategoryToggle).toHaveBeenCalledWith('vegetarian');
  });

  it('shows selected state for active categories', () => {
    const selectedCategories: RecipeCategory[] = ['vegetarian'];
    const { getByLabelText } = render(
      <CategoryDiscoverySection
        selectedCategories={selectedCategories}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    const vegetarianChip = getByLabelText('🥬 Vegetarian category filter');
    expect(vegetarianChip.props.accessibilityState).toEqual({ selected: true });
  });

  it('shows unselected state for inactive categories', () => {
    const selectedCategories: RecipeCategory[] = ['vegetarian'];
    const { getByLabelText } = render(
      <CategoryDiscoverySection
        selectedCategories={selectedCategories}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    const veganChip = getByLabelText('🌱 Vegan category filter');
    expect(veganChip.props.accessibilityState).toEqual({ selected: false });
  });

  it('handles multiple selected categories correctly', () => {
    const selectedCategories: RecipeCategory[] = ['vegetarian', 'vegan', 'healthy'];
    const { getByText } = render(
      <CategoryDiscoverySection
        selectedCategories={selectedCategories}
        onCategoryToggle={mockOnCategoryToggle}
        onClearCategories={mockOnClearCategories}
      />
    );

    expect(getByText('3 categories selected')).toBeTruthy();
    expect(getByText('Clear All')).toBeTruthy();
  });
});