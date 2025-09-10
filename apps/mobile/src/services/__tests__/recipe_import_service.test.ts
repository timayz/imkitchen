import { RecipeImportService } from '../recipe_import_service';
import type { RecipeImportRequest } from '@imkitchen/shared-types';

// Mock fetch
global.fetch = jest.fn();
const mockFetch = fetch as jest.MockedFunction<typeof fetch>;

describe('RecipeImportService', () => {
  let service: RecipeImportService;
  const baseURL = 'https://api.example.com';
  const authToken = 'test-token';

  beforeEach(() => {
    service = new RecipeImportService(baseURL);
    service.setAuthToken(authToken);
    mockFetch.mockClear();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('importCommunityRecipe', () => {
    it('successfully imports a community recipe', async () => {
      const mockRequest: RecipeImportRequest = {
        communityRecipeId: 'test-recipe-id',
        preserveAttribution: true,
        customizations: {
          title: 'My Custom Recipe',
          notes: 'My personal notes',
          servingAdjustment: 4,
        },
      };

      const mockResponse = {
        success: true,
        personalRecipeId: 'new-recipe-id',
        message: 'Recipe successfully imported',
        attribution: {
          originalContributor: 'Original Chef',
          importDate: '2024-09-10T10:00:00Z',
          communityMetrics: {
            totalImports: 42,
            averageRating: 4.5,
          },
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await service.importCommunityRecipe(mockRequest);

      expect(mockFetch).toHaveBeenCalledWith(
        `${baseURL}/api/v1/recipes/import`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${authToken}`,
          },
          body: JSON.stringify(mockRequest),
        }
      );
      expect(result).toEqual(mockResponse);
    });

    it('handles import errors properly', async () => {
      const mockRequest: RecipeImportRequest = {
        communityRecipeId: 'invalid-recipe-id',
        preserveAttribution: true,
      };

      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        json: async () => ({ error: 'Community recipe not found' }),
      } as Response);

      await expect(service.importCommunityRecipe(mockRequest)).rejects.toThrow(
        'Community recipe not found'
      );
    });

    it('handles network errors', async () => {
      const mockRequest: RecipeImportRequest = {
        communityRecipeId: 'test-recipe-id',
        preserveAttribution: true,
      };

      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: async () => {
          throw new Error('Invalid JSON');
        },
      } as Response);

      await expect(service.importCommunityRecipe(mockRequest)).rejects.toThrow(
        'HTTP 500: Internal Server Error'
      );
    });
  });

  describe('checkImportConflict', () => {
    it('returns no conflict when recipe not imported', async () => {
      const mockResponse = { hasConflict: false };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await service.checkImportConflict('test-recipe-id');

      expect(mockFetch).toHaveBeenCalledWith(
        `${baseURL}/api/v1/recipes/import/check/test-recipe-id`,
        {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${authToken}`,
          },
        }
      );
      expect(result).toEqual(mockResponse);
    });

    it('returns conflict details when recipe already imported', async () => {
      const mockResponse = {
        hasConflict: true,
        conflict: {
          existingRecipeId: 'existing-id',
          existingRecipeTitle: 'Existing Recipe',
          importedAt: '2024-09-09T10:00:00Z',
          conflictType: 'duplicate_import',
          resolution: {
            options: ['rename', 'merge', 'replace', 'cancel'],
            recommended: 'rename',
          },
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await service.checkImportConflict('test-recipe-id');

      expect(result).toEqual(mockResponse);
    });
  });

  describe('getImportHistory', () => {
    it('retrieves import history with pagination', async () => {
      const mockResponse = {
        imports: [
          {
            id: 'import-1',
            userId: 'user-1',
            personalRecipeId: 'recipe-1',
            communityRecipeId: 'community-1',
            importedAt: '2024-09-10T10:00:00Z',
            preserveAttribution: true,
          },
        ],
        pagination: {
          page: 1,
          limit: 20,
          total: 1,
          hasNext: false,
          hasPrevious: false,
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await service.getImportHistory(1, 20);

      expect(mockFetch).toHaveBeenCalledWith(
        `${baseURL}/api/v1/recipes/import/history?page=1&limit=20`,
        {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${authToken}`,
          },
        }
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('quickImport', () => {
    it('performs quick import with minimal parameters', async () => {
      const mockResponse = {
        success: true,
        personalRecipeId: 'new-recipe-id',
        message: 'Recipe successfully imported',
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await service.quickImport('test-recipe-id', true);

      expect(mockFetch).toHaveBeenCalledWith(
        `${baseURL}/api/v1/recipes/import`,
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({
            communityRecipeId: 'test-recipe-id',
            preserveAttribution: true,
          }),
        })
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('batchImport', () => {
    it('imports multiple recipes with progress tracking', async () => {
      const recipeIds = ['recipe-1', 'recipe-2', 'recipe-3'];
      const mockResponses = recipeIds.map((id) => ({
        success: true,
        personalRecipeId: `imported-${id}`,
        message: 'Recipe successfully imported',
      }));

      // Mock successful responses for all recipes
      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockResponses[0],
        } as Response)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockResponses[1],
        } as Response)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockResponses[2],
        } as Response);

      const onProgress = jest.fn();
      const result = await service.batchImport(recipeIds, true, onProgress);

      expect(result.successful).toHaveLength(3);
      expect(result.failed).toHaveLength(0);
      expect(onProgress).toHaveBeenCalledTimes(3);
      expect(onProgress).toHaveBeenLastCalledWith(3, 3);
    });

    it('handles partial failures in batch import', async () => {
      const recipeIds = ['recipe-1', 'recipe-2'];

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ success: true, personalRecipeId: 'imported-recipe-1' }),
        } as Response)
        .mockResolvedValueOnce({
          ok: false,
          status: 404,
          json: async () => ({ error: 'Recipe not found' }),
        } as Response);

      const result = await service.batchImport(recipeIds);

      expect(result.successful).toHaveLength(1);
      expect(result.failed).toHaveLength(1);
      expect(result.failed[0]).toEqual({
        id: 'recipe-2',
        error: 'Recipe not found',
      });
    });
  });

  describe('resolveConflict', () => {
    it('returns null for cancel resolution', async () => {
      const result = await service.resolveConflict('test-recipe-id', 'cancel');
      expect(result).toBeNull();
      expect(mockFetch).not.toHaveBeenCalled();
    });

    it('automatically renames recipe for rename resolution', async () => {
      const mockResponse = {
        success: true,
        personalRecipeId: 'new-recipe-id',
        message: 'Recipe successfully imported',
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await service.resolveConflict('test-recipe-id', 'rename');

      expect(mockFetch).toHaveBeenCalledWith(
        `${baseURL}/api/v1/recipes/import`,
        expect.objectContaining({
          body: expect.stringContaining('Imported Recipe'),
        })
      );
      expect(result).toEqual(mockResponse);
    });

    it('uses custom title for rename resolution when provided', async () => {
      const mockResponse = {
        success: true,
        personalRecipeId: 'new-recipe-id',
        message: 'Recipe successfully imported',
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      await service.resolveConflict('test-recipe-id', 'rename', {
        title: 'Custom Title',
      });

      expect(mockFetch).toHaveBeenCalledWith(
        `${baseURL}/api/v1/recipes/import`,
        expect.objectContaining({
          body: expect.stringContaining('Custom Title'),
        })
      );
    });
  });

  describe('authentication', () => {
    it('includes auth headers when token is set', async () => {
      const mockResponse = { hasConflict: false };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      await service.checkImportConflict('test-recipe-id');

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            Authorization: `Bearer ${authToken}`,
          }),
        })
      );
    });

    it('excludes auth headers when token is not set', async () => {
      const serviceWithoutAuth = new RecipeImportService(baseURL);
      const mockResponse = { hasConflict: false };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      await serviceWithoutAuth.checkImportConflict('test-recipe-id');

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: {
            'Content-Type': 'application/json',
          },
        })
      );
    });
  });
});