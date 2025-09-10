import type { Recipe } from '@imkitchen/shared-types';
import type { SwapSuggestion, SwapFilters } from '../components/meal-plans/QuickSwapModal';

interface SimilarityWeights {
  mealType: number;
  complexity: number;
  prepTime: number;
  cuisine: number;
  ingredients: number;
  nutrition: number;
  userPreferences: number;
}

interface NutritionProfile {
  calories?: number;
  protein?: number;
  carbs?: number;
  fat?: number;
  fiber?: number;
}

interface UserPreferences {
  favoriteIngredients?: string[];
  dislikedIngredients?: string[];
  preferredComplexity?: 'simple' | 'moderate' | 'complex';
  preferredCuisines?: string[];
  maxPrepTime?: number;
  dietaryRestrictions?: string[];
}

export class SwapSuggestionService {
  private static readonly DEFAULT_WEIGHTS: SimilarityWeights = {
    mealType: 0.25,     // Must match meal type
    complexity: 0.15,   // Similar complexity level
    prepTime: 0.15,     // Similar preparation time
    cuisine: 0.10,      // Same or compatible cuisine
    ingredients: 0.15,  // Shared ingredients
    nutrition: 0.10,    // Similar nutritional profile
    userPreferences: 0.10, // User's historical preferences
  };

  private weights: SimilarityWeights;
  private userPreferences?: UserPreferences;

  constructor(weights?: Partial<SimilarityWeights>, userPreferences?: UserPreferences) {
    this.weights = { ...SwapSuggestionService.DEFAULT_WEIGHTS, ...weights };
    this.userPreferences = userPreferences;
  }

  /**
   * Calculate compatibility score between original recipe and potential replacement
   */
  calculateCompatibilityScore(originalRecipe: Recipe, candidateRecipe: Recipe): number {
    const scores = {
      mealType: this.calculateMealTypeScore(originalRecipe, candidateRecipe),
      complexity: this.calculateComplexityScore(originalRecipe, candidateRecipe),
      prepTime: this.calculatePrepTimeScore(originalRecipe, candidateRecipe),
      cuisine: this.calculateCuisineScore(originalRecipe, candidateRecipe),
      ingredients: this.calculateIngredientScore(originalRecipe, candidateRecipe),
      nutrition: this.calculateNutritionScore(originalRecipe, candidateRecipe),
      userPreferences: this.calculateUserPreferenceScore(candidateRecipe),
    };

    // Calculate weighted average
    let totalScore = 0;
    let totalWeight = 0;

    Object.entries(scores).forEach(([key, score]) => {
      if (score !== null) {
        const weight = this.weights[key as keyof SimilarityWeights];
        totalScore += score * weight;
        totalWeight += weight;
      }
    });

    return totalWeight > 0 ? totalScore / totalWeight : 0;
  }

  /**
   * Generate similarity reasons based on comparison
   */
  generateSimilarityReasons(originalRecipe: Recipe, candidateRecipe: Recipe): string[] {
    const reasons: string[] = [];

    // Meal type match
    const commonMealTypes = originalRecipe.mealType.filter(mt => 
      candidateRecipe.mealType.includes(mt)
    );
    if (commonMealTypes.length > 0) {
      reasons.push(`Perfect for ${commonMealTypes.join(' or ')}`);
    }

    // Prep time similarity
    const timeDiff = Math.abs((originalRecipe.prepTime || 0) - (candidateRecipe.prepTime || 0));
    if (timeDiff <= 10) {
      reasons.push('Similar preparation time');
    } else if (candidateRecipe.prepTime < originalRecipe.prepTime) {
      reasons.push('Faster to prepare');
    }

    // Complexity match
    if (originalRecipe.complexity === candidateRecipe.complexity) {
      reasons.push(`Same ${originalRecipe.complexity} complexity level`);
    } else if (this.isComplexityUpgrade(originalRecipe.complexity, candidateRecipe.complexity)) {
      reasons.push('Slightly more sophisticated version');
    }

    // Ingredient overlap
    const sharedIngredients = this.getSharedIngredients(originalRecipe, candidateRecipe);
    if (sharedIngredients.length > 0) {
      reasons.push(`Uses ${sharedIngredients.length} similar ingredients`);
    }

    // Cuisine compatibility
    if (originalRecipe.cuisineType === candidateRecipe.cuisineType) {
      reasons.push(`Same ${originalRecipe.cuisineType} cuisine`);
    }

    // Dietary compatibility
    const sharedDietaryLabels = originalRecipe.dietaryLabels.filter(label =>
      candidateRecipe.dietaryLabels.includes(label)
    );
    if (sharedDietaryLabels.length > 0) {
      reasons.push(`Maintains ${sharedDietaryLabels.join(', ')} requirements`);
    }

    // User preference match
    if (this.userPreferences?.favoriteIngredients) {
      const favoriteIngredients = candidateRecipe.ingredients.filter(ing =>
        this.userPreferences!.favoriteIngredients!.some(fav => 
          ing.name.toLowerCase().includes(fav.toLowerCase())
        )
      );
      if (favoriteIngredients.length > 0) {
        reasons.push('Contains ingredients you love');
      }
    }

    // Rating-based recommendation
    if (candidateRecipe.averageRating >= 4.0) {
      reasons.push(`Highly rated (${candidateRecipe.averageRating.toFixed(1)} stars)`);
    }

    return reasons.slice(0, 4); // Limit to top 4 reasons
  }

  /**
   * Filter and rank recipe suggestions based on criteria
   */
  async getSuggestions(
    originalRecipe: Recipe,
    candidateRecipes: Recipe[],
    filters: SwapFilters = {}
  ): Promise<SwapSuggestion[]> {
    let filteredRecipes = candidateRecipes;

    // Apply filters
    if (filters.maxPrepTime) {
      filteredRecipes = filteredRecipes.filter(recipe => 
        (recipe.prepTime || 0) <= filters.maxPrepTime!
      );
    }

    if (filters.complexity) {
      filteredRecipes = filteredRecipes.filter(recipe => 
        recipe.complexity === filters.complexity
      );
    }

    if (filters.cuisine) {
      filteredRecipes = filteredRecipes.filter(recipe => 
        recipe.cuisineType === filters.cuisine
      );
    }

    if (filters.maxTimeDifference) {
      filteredRecipes = filteredRecipes.filter(recipe => {
        const timeDiff = Math.abs((originalRecipe.totalTime || 0) - (recipe.totalTime || 0));
        return timeDiff <= filters.maxTimeDifference!;
      });
    }

    if (filters.excludeRecipeIds) {
      filteredRecipes = filteredRecipes.filter(recipe => 
        !filters.excludeRecipeIds!.includes(recipe.id)
      );
    }

    // Calculate compatibility scores and create suggestions
    const suggestions: SwapSuggestion[] = filteredRecipes
      .map(recipe => {
        const compatibilityScore = this.calculateCompatibilityScore(originalRecipe, recipe);
        const reasons = this.generateSimilarityReasons(originalRecipe, recipe);
        const timeDifference = (recipe.totalTime || 0) - (originalRecipe.totalTime || 0);
        
        return {
          recipe,
          compatibilityScore,
          reasons,
          timeDifference,
          complexityMatch: originalRecipe.complexity === recipe.complexity,
          cuisineMatch: originalRecipe.cuisineType === recipe.cuisineType,
          shoppingListImpact: {
            itemsAdded: this.estimateNewIngredients(originalRecipe, recipe),
            itemsRemoved: this.estimateRemovedIngredients(originalRecipe, recipe),
            estimatedCostChange: this.estimateCostChange(originalRecipe, recipe),
          },
        };
      })
      .filter(suggestion => suggestion.compatibilityScore > 0.3) // Only show decent matches
      .sort((a, b) => b.compatibilityScore - a.compatibilityScore) // Sort by compatibility
      .slice(0, 10); // Limit to top 10 suggestions

    return suggestions;
  }

  // Private helper methods for score calculations

  private calculateMealTypeScore(original: Recipe, candidate: Recipe): number {
    const overlap = original.mealType.filter(mt => candidate.mealType.includes(mt)).length;
    const total = Math.max(original.mealType.length, candidate.mealType.length);
    return total > 0 ? overlap / total : 0;
  }

  private calculateComplexityScore(original: Recipe, candidate: Recipe): number {
    const complexityOrder = { 'simple': 1, 'moderate': 2, 'complex': 3 };
    const originalLevel = complexityOrder[original.complexity] || 2;
    const candidateLevel = complexityOrder[candidate.complexity] || 2;
    
    // Perfect match gets 1.0, adjacent levels get 0.7, distant levels get 0.3
    const difference = Math.abs(originalLevel - candidateLevel);
    if (difference === 0) return 1.0;
    if (difference === 1) return 0.7;
    return 0.3;
  }

  private calculatePrepTimeScore(original: Recipe, candidate: Recipe): number {
    const originalTime = original.prepTime || 0;
    const candidateTime = candidate.prepTime || 0;
    
    if (originalTime === 0 && candidateTime === 0) return 1.0;
    if (originalTime === 0 || candidateTime === 0) return 0.5;
    
    const timeDiff = Math.abs(originalTime - candidateTime);
    const maxTime = Math.max(originalTime, candidateTime);
    
    // Closer prep times get higher scores
    return Math.max(0, 1 - (timeDiff / maxTime));
  }

  private calculateCuisineScore(original: Recipe, candidate: Recipe): number {
    if (!original.cuisineType || !candidate.cuisineType) return 0.5;
    if (original.cuisineType === candidate.cuisineType) return 1.0;
    
    // Could implement cuisine compatibility matrix here
    // For now, different cuisines get a moderate score
    return 0.3;
  }

  private calculateIngredientScore(original: Recipe, candidate: Recipe): number {
    const originalIngredients = new Set(
      original.ingredients.map(ing => ing.name.toLowerCase())
    );
    const candidateIngredients = new Set(
      candidate.ingredients.map(ing => ing.name.toLowerCase())
    );
    
    const intersection = new Set(
      [...originalIngredients].filter(x => candidateIngredients.has(x))
    );
    const union = new Set([...originalIngredients, ...candidateIngredients]);
    
    // Jaccard similarity coefficient
    return union.size > 0 ? intersection.size / union.size : 0;
  }

  private calculateNutritionScore(original: Recipe, candidate: Recipe): number {
    // This would require nutrition data - return neutral score for now
    return 0.5;
  }

  private calculateUserPreferenceScore(candidate: Recipe): number {
    if (!this.userPreferences) return 0.5;

    let score = 0.5;

    // Favorite ingredients boost
    if (this.userPreferences.favoriteIngredients) {
      const hasFavorites = candidate.ingredients.some(ing =>
        this.userPreferences!.favoriteIngredients!.some(fav =>
          ing.name.toLowerCase().includes(fav.toLowerCase())
        )
      );
      if (hasFavorites) score += 0.2;
    }

    // Disliked ingredients penalty
    if (this.userPreferences.dislikedIngredients) {
      const hasDislikes = candidate.ingredients.some(ing =>
        this.userPreferences!.dislikedIngredients!.some(dislike =>
          ing.name.toLowerCase().includes(dislike.toLowerCase())
        )
      );
      if (hasDislikes) score -= 0.3;
    }

    // Preferred complexity
    if (this.userPreferences.preferredComplexity === candidate.complexity) {
      score += 0.1;
    }

    // Preferred cuisines
    if (this.userPreferences.preferredCuisines?.includes(candidate.cuisineType || '')) {
      score += 0.1;
    }

    return Math.max(0, Math.min(1, score));
  }

  private isComplexityUpgrade(original: string, candidate: string): boolean {
    const complexityOrder = { 'simple': 1, 'moderate': 2, 'complex': 3 };
    const originalLevel = complexityOrder[original] || 2;
    const candidateLevel = complexityOrder[candidate] || 2;
    return candidateLevel === originalLevel + 1;
  }

  private getSharedIngredients(original: Recipe, candidate: Recipe): string[] {
    const originalNames = new Set(original.ingredients.map(ing => ing.name.toLowerCase()));
    return candidate.ingredients
      .filter(ing => originalNames.has(ing.name.toLowerCase()))
      .map(ing => ing.name);
  }

  private estimateNewIngredients(original: Recipe, candidate: Recipe): number {
    const originalNames = new Set(original.ingredients.map(ing => ing.name.toLowerCase()));
    return candidate.ingredients.filter(ing => 
      !originalNames.has(ing.name.toLowerCase())
    ).length;
  }

  private estimateRemovedIngredients(original: Recipe, candidate: Recipe): number {
    const candidateNames = new Set(candidate.ingredients.map(ing => ing.name.toLowerCase()));
    return original.ingredients.filter(ing => 
      !candidateNames.has(ing.name.toLowerCase())
    ).length;
  }

  private estimateCostChange(original: Recipe, candidate: Recipe): number {
    // This would require ingredient pricing data
    // For now, return a simple estimation based on ingredient count difference
    const ingredientDiff = candidate.ingredients.length - original.ingredients.length;
    return ingredientDiff * 2.50; // Rough estimate: $2.50 per additional ingredient
  }

  /**
   * Update user preferences based on user interactions
   */
  updateUserPreferences(preferences: Partial<UserPreferences>): void {
    this.userPreferences = { ...this.userPreferences, ...preferences };
  }

  /**
   * Learn from user swap choices to improve future suggestions
   */
  recordSwapChoice(
    originalRecipe: Recipe,
    chosenRecipe: Recipe,
    rejectedSuggestions: Recipe[]
  ): void {
    // In a full implementation, this would update ML models or preference weights
    // For now, we could update user preferences based on patterns
    
    if (!this.userPreferences) {
      this.userPreferences = {};
    }

    // Learn preferred complexity
    if (!this.userPreferences.preferredComplexity) {
      this.userPreferences.preferredComplexity = chosenRecipe.complexity as any;
    }

    // Learn preferred cuisines
    if (chosenRecipe.cuisineType) {
      if (!this.userPreferences.preferredCuisines) {
        this.userPreferences.preferredCuisines = [];
      }
      if (!this.userPreferences.preferredCuisines.includes(chosenRecipe.cuisineType)) {
        this.userPreferences.preferredCuisines.push(chosenRecipe.cuisineType);
      }
    }

    // Learn ingredient preferences from chosen recipe
    chosenRecipe.ingredients.forEach(ingredient => {
      if (!this.userPreferences!.favoriteIngredients) {
        this.userPreferences!.favoriteIngredients = [];
      }
      
      const ingredientName = ingredient.name.toLowerCase();
      if (!this.userPreferences!.favoriteIngredients.includes(ingredientName)) {
        this.userPreferences!.favoriteIngredients.push(ingredientName);
      }
    });

    console.log('Recorded swap choice for future improvements:', {
      from: originalRecipe.title,
      to: chosenRecipe.title,
      rejectedCount: rejectedSuggestions.length
    });
  }
}