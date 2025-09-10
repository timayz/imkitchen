import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { CategoryChip } from '../CategoryChip';
import type { RecipeCategory } from '@imkitchen/shared-types';

describe('CategoryChip', () => {
  const mockOnPress = jest.fn();
  const mockCategory: RecipeCategory = 'vegetarian';
  const mockLabel = 'Vegetarian';

  beforeEach(() => {
    mockOnPress.mockClear();
  });

  it('renders correctly with basic props', () => {
    const { getByText, getByRole } = render(
      <CategoryChip
        category={mockCategory}
        label={mockLabel}
        isSelected={false}
        onPress={mockOnPress}
      />
    );

    expect(getByText(mockLabel)).toBeTruthy();
    expect(getByRole('button')).toBeTruthy();
  });

  it('applies selected styles when isSelected is true', () => {
    const { getByRole } = render(
      <CategoryChip
        category={mockCategory}
        label={mockLabel}
        isSelected={true}
        onPress={mockOnPress}
      />
    );

    const button = getByRole('button');
    expect(button.props.accessibilityState).toEqual({ selected: true });
  });

  it('applies unselected styles when isSelected is false', () => {
    const { getByRole } = render(
      <CategoryChip
        category={mockCategory}
        label={mockLabel}
        isSelected={false}
        onPress={mockOnPress}
      />
    );

    const button = getByRole('button');
    expect(button.props.accessibilityState).toEqual({ selected: false });
  });

  it('calls onPress with correct category when pressed', () => {
    const { getByRole } = render(
      <CategoryChip
        category={mockCategory}
        label={mockLabel}
        isSelected={false}
        onPress={mockOnPress}
      />
    );

    fireEvent.press(getByRole('button'));
    expect(mockOnPress).toHaveBeenCalledTimes(1);
    expect(mockOnPress).toHaveBeenCalledWith(mockCategory);
  });

  it('has correct accessibility properties', () => {
    const { getByRole } = render(
      <CategoryChip
        category={mockCategory}
        label={mockLabel}
        isSelected={false}
        onPress={mockOnPress}
      />
    );

    const button = getByRole('button');
    expect(button.props.accessibilityLabel).toBe('Vegetarian category filter');
    expect(button.props.accessibilityRole).toBe('button');
  });

  it('applies custom style when provided', () => {
    const customStyle = { marginTop: 10 };
    const { getByRole } = render(
      <CategoryChip
        category={mockCategory}
        label={mockLabel}
        isSelected={false}
        onPress={mockOnPress}
        style={customStyle}
      />
    );

    const button = getByRole('button');
    expect(button.props.style).toEqual(expect.arrayContaining([
      expect.objectContaining(customStyle)
    ]));
  });

  it('handles different category types', () => {
    const categories: RecipeCategory[] = [
      'vegetarian',
      'vegan',
      'quick_meals',
      'comfort_food',
      'healthy',
      'budget_friendly',
      'family_friendly'
    ];

    categories.forEach((category) => {
      const { getByRole } = render(
        <CategoryChip
          category={category}
          label={category}
          isSelected={false}
          onPress={mockOnPress}
        />
      );

      fireEvent.press(getByRole('button'));
      expect(mockOnPress).toHaveBeenLastCalledWith(category);
    });

    expect(mockOnPress).toHaveBeenCalledTimes(categories.length);
  });
});