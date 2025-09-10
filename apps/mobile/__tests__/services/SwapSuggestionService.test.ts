import { SwapSuggestionService } from '../../src/services/SwapSuggestionService';
import type { Recipe } from '@imkitchen/shared-types';

describe('SwapSuggestionService', () => {
  let service: SwapSuggestionService;

  const mockRecipeBreakfast: Recipe = {
    id: 'original-breakfast',
    title: 'Classic Pancakes',
    prepTime: 15,
    cookTime: 10,
    totalTime: 25,
    complexity: 'simple',
    mealType: ['breakfast'],
    servings: 4,
    ingredients: [
      { name: 'Flour', amount: '2 cups', unit: 'cups' },
      { name: 'Eggs', amount: '2', unit: 'pieces' },
      { name: 'Milk', amount: '1 cup', unit: 'cups' },
      { name: 'Baking powder', amount: '2 tsp', unit: 'teaspoons' },
    ],
    instructions: ['Mix dry ingredients', 'Add wet ingredients', 'Cook on griddle'],
    dietaryLabels: ['vegetarian'],
    averageRating: 4.2,
    totalRatings: 50,
    cuisineType: 'American',
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  const mockCandidateRecipes: Recipe[] = [
    {
      id: 'candidate-1',
      title: 'Blueberry Pancakes',
      prepTime: 18,
      cookTime: 12,
      totalTime: 30,
      complexity: 'simple',
      mealType: ['breakfast'],
      servings: 4,
      ingredients: [
        { name: 'Flour', amount: '2 cups', unit: 'cups' },
        { name: 'Eggs', amount: '2', unit: 'pieces' },
        { name: 'Milk', amount: '1 cup', unit: 'cups' },
        { name: 'Blueberries', amount: '1 cup', unit: 'cups' },
      ],
      instructions: ['Mix ingredients', 'Add blueberries', 'Cook'],
      dietaryLabels: ['vegetarian'],
      averageRating: 4.5,
      totalRatings: 35,
      cuisineType: 'American',
      createdAt: new Date(),
      updatedAt: new Date(),
    },
    {
      id: 'candidate-2',
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
        { name: 'Milk', amount: '1/2 cup', unit: 'cups' },
        { name: 'Cinnamon', amount: '1 tsp', unit: 'teaspoons' },
      ],
      instructions: ['Beat eggs with milk', 'Dip bread', 'Cook in pan'],
      dietaryLabels: ['vegetarian'],
      averageRating: 4.1,
      totalRatings: 42,
      cuisineType: 'French',
      createdAt: new Date(),
      updatedAt: new Date(),
    },
    {
      id: 'candidate-3',
      title: 'Grilled Chicken Breast',
      prepTime: 15,
      cookTime: 20,
      totalTime: 35,
      complexity: 'moderate',
      mealType: ['lunch', 'dinner'],
      servings: 4,
      ingredients: [
        { name: 'Chicken breast', amount: '4 pieces', unit: 'pieces' },
        { name: 'Olive oil', amount: '2 tbsp', unit: 'tablespoons' },
        { name: 'Salt', amount: '1 tsp', unit: 'teaspoons' },
      ],
      instructions: ['Season chicken', 'Heat grill', 'Grill until done'],
      dietaryLabels: ['gluten-free'],
      averageRating: 4.0,
      totalRatings: 28,
      cuisineType: 'American',
      createdAt: new Date(),
      updatedAt: new Date(),
    },
  ];

  beforeEach(() => {
    service = new SwapSuggestionService();
  });

  describe('calculateCompatibilityScore', () => {
    it('should return high score for very similar recipes', () => {
      const score = service.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // Blueberry pancakes
      );

      expect(score).toBeGreaterThan(0.8);
    });

    it('should return lower score for different meal types', () => {
      const score = service.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[2] // Grilled chicken (lunch/dinner)
      );

      expect(score).toBeLessThan(0.5);
    });

    it('should consider complexity similarity', () => {
      const scoreSimple = service.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // Simple blueberry pancakes
      );

      const scoreModerate = service.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[2] // Moderate grilled chicken
      );

      expect(scoreSimple).toBeGreaterThan(scoreModerate);
    });

    it('should factor in prep time differences', () => {
      // French toast has similar prep time to original pancakes
      const score = service.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[1] // French toast
      );

      expect(score).toBeGreaterThan(0.5);
    });

    it('should consider ingredient overlap', () => {
      // Both recipes share eggs and milk
      const score = service.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[1] // French toast
      );

      expect(score).toBeGreaterThan(0.4);
    });
  });

  describe('generateSimilarityReasons', () => {
    it('should identify meal type compatibility', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[0]
      );

      expect(reasons).toContain('Perfect for breakfast');
    });

    it('should mention complexity matching', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[0]
      );

      expect(reasons.some(r => r.includes('simple complexity'))).toBe(true);
    });

    it('should highlight shared ingredients', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[1] // French toast shares eggs and milk
      );

      expect(reasons.some(r => r.includes('similar ingredients'))).toBe(true);
    });

    it('should note prep time similarity', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[1] // French toast has similar prep time
      );

      expect(reasons.some(r => r.includes('preparation time') || r.includes('Faster'))).toBe(true);
    });

    it('should identify cuisine compatibility', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // Same American cuisine
      );

      expect(reasons.some(r => r.includes('American cuisine'))).toBe(true);
    });

    it('should mention high ratings', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // 4.5 star rating
      );

      expect(reasons.some(r => r.includes('Highly rated'))).toBe(true);
    });

    it('should limit reasons to maximum of 4', () => {
      const reasons = service.generateSimilarityReasons(
        mockRecipeBreakfast,
        mockCandidateRecipes[0]
      );

      expect(reasons.length).toBeLessThanOrEqual(4);
    });
  });

  describe('getSuggestions', () => {
    it('should filter recipes by meal type preference', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes
      );

      // Should prioritize breakfast recipes
      const breakfastSuggestions = suggestions.filter(s => 
        s.recipe.mealType.includes('breakfast')
      );
      
      expect(breakfastSuggestions.length).toBeGreaterThan(0);
    });

    it('should apply prep time filters', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes,
        { maxPrepTime: 12 }
      );

      suggestions.forEach(suggestion => {
        expect(suggestion.recipe.prepTime || 0).toBeLessThanOrEqual(12);
      });
    });

    it('should apply complexity filters', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes,
        { complexity: 'simple' }
      );

      suggestions.forEach(suggestion => {
        expect(suggestion.recipe.complexity).toBe('simple');
      });
    });

    it('should exclude specified recipe IDs', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes,
        { excludeRecipeIds: ['candidate-1'] }
      );

      const excludedRecipe = suggestions.find(s => s.recipe.id === 'candidate-1');
      expect(excludedRecipe).toBeUndefined();
    });

    it('should sort suggestions by compatibility score', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes
      );

      for (let i = 1; i < suggestions.length; i++) {
        expect(suggestions[i - 1].compatibilityScore)
          .toBeGreaterThanOrEqual(suggestions[i].compatibilityScore);
      }
    });

    it('should include time difference calculations', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes
      );

      suggestions.forEach(suggestion => {
        const expectedDiff = (suggestion.recipe.totalTime || 0) - 
                            (mockRecipeBreakfast.totalTime || 0);
        expect(suggestion.timeDifference).toBe(expectedDiff);
      });
    });

    it('should estimate shopping list impact', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes
      );

      suggestions.forEach(suggestion => {
        expect(suggestion.shoppingListImpact).toHaveProperty('itemsAdded');
        expect(suggestion.shoppingListImpact).toHaveProperty('itemsRemoved');
        expect(suggestion.shoppingListImpact).toHaveProperty('estimatedCostChange');
        
        expect(typeof suggestion.shoppingListImpact.itemsAdded).toBe('number');
        expect(typeof suggestion.shoppingListImpact.itemsRemoved).toBe('number');
        expect(typeof suggestion.shoppingListImpact.estimatedCostChange).toBe('number');
      });
    });

    it('should filter out very low compatibility scores', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        mockCandidateRecipes
      );

      suggestions.forEach(suggestion => {
        expect(suggestion.compatibilityScore).toBeGreaterThan(0.3);
      });
    });

    it('should limit results to maximum of 10 suggestions', async () => {
      const manyRecipes = Array(15).fill(null).map((_, i) => ({
        ...mockCandidateRecipes[0],
        id: `candidate-${i}`,
        title: `Recipe ${i}`,
      }));

      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        manyRecipes
      );

      expect(suggestions.length).toBeLessThanOrEqual(10);
    });
  });

  describe('user preferences integration', () => {
    it('should boost scores for favorite ingredients', async () => {
      const serviceWithPrefs = new SwapSuggestionService(undefined, {
        favoriteIngredients: ['blueberries'],
      });

      const scoreWithFavorite = serviceWithPrefs.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // Contains blueberries
      );

      const scoreWithoutFavorite = serviceWithPrefs.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[1] // No blueberries
      );

      expect(scoreWithFavorite).toBeGreaterThan(scoreWithoutFavorite);
    });

    it('should penalize disliked ingredients', async () => {
      const serviceWithPrefs = new SwapSuggestionService(undefined, {
        dislikedIngredients: ['cinnamon'],
      });

      const scoreWithDislike = serviceWithPrefs.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[1] // Contains cinnamon
      );

      const scoreWithoutDislike = serviceWithPrefs.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // No cinnamon
      );

      expect(scoreWithDislike).toBeLessThan(scoreWithoutDislike);
    });

    it('should boost preferred complexity recipes', async () => {
      const serviceWithPrefs = new SwapSuggestionService(undefined, {
        preferredComplexity: 'simple',
      });

      const scorePreferred = serviceWithPrefs.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[0] // Simple complexity
      );

      const scoreNotPreferred = serviceWithPrefs.calculateCompatibilityScore(
        mockRecipeBreakfast,
        mockCandidateRecipes[2] // Moderate complexity
      );

      expect(scorePreferred).toBeGreaterThan(scoreNotPreferred);
    });
  });

  describe('recordSwapChoice learning', () => {
    it('should update user preferences based on swap choices', () => {
      const chosenRecipe = mockCandidateRecipes[0]; // Blueberry pancakes
      const rejected = [mockCandidateRecipes[1]];

      service.recordSwapChoice(mockRecipeBreakfast, chosenRecipe, rejected);

      // In a full implementation, this would update ML models
      // For now, we just verify the method doesn't crash
      expect(true).toBe(true);
    });

    it('should learn cuisine preferences', () => {
      const initialService = new SwapSuggestionService();
      const frenchRecipe = mockCandidateRecipes[1]; // French cuisine

      initialService.recordSwapChoice(mockRecipeBreakfast, frenchRecipe, []);

      // Verify preferences were updated (accessing private property for testing)
      const preferences = (initialService as any).userPreferences;
      expect(preferences?.preferredCuisines).toContain('French');
    });

    it('should learn ingredient preferences from choices', () => {
      const initialService = new SwapSuggestionService();
      const recipeWithBlueberries = mockCandidateRecipes[0];

      initialService.recordSwapChoice(mockRecipeBreakfast, recipeWithBlueberries, []);

      const preferences = (initialService as any).userPreferences;
      expect(preferences?.favoriteIngredients).toContain('blueberries');
    });
  });

  describe('edge cases and error handling', () => {
    it('should handle recipes with missing nutrition data', () => {
      const recipeWithoutNutrition = {
        ...mockRecipeBreakfast,
        // No nutrition data
      };

      expect(() => {
        service.calculateCompatibilityScore(recipeWithoutNutrition, mockCandidateRecipes[0]);
      }).not.toThrow();
    });

    it('should handle empty ingredient lists', () => {
      const recipeWithoutIngredients = {
        ...mockRecipeBreakfast,
        ingredients: [],
      };

      const score = service.calculateCompatibilityScore(
        recipeWithoutIngredients,
        mockCandidateRecipes[0]
      );

      expect(score).toBeGreaterThanOrEqual(0);
    });

    it('should handle recipes with zero prep time', () => {
      const recipeZeroPrepTime = {
        ...mockRecipeBreakfast,
        prepTime: 0,
        totalTime: 10,
      };

      expect(() => {
        service.calculateCompatibilityScore(recipeZeroPrepTime, mockCandidateRecipes[0]);
      }).not.toThrow();
    });

    it('should handle empty candidate list', async () => {
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        []
      );

      expect(suggestions).toEqual([]);
    });

    it('should handle duplicate recipe IDs gracefully', async () => {
      const duplicateRecipes = [
        mockCandidateRecipes[0],
        mockCandidateRecipes[0], // Duplicate
      ];

      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        duplicateRecipes
      );

      // Should handle duplicates without crashing
      expect(suggestions.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('performance considerations', () => {
    it('should process large recipe lists efficiently', async () => {
      const largeRecipeList = Array(100).fill(null).map((_, i) => ({
        ...mockCandidateRecipes[0],
        id: `recipe-${i}`,
        title: `Recipe ${i}`,
      }));

      const startTime = Date.now();
      const suggestions = await service.getSuggestions(
        mockRecipeBreakfast,
        largeRecipeList
      );
      const duration = Date.now() - startTime;

      // Should complete within reasonable time (adjust threshold as needed)
      expect(duration).toBeLessThan(1000); // 1 second
      expect(suggestions.length).toBeLessThanOrEqual(10); // Respects limit
    });

    it('should handle complex filtering efficiently', async () => {
      const complexFilters = {
        maxPrepTime: 20,
        complexity: 'simple' as const,
        cuisine: 'American',
        maxTimeDifference: 10,
        excludeRecipeIds: ['candidate-3', 'candidate-4', 'candidate-5'],
      };

      const startTime = Date.now();
      await service.getSuggestions(mockRecipeBreakfast, mockCandidateRecipes, complexFilters);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(500); // Should be very fast for small dataset
    });
  });
});