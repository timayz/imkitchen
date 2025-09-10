import React from 'react';
import { render, fireEvent, screen, waitFor } from '@testing-library/react-native';
import { FavoritesManager } from '../FavoritesManager';
import { useFavoritesStore } from '../../../store/favorites_store';

// Mock the favorites store
jest.mock('../../../store/favorites_store');
const mockUseFavoritesStore = useFavoritesStore as jest.MockedFunction<typeof useFavoritesStore>;

// Mock Alert
jest.mock('react-native', () => ({
  ...jest.requireActual('react-native'),
  Alert: {
    alert: jest.fn(),
  },
}));

const mockFavorites = [
  {
    recipeId: 'recipe-1',
    recipe: {
      id: 'recipe-1',
      name: 'Spaghetti Carbonara',
      description: 'Classic Italian pasta dish',
      prepTime: 30,
      complexity: 'moderate' as const,
      tags: ['pasta', 'italian', 'quick'],
    },
    favoriteAt: '2024-01-01T00:00:00Z',
  },
  {
    recipeId: 'recipe-2',
    recipe: {
      id: 'recipe-2',
      name: 'Chocolate Chip Cookies',
      description: 'Soft and chewy cookies',
      prepTime: 45,
      complexity: 'simple' as const,
      tags: ['dessert', 'baking'],
    },
    favoriteAt: '2024-01-02T00:00:00Z',
  },
];

describe('FavoritesManager', () => {
  const mockLoadFavorites = jest.fn();
  const mockToggleFavorite = jest.fn();
  const mockClearError = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    
    mockUseFavoritesStore.mockReturnValue({
      favorites: mockFavorites,
      isLoading: false,
      error: null,
      loadFavorites: mockLoadFavorites,
      toggleFavorite: mockToggleFavorite,
      clearError: mockClearError,
      // Add other required properties with mock implementations
      addFavorite: jest.fn(),
      removeFavorite: jest.fn(),
      isFavorite: jest.fn(),
      getFavoriteMultiplier: jest.fn(),
      exportFavorites: jest.fn(),
      importFavorites: jest.fn(),
      refreshFavorites: jest.fn(),
      lastUpdated: null,
    });
  });

  it('renders loading state correctly', () => {
    mockUseFavoritesStore.mockReturnValue({
      favorites: [],
      isLoading: true,
      error: null,
      loadFavorites: mockLoadFavorites,
      toggleFavorite: mockToggleFavorite,
      clearError: mockClearError,
      addFavorite: jest.fn(),
      removeFavorite: jest.fn(),
      isFavorite: jest.fn(),
      getFavoriteMultiplier: jest.fn(),
      exportFavorites: jest.fn(),
      importFavorites: jest.fn(),
      refreshFavorites: jest.fn(),
      lastUpdated: null,
    });

    render(<FavoritesManager />);

    expect(screen.getByText('Loading your favorites...')).toBeTruthy();
  });

  it('renders favorites list correctly', () => {
    render(<FavoritesManager />);

    expect(screen.getByText('Your Favorite Recipes')).toBeTruthy();
    expect(screen.getByText('2 recipes marked as favorite')).toBeTruthy();
    expect(screen.getByText('Spaghetti Carbonara')).toBeTruthy();
    expect(screen.getByText('Chocolate Chip Cookies')).toBeTruthy();
  });

  it('loads favorites on mount', () => {
    render(<FavoritesManager />);

    expect(mockLoadFavorites).toHaveBeenCalled();
  });

  it('renders empty state when no favorites', () => {
    mockUseFavoritesStore.mockReturnValue({
      favorites: [],
      isLoading: false,
      error: null,
      loadFavorites: mockLoadFavorites,
      toggleFavorite: mockToggleFavorite,
      clearError: mockClearError,
      addFavorite: jest.fn(),
      removeFavorite: jest.fn(),
      isFavorite: jest.fn(),
      getFavoriteMultiplier: jest.fn(),
      exportFavorites: jest.fn(),
      importFavorites: jest.fn(),
      refreshFavorites: jest.fn(),
      lastUpdated: null,
    });

    render(<FavoritesManager />);

    expect(screen.getByText('No favorites yet')).toBeTruthy();
    expect(screen.getByText('Start exploring recipes and tap the heart icon to add them here')).toBeTruthy();
  });

  it('handles search functionality', () => {
    render(<FavoritesManager />);

    const searchInput = screen.getByPlaceholderText('Search your favorites...');
    fireEvent.changeText(searchInput, 'spaghetti');

    // Should still show the matching recipe
    expect(screen.getByText('Spaghetti Carbonara')).toBeTruthy();
  });

  it('filters favorites based on search query', async () => {
    render(<FavoritesManager />);

    const searchInput = screen.getByPlaceholderText('Search your favorites...');
    fireEvent.changeText(searchInput, 'cookies');

    // Should show filtered results
    await waitFor(() => {
      expect(screen.getByText('Chocolate Chip Cookies')).toBeTruthy();
    });
  });

  it('shows no matching favorites message for non-matching search', async () => {
    render(<FavoritesManager />);

    const searchInput = screen.getByPlaceholderText('Search your favorites...');
    fireEvent.changeText(searchInput, 'xyz123');

    await waitFor(() => {
      expect(screen.getByText('No matching favorites')).toBeTruthy();
      expect(screen.getByText('Try adjusting your search terms')).toBeTruthy();
    });
  });

  it('handles toggle favorite correctly', async () => {
    mockToggleFavorite.mockResolvedValue(undefined);

    render(<FavoritesManager />);

    // Find favorite buttons and click one
    const favoriteButtons = screen.getAllByText('❤️');
    fireEvent.press(favoriteButtons[0]);

    await waitFor(() => {
      expect(mockToggleFavorite).toHaveBeenCalledWith('recipe-1');
    });
  });

  it('handles toggle favorite error', async () => {
    const mockAlert = require('react-native').Alert.alert;
    mockToggleFavorite.mockRejectedValue(new Error('Network error'));

    render(<FavoritesManager />);

    const favoriteButtons = screen.getAllByText('❤️');
    fireEvent.press(favoriteButtons[0]);

    await waitFor(() => {
      expect(mockAlert).toHaveBeenCalledWith(
        'Error',
        'Failed to update favorite status. Please try again.',
        [{ text: 'OK' }]
      );
    });
  });

  it('displays recipe statistics correctly', () => {
    render(<FavoritesManager />);

    expect(screen.getByText('⏱️ 30min')).toBeTruthy();
    expect(screen.getByText('🍳 moderate')).toBeTruthy();
    expect(screen.getByText('⏱️ 45min')).toBeTruthy();
    expect(screen.getByText('🍳 simple')).toBeTruthy();
  });

  it('shows import/export section when showImportExport is true', () => {
    render(<FavoritesManager showImportExport={true} />);

    expect(screen.getByText('Data Management')).toBeTruthy();
  });

  it('hides import/export section when showImportExport is false', () => {
    render(<FavoritesManager showImportExport={false} />);

    expect(screen.queryByText('Data Management')).toBeNull();
  });
});