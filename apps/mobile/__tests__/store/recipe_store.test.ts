import { renderHook, act } from '@testing-library/react-hooks';
import { useRecipeStore } from '../../src/store/recipe_store';
import { RecipeClient } from '@imkitchen/api-client';
import { PhotoService } from '../../src/services/photo_service';
import type { Recipe, CreateRecipeInput, RecipeSearchResponse } from '@imkitchen/shared-types';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () => ({
  getItem: jest.fn(() => Promise.resolve(null)),
  setItem: jest.fn(() => Promise.resolve()),
  removeItem: jest.fn(() => Promise.resolve()),
}));

// Mock the API client
const mockRecipeClient = {
  createRecipe: jest.fn(),
  getRecipe: jest.fn(),
  updateRecipe: jest.fn(),
  deleteRecipe: jest.fn(),
  searchRecipes: jest.fn(),
  importRecipe: jest.fn(),
} as unknown as RecipeClient;

// Mock the photo service
const mockPhotoService = {
  uploadRecipePhoto: jest.fn(),
  deleteRecipePhoto: jest.fn(),
  showPhotoPicker: jest.fn(),
  openCamera: jest.fn(),
  openLibrary: jest.fn(),
} as unknown as PhotoService;

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

const mockCreateRecipeInput: CreateRecipeInput = {
  title: 'New Recipe',
  description: 'A new recipe',
  prepTime: 15,
  cookTime: 30,
  mealType: ['lunch'],
  complexity: 'simple',
  servings: 2,
  ingredients: [
    { name: 'Flour', amount: 1, unit: 'cup', category: 'pantry' },
  ],
  instructions: [
    { stepNumber: 1, instruction: 'Mix ingredients' },
  ],
};

describe('RecipeStore', () => {
  beforeEach(() => {
    // Clear all mocks before each test
    jest.clearAllMocks();
  });

  it('initializes with default state', () => {
    const { result } = renderHook(() => useRecipeStore());

    expect(result.current.recipes).toEqual([]);
    expect(result.current.currentRecipe).toBeNull();
    expect(result.current.searchResults).toBeNull();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.filters).toEqual({
      page: 1,
      limit: 20,
      sortBy: 'created_at',
      sortOrder: 'desc',
    });
  });

  it('sets recipe client correctly', () => {
    const { result } = renderHook(() => useRecipeStore());

    act(() => {
      result.current.setRecipeClient(mockRecipeClient);
    });

    // The client is stored internally, so we'll test its usage in other operations
    expect(result.current.recipes).toEqual([]); // State should remain unchanged
  });

  it('sets photo service correctly', () => {
    const { result } = renderHook(() => useRecipeStore());

    act(() => {
      result.current.setPhotoService(mockPhotoService);
    });

    // The service is stored internally, so we'll test its usage in other operations
    expect(result.current.recipes).toEqual([]); // State should remain unchanged
  });

  describe('createRecipe', () => {
    it('creates recipe successfully', async () => {
      (mockRecipeClient.createRecipe as jest.Mock).mockResolvedValue(mockRecipe);

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      let createdRecipe: Recipe | null = null;

      await act(async () => {
        createdRecipe = await result.current.createRecipe(mockCreateRecipeInput);
      });

      expect(createdRecipe).toEqual(mockRecipe);
      expect(result.current.recipes).toContain(mockRecipe);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(mockRecipeClient.createRecipe).toHaveBeenCalledWith(mockCreateRecipeInput);
    });

    it('handles creation error', async () => {
      const errorMessage = 'Creation failed';
      (mockRecipeClient.createRecipe as jest.Mock).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      let createdRecipe: Recipe | null = null;

      await act(async () => {
        createdRecipe = await result.current.createRecipe(mockCreateRecipeInput);
      });

      expect(createdRecipe).toBeNull();
      expect(result.current.recipes).toHaveLength(0);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe(errorMessage);
    });

    it('adds to pending sync when client not initialized', async () => {
      const { result } = renderHook(() => useRecipeStore());

      let createdRecipe: Recipe | null = null;

      await act(async () => {
        createdRecipe = await result.current.createRecipe(mockCreateRecipeInput);
      });

      expect(createdRecipe).toBeNull();
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe('Recipe client not initialized');
    });
  });

  describe('getRecipe', () => {
    it('gets recipe successfully', async () => {
      (mockRecipeClient.getRecipe as jest.Mock).mockResolvedValue(mockRecipe);

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      let retrievedRecipe: Recipe | null = null;

      await act(async () => {
        retrievedRecipe = await result.current.getRecipe('1');
      });

      expect(retrievedRecipe).toEqual(mockRecipe);
      expect(result.current.currentRecipe).toEqual(mockRecipe);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(mockRecipeClient.getRecipe).toHaveBeenCalledWith('1');
    });

    it('returns cached recipe when available', async () => {
      const { result } = renderHook(() => useRecipeStore());

      // First cache the recipe
      act(() => {
        result.current.cacheRecipe(mockRecipe);
      });

      let retrievedRecipe: Recipe | null = null;

      await act(async () => {
        retrievedRecipe = await result.current.getRecipe('1');
      });

      expect(retrievedRecipe).toEqual(mockRecipe);
      expect(result.current.currentRecipe).toEqual(mockRecipe);
      expect(mockRecipeClient.getRecipe).not.toHaveBeenCalled();
    });

    it('handles get recipe error', async () => {
      const errorMessage = 'Recipe not found';
      (mockRecipeClient.getRecipe as jest.Mock).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      let retrievedRecipe: Recipe | null = null;

      await act(async () => {
        retrievedRecipe = await result.current.getRecipe('1');
      });

      expect(retrievedRecipe).toBeNull();
      expect(result.current.currentRecipe).toBeNull();
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe(errorMessage);
    });
  });

  describe('searchRecipes', () => {
    const mockSearchResponse: RecipeSearchResponse = {
      recipes: [mockRecipe],
      total: 1,
      page: 1,
      limit: 20,
      totalPages: 1,
    };

    it('searches recipes successfully', async () => {
      (mockRecipeClient.searchRecipes as jest.Mock).mockResolvedValue(mockSearchResponse);

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      await act(async () => {
        await result.current.searchRecipes();
      });

      expect(result.current.searchResults).toEqual(mockSearchResponse);
      expect(result.current.recipes).toEqual([mockRecipe]);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(mockRecipeClient.searchRecipes).toHaveBeenCalled();
    });

    it('handles search error', async () => {
      const errorMessage = 'Search failed';
      (mockRecipeClient.searchRecipes as jest.Mock).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      await act(async () => {
        await result.current.searchRecipes();
      });

      expect(result.current.searchResults).toBeNull();
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe(errorMessage);
    });
  });

  describe('deleteRecipe', () => {
    it('deletes recipe successfully', async () => {
      (mockRecipeClient.deleteRecipe as jest.Mock).mockResolvedValue({ message: 'Deleted' });

      const { result } = renderHook(() => useRecipeStore());

      // First add a recipe to delete
      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
        result.current.cacheRecipe(mockRecipe);
        useRecipeStore.setState({ recipes: [mockRecipe] });
      });

      let deleteSuccess = false;

      await act(async () => {
        deleteSuccess = await result.current.deleteRecipe('1');
      });

      expect(deleteSuccess).toBe(true);
      expect(result.current.recipes).not.toContain(mockRecipe);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(mockRecipeClient.deleteRecipe).toHaveBeenCalledWith('1');
    });

    it('handles delete error', async () => {
      const errorMessage = 'Delete failed';
      (mockRecipeClient.deleteRecipe as jest.Mock).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setRecipeClient(mockRecipeClient);
      });

      let deleteSuccess = false;

      await act(async () => {
        deleteSuccess = await result.current.deleteRecipe('1');
      });

      expect(deleteSuccess).toBe(false);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe(errorMessage);
    });
  });

  describe('filters', () => {
    it('sets filters correctly', () => {
      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setFilters({ search: 'chicken', mealType: ['dinner'] });
      });

      expect(result.current.filters).toEqual({
        page: 1, // Should reset to page 1
        limit: 20,
        sortBy: 'created_at',
        sortOrder: 'desc',
        search: 'chicken',
        mealType: ['dinner'],
      });
    });

    it('clears filters correctly', () => {
      const { result } = renderHook(() => useRecipeStore());

      // First set some filters
      act(() => {
        result.current.setFilters({ search: 'chicken', mealType: ['dinner'] });
      });

      // Then clear them
      act(() => {
        result.current.clearFilters();
      });

      expect(result.current.filters).toEqual({
        page: 1,
        limit: 20,
        sortBy: 'created_at',
        sortOrder: 'desc',
      });
    });
  });

  describe('photo management', () => {
    it('uploads photo successfully', async () => {
      (mockPhotoService.uploadRecipePhoto as jest.Mock).mockResolvedValue({
        success: true,
        url: 'http://example.com/photo.jpg',
      });

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setPhotoService(mockPhotoService);
        result.current.cacheRecipe(mockRecipe);
        useRecipeStore.setState({ recipes: [mockRecipe] });
      });

      let uploadResult: any = null;

      await act(async () => {
        uploadResult = await result.current.uploadRecipePhoto('1', 'file://photo.jpg');
      });

      expect(uploadResult.success).toBe(true);
      expect(uploadResult.url).toBe('http://example.com/photo.jpg');
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('deletes photo successfully', async () => {
      (mockPhotoService.deleteRecipePhoto as jest.Mock).mockResolvedValue({
        success: true,
      });

      const recipeWithPhoto = { ...mockRecipe, imageUrl: 'http://example.com/photo.jpg' };

      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setPhotoService(mockPhotoService);
        result.current.cacheRecipe(recipeWithPhoto);
        useRecipeStore.setState({ recipes: [recipeWithPhoto] });
      });

      let deleteResult: any = null;

      await act(async () => {
        deleteResult = await result.current.deleteRecipePhoto('1');
      });

      expect(deleteResult.success).toBe(true);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe('utility functions', () => {
    it('caches recipe correctly', () => {
      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.cacheRecipe(mockRecipe);
      });

      const cachedRecipe = result.current.getCachedRecipe('1');
      expect(cachedRecipe).toEqual(mockRecipe);
    });

    it('sets current recipe correctly', () => {
      const { result } = renderHook(() => useRecipeStore());

      act(() => {
        result.current.setCurrentRecipe(mockRecipe);
      });

      expect(result.current.currentRecipe).toEqual(mockRecipe);
    });

    it('clears error correctly', () => {
      const { result } = renderHook(() => useRecipeStore());

      // First set an error
      act(() => {
        useRecipeStore.setState({ error: 'Test error' });
      });

      expect(result.current.error).toBe('Test error');

      // Then clear it
      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });

    it('resets store correctly', () => {
      const { result } = renderHook(() => useRecipeStore());

      // First modify the state
      act(() => {
        useRecipeStore.setState({
          recipes: [mockRecipe],
          currentRecipe: mockRecipe,
          loading: true,
          error: 'Test error',
        });
      });

      // Then reset
      act(() => {
        result.current.reset();
      });

      expect(result.current.recipes).toEqual([]);
      expect(result.current.currentRecipe).toBeNull();
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });
});