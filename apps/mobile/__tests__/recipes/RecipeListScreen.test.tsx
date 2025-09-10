import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { RecipeListScreen } from '../../src/screens/recipes/RecipeListScreen';
import { useRecipeStore } from '../../src/store/recipe_store';
import type { Recipe } from '@imkitchen/shared-types';

// Mock the store
jest.mock('../../src/store/recipe_store', () => ({
  useRecipeStore: jest.fn(),
}));

// Mock navigation
const mockNavigate = jest.fn();
jest.mock('@react-navigation/native', () => ({
  ...jest.requireActual('@react-navigation/native'),
  useNavigation: () => ({
    navigate: mockNavigate,
  }),
  useFocusEffect: (callback: () => void) => {
    React.useEffect(callback, []);
  },
}));

const Stack = createNativeStackNavigator();

const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <NavigationContainer>
    <Stack.Navigator>
      <Stack.Screen name="RecipeList" component={() => <>{children}</>} />
    </Stack.Navigator>
  </NavigationContainer>
);

const mockRecipe: Recipe = {
  id: '1',
  title: 'Test Recipe',
  description: 'A delicious test recipe',
  prepTime: 30,
  cookTime: 45,
  totalTime: 75,
  mealType: ['dinner'],
  complexity: 'simple',
  servings: 4,
  ingredients: [],
  instructions: [],
  averageRating: 4.5,
  totalRatings: 10,
  dietaryLabels: ['vegetarian'],
  createdAt: new Date(),
  updatedAt: new Date(),
};

describe('RecipeListScreen', () => {
  const mockUseRecipeStore = useRecipeStore as jest.MockedFunction<typeof useRecipeStore>;

  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it('renders loading state correctly', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [],
      searchResults: null,
      loading: true,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    // Should show header
    expect(getByText('My Recipes')).toBeTruthy();
  });

  it('renders empty state correctly', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [],
      searchResults: null,
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    expect(getByText('No recipes yet. Add your first recipe!')).toBeTruthy();
    expect(getByText('Add Recipe')).toBeTruthy();
  });

  it('renders recipe list correctly', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [mockRecipe],
      searchResults: { recipes: [mockRecipe], total: 1, page: 1, limit: 20, totalPages: 1 },
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    expect(getByText('Test Recipe')).toBeTruthy();
    expect(getByText('A delicious test recipe')).toBeTruthy();
  });

  it('handles search input correctly', async () => {
    const mockSetFilters = jest.fn();
    const mockSearchRecipes = jest.fn();

    mockUseRecipeStore.mockReturnValue({
      recipes: [],
      searchResults: null,
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: mockSearchRecipes,
      setFilters: mockSetFilters,
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByPlaceholderText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    const searchInput = getByPlaceholderText('Search recipes...');
    fireEvent.changeText(searchInput, 'chicken');

    await waitFor(() => {
      expect(mockSetFilters).toHaveBeenCalledWith({ search: 'chicken' });
      expect(mockSearchRecipes).toHaveBeenCalled();
    });
  });

  it('navigates to add recipe screen when add button is pressed', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [],
      searchResults: null,
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getAllByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    // Get the + button (header add button)
    const addButtons = getAllByText('+');
    fireEvent.press(addButtons[0]);

    expect(mockNavigate).toHaveBeenCalledWith('AddRecipe');
  });

  it('navigates to import recipe screen when import button is pressed', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [],
      searchResults: null,
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    const importButton = getByText('Import');
    fireEvent.press(importButton);

    expect(mockNavigate).toHaveBeenCalledWith('ImportRecipe');
  });

  it('clears filters when clear button is pressed', () => {
    const mockClearFilters = jest.fn();
    const mockSearchRecipes = jest.fn();

    mockUseRecipeStore.mockReturnValue({
      recipes: [],
      searchResults: null,
      loading: false,
      error: null,
      filters: { 
        search: 'chicken',
        page: 1, 
        limit: 20, 
        sortBy: 'created_at', 
        sortOrder: 'desc' 
      },
      searchRecipes: mockSearchRecipes,
      setFilters: jest.fn(),
      clearFilters: mockClearFilters,
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    const clearButton = getByText('Clear');
    fireEvent.press(clearButton);

    expect(mockClearFilters).toHaveBeenCalled();
    expect(mockSearchRecipes).toHaveBeenCalled();
  });

  it('shows results count when search results are available', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [mockRecipe],
      searchResults: { recipes: [mockRecipe], total: 1, page: 1, limit: 20, totalPages: 1 },
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    expect(getByText('1 recipe found')).toBeTruthy();
  });

  it('handles recipe press correctly', () => {
    mockUseRecipeStore.mockReturnValue({
      recipes: [mockRecipe],
      searchResults: { recipes: [mockRecipe], total: 1, page: 1, limit: 20, totalPages: 1 },
      loading: false,
      error: null,
      filters: { page: 1, limit: 20, sortBy: 'created_at', sortOrder: 'desc' },
      searchRecipes: jest.fn(),
      setFilters: jest.fn(),
      clearFilters: jest.fn(),
      clearError: jest.fn(),
    } as any);

    const { getByText } = render(
      <TestWrapper>
        <RecipeListScreen />
      </TestWrapper>
    );

    // Press on the recipe title
    const recipeTitle = getByText('Test Recipe');
    fireEvent.press(recipeTitle);

    expect(mockNavigate).toHaveBeenCalledWith('RecipeDetail', { recipeId: '1' });
  });
});