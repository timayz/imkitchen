import React from 'react';
import { render, fireEvent, screen, waitFor } from '@testing-library/react-native';
import { RecipeFavoriteButton } from '../RecipeFavoriteButton';

const mockOnToggle = jest.fn();

describe('RecipeFavoriteButton', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly when not favorited', () => {
    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
      />
    );

    expect(screen.getByText('🤍')).toBeTruthy();
  });

  it('renders correctly when favorited', () => {
    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={true}
        onToggle={mockOnToggle}
      />
    );

    expect(screen.getByText('❤️')).toBeTruthy();
  });

  it('calls onToggle when pressed', async () => {
    mockOnToggle.mockResolvedValue(undefined);

    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
      />
    );

    fireEvent.press(screen.getByText('🤍'));

    await waitFor(() => {
      expect(mockOnToggle).toHaveBeenCalledWith('recipe-1');
    });
  });

  it('shows loading state during toggle', async () => {
    let resolveToggle: () => void;
    const togglePromise = new Promise<void>((resolve) => {
      resolveToggle = resolve;
    });
    
    mockOnToggle.mockReturnValue(togglePromise);

    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
      />
    );

    fireEvent.press(screen.getByText('🤍'));

    // Should show loading indicator
    expect(screen.getByText('⏳')).toBeTruthy();

    // Resolve the promise
    resolveToggle!();
    
    await waitFor(() => {
      expect(screen.queryByText('⏳')).toBeNull();
    });
  });

  it('displays label when showLabel is true', () => {
    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
        showLabel={true}
      />
    );

    expect(screen.getByText('Add to Favorites')).toBeTruthy();
  });

  it('displays favorited label when favorited and showLabel is true', () => {
    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={true}
        onToggle={mockOnToggle}
        showLabel={true}
      />
    );

    expect(screen.getByText('Favorited')).toBeTruthy();
  });

  it('does not call onToggle when disabled', () => {
    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
        disabled={true}
      />
    );

    fireEvent.press(screen.getByText('🤍'));

    expect(mockOnToggle).not.toHaveBeenCalled();
  });

  it('applies correct size styling', () => {
    const { rerender } = render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
        size="small"
      />
    );

    expect(screen.getByText('🤍')).toBeTruthy();

    rerender(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
        size="large"
      />
    );

    expect(screen.getByText('🤍')).toBeTruthy();
  });

  it('handles async onToggle errors gracefully', async () => {
    const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {});
    mockOnToggle.mockRejectedValue(new Error('Network error'));

    render(
      <RecipeFavoriteButton
        recipeId="recipe-1"
        isFavorite={false}
        onToggle={mockOnToggle}
      />
    );

    fireEvent.press(screen.getByText('🤍'));

    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith('Failed to toggle favorite:', expect.any(Error));
    });

    consoleSpy.mockRestore();
  });
});