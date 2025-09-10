package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sort"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// RecipeIndex provides fast access to recipes based on pre-computed indices
type RecipeIndex struct {
	// Core indices for fast filtering
	ByMealType   map[string][]string `json:"byMealType"`   // meal_type -> recipe_ids
	ByComplexity map[string][]string `json:"byComplexity"` // complexity -> recipe_ids
	ByDietary    map[string][]string `json:"byDietary"`    // dietary_label -> recipe_ids
	ByCuisine    map[string][]string `json:"byCuisine"`    // cuisine_type -> recipe_ids
	ByPrepTime   map[string][]string `json:"byPrepTime"`   // time_range -> recipe_ids

	// Performance optimizations
	HighRated    []string `json:"highRated"`    // recipes with 4+ stars
	QuickMeals   []string `json:"quickMeals"`   // recipes <= 30 min total time
	WeekendMeals []string `json:"weekendMeals"` // complex/moderate weekend-appropriate

	// Recipe metadata for fast access
	RecipeMetadata map[string]*RecipeIndexEntry `json:"recipeMetadata"`

	// Index metadata
	LastUpdated time.Time `json:"lastUpdated"`
	RecipeCount int       `json:"recipeCount"`
	Version     string    `json:"version"`
}

// RecipeIndexEntry contains essential recipe data for fast filtering
type RecipeIndexEntry struct {
	ID            string   `json:"id"`
	Title         string   `json:"title"`
	MealTypes     []string `json:"mealTypes"`
	Complexity    string   `json:"complexity"`
	PrepTime      int      `json:"prepTime"`
	CookTime      int      `json:"cookTime"`
	TotalTime     int      `json:"totalTime"`
	DietaryLabels []string `json:"dietaryLabels"`
	CuisineType   string   `json:"cuisineType"`
	AverageRating float64  `json:"averageRating"`
	Servings      int      `json:"servings"`
	Score         float64  `json:"score"` // Pre-computed base score
}

// RecipeIndexService manages pre-computed recipe indices for fast meal plan generation
type RecipeIndexService interface {
	BuildUserIndex(ctx context.Context, userID uuid.UUID) (*RecipeIndex, error)
	GetUserIndex(ctx context.Context, userID uuid.UUID) (*RecipeIndex, error)
	RefreshUserIndex(ctx context.Context, userID uuid.UUID) error
	FindCandidates(ctx context.Context, userID uuid.UUID, criteria *RecipeSelectionCriteria) ([]string, error)
	GetRecipeMetadata(ctx context.Context, userID uuid.UUID, recipeIDs []string) ([]*RecipeIndexEntry, error)
	InvalidateUserIndex(ctx context.Context, userID uuid.UUID) error
}

type recipeIndexService struct {
	recipeRepo repositories.RecipeRepository
	cache      *CacheService
	indexTTL   time.Duration
}

func NewRecipeIndexService(recipeRepo repositories.RecipeRepository, cache *CacheService) RecipeIndexService {
	return &recipeIndexService{
		recipeRepo: recipeRepo,
		cache:      cache,
		indexTTL:   24 * time.Hour, // Rebuild indices daily
	}
}

// BuildUserIndex creates a comprehensive index for a user's available recipes
func (r *recipeIndexService) BuildUserIndex(ctx context.Context, userID uuid.UUID) (*RecipeIndex, error) {
	log.Printf("Building recipe index for user %s", userID.String())

	// Get all user's recipes with minimal filtering
	searchParams := &models.RecipeSearchParams{
		Page:      1,
		Limit:     1000, // Large limit to get comprehensive dataset
		SortBy:    "average_rating",
		SortOrder: "desc",
	}

	startTime := time.Now()
	searchResult, err := r.recipeRepo.Search(userID, searchParams)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch recipes for indexing: %w", err)
	}
	log.Printf("Fetched %d recipes in %v", len(searchResult.Recipes), time.Since(startTime))

	// Initialize index structure
	index := &RecipeIndex{
		ByMealType:     make(map[string][]string),
		ByComplexity:   make(map[string][]string),
		ByDietary:      make(map[string][]string),
		ByCuisine:      make(map[string][]string),
		ByPrepTime:     make(map[string][]string),
		HighRated:      make([]string, 0),
		QuickMeals:     make([]string, 0),
		WeekendMeals:   make([]string, 0),
		RecipeMetadata: make(map[string]*RecipeIndexEntry),
		LastUpdated:    time.Now(),
		RecipeCount:    len(searchResult.Recipes),
		Version:        "1.0",
	}

	// Build indices
	for _, recipe := range searchResult.Recipes {
		recipeID := recipe.ID.String()

		// Create metadata entry
		metadata := &RecipeIndexEntry{
			ID:            recipeID,
			Title:         recipe.Title,
			MealTypes:     recipe.MealType,
			Complexity:    recipe.Complexity,
			PrepTime:      recipe.PrepTime,
			CookTime:      recipe.CookTime,
			TotalTime:     recipe.PrepTime + recipe.CookTime,
			DietaryLabels: recipe.DietaryLabels,
			AverageRating: recipe.AverageRating,
			Servings:      recipe.Servings,
			Score:         r.calculateBaseScore(recipe),
		}

		if recipe.CuisineType != nil {
			metadata.CuisineType = *recipe.CuisineType
		}

		index.RecipeMetadata[recipeID] = metadata

		// Index by meal type
		for _, mealType := range recipe.MealType {
			index.ByMealType[mealType] = append(index.ByMealType[mealType], recipeID)
		}

		// Index by complexity
		index.ByComplexity[recipe.Complexity] = append(index.ByComplexity[recipe.Complexity], recipeID)

		// Index by dietary labels
		for _, label := range recipe.DietaryLabels {
			index.ByDietary[label] = append(index.ByDietary[label], recipeID)
		}

		// Index by cuisine
		if recipe.CuisineType != nil {
			index.ByCuisine[*recipe.CuisineType] = append(index.ByCuisine[*recipe.CuisineType], recipeID)
		}

		// Index by prep time ranges
		totalTime := recipe.PrepTime + recipe.CookTime
		if totalTime <= 15 {
			index.ByPrepTime["ultra_quick"] = append(index.ByPrepTime["ultra_quick"], recipeID)
		} else if totalTime <= 30 {
			index.ByPrepTime["quick"] = append(index.ByPrepTime["quick"], recipeID)
		} else if totalTime <= 60 {
			index.ByPrepTime["moderate"] = append(index.ByPrepTime["moderate"], recipeID)
		} else {
			index.ByPrepTime["long"] = append(index.ByPrepTime["long"], recipeID)
		}

		// Special collections
		if recipe.AverageRating >= 4.0 {
			index.HighRated = append(index.HighRated, recipeID)
		}

		if totalTime <= 30 {
			index.QuickMeals = append(index.QuickMeals, recipeID)
		}

		if recipe.Complexity == "moderate" || recipe.Complexity == "complex" {
			index.WeekendMeals = append(index.WeekendMeals, recipeID)
		}
	}

	// Sort all indices by score for better performance
	r.sortIndicesByScore(index)

	// Cache the index
	cacheKey := fmt.Sprintf("recipe_index:%s", userID.String())
	if err := r.cache.Set(ctx, cacheKey, index, r.indexTTL); err != nil {
		log.Printf("Warning: failed to cache recipe index: %v", err)
	}

	log.Printf("Built recipe index with %d recipes in %v", index.RecipeCount, time.Since(startTime))
	return index, nil
}

// GetUserIndex retrieves the index for a user, building it if necessary
func (r *recipeIndexService) GetUserIndex(ctx context.Context, userID uuid.UUID) (*RecipeIndex, error) {
	cacheKey := fmt.Sprintf("recipe_index:%s", userID.String())

	// Try cache first
	cached, err := r.cache.Get(ctx, cacheKey)
	if err == nil {
		var index RecipeIndex
		if err := json.Unmarshal([]byte(cached), &index); err == nil {
			// Check if index is recent enough
			if time.Since(index.LastUpdated) < r.indexTTL {
				return &index, nil
			}
		}
	}

	// Build new index
	return r.BuildUserIndex(ctx, userID)
}

// RefreshUserIndex forces a rebuild of the user's index
func (r *recipeIndexService) RefreshUserIndex(ctx context.Context, userID uuid.UUID) error {
	_, err := r.BuildUserIndex(ctx, userID)
	return err
}

// FindCandidates uses the index to quickly find recipe candidates
func (r *recipeIndexService) FindCandidates(ctx context.Context, userID uuid.UUID, criteria *RecipeSelectionCriteria) ([]string, error) {
	index, err := r.GetUserIndex(ctx, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get recipe index: %w", err)
	}

	// Start with all recipes matching the meal type
	candidates := make(map[string]bool)
	if mealTypeRecipes, exists := index.ByMealType[criteria.MealType]; exists {
		for _, recipeID := range mealTypeRecipes {
			candidates[recipeID] = true
		}
	} else {
		// No recipes for this meal type
		return []string{}, nil
	}

	// Apply complexity filter
	if len(criteria.PreferredComplexity) > 0 {
		complexityMatches := make(map[string]bool)
		for _, complexity := range criteria.PreferredComplexity {
			if complexityRecipes, exists := index.ByComplexity[complexity]; exists {
				for _, recipeID := range complexityRecipes {
					complexityMatches[recipeID] = true
				}
			}
		}

		// Intersect with candidates
		filteredCandidates := make(map[string]bool)
		for recipeID := range candidates {
			if complexityMatches[recipeID] {
				filteredCandidates[recipeID] = true
			}
		}
		candidates = filteredCandidates
	}

	// Apply dietary restrictions filter
	if len(criteria.DietaryRestrictions) > 0 {
		for _, restriction := range criteria.DietaryRestrictions {
			if restrictionRecipes, exists := index.ByDietary[restriction]; exists {
				restrictionMatches := make(map[string]bool)
				for _, recipeID := range restrictionRecipes {
					restrictionMatches[recipeID] = true
				}

				// Intersect with candidates
				filteredCandidates := make(map[string]bool)
				for recipeID := range candidates {
					if restrictionMatches[recipeID] {
						filteredCandidates[recipeID] = true
					}
				}
				candidates = filteredCandidates
			} else {
				// No recipes match this restriction
				return []string{}, nil
			}
		}
	}

	// Apply cuisine preference filter
	if len(criteria.CuisinePreferences) > 0 {
		cuisineMatches := make(map[string]bool)
		for _, cuisine := range criteria.CuisinePreferences {
			if cuisineRecipes, exists := index.ByCuisine[cuisine]; exists {
				for _, recipeID := range cuisineRecipes {
					cuisineMatches[recipeID] = true
				}
			}
		}

		// Intersect with candidates (soft constraint)
		if len(cuisineMatches) > 0 {
			filteredCandidates := make(map[string]bool)
			for recipeID := range candidates {
				if cuisineMatches[recipeID] {
					filteredCandidates[recipeID] = true
				}
			}
			if len(filteredCandidates) > 0 {
				candidates = filteredCandidates
			}
		}
	}

	// Apply prep time filter
	if criteria.MaxPrepTime != nil {
		prepTimeMatches := make(map[string]bool)
		maxTime := *criteria.MaxPrepTime

		// Get appropriate time range buckets
		if maxTime <= 15 {
			for _, recipeID := range index.ByPrepTime["ultra_quick"] {
				prepTimeMatches[recipeID] = true
			}
		} else if maxTime <= 30 {
			for _, recipeID := range index.ByPrepTime["ultra_quick"] {
				prepTimeMatches[recipeID] = true
			}
			for _, recipeID := range index.ByPrepTime["quick"] {
				prepTimeMatches[recipeID] = true
			}
		} else if maxTime <= 60 {
			for _, recipeID := range index.ByPrepTime["ultra_quick"] {
				prepTimeMatches[recipeID] = true
			}
			for _, recipeID := range index.ByPrepTime["quick"] {
				prepTimeMatches[recipeID] = true
			}
			for _, recipeID := range index.ByPrepTime["moderate"] {
				prepTimeMatches[recipeID] = true
			}
		} else {
			// All time ranges allowed
			for _, timeRange := range []string{"ultra_quick", "quick", "moderate", "long"} {
				for _, recipeID := range index.ByPrepTime[timeRange] {
					prepTimeMatches[recipeID] = true
				}
			}
		}

		// Fine-grained time filtering using metadata
		refinedMatches := make(map[string]bool)
		for recipeID := range prepTimeMatches {
			if candidates[recipeID] {
				if metadata, exists := index.RecipeMetadata[recipeID]; exists {
					if metadata.TotalTime <= maxTime {
						refinedMatches[recipeID] = true
					}
				}
			}
		}
		candidates = refinedMatches
	}

	// Remove avoided recipes
	for _, avoidID := range criteria.AvoidRecipeIDs {
		delete(candidates, avoidID)
	}

	// Remove recipes used this week
	for usedID := range criteria.UsedThisWeek {
		delete(candidates, usedID)
	}

	// Convert to sorted slice
	result := make([]string, 0, len(candidates))
	for recipeID := range candidates {
		result = append(result, recipeID)
	}

	// Sort by pre-computed score for best candidates first
	sort.Slice(result, func(i, j int) bool {
		metaI := index.RecipeMetadata[result[i]]
		metaJ := index.RecipeMetadata[result[j]]
		if metaI != nil && metaJ != nil {
			return metaI.Score > metaJ.Score
		}
		return false
	})

	return result, nil
}

// GetRecipeMetadata returns metadata for specific recipe IDs
func (r *recipeIndexService) GetRecipeMetadata(ctx context.Context, userID uuid.UUID, recipeIDs []string) ([]*RecipeIndexEntry, error) {
	index, err := r.GetUserIndex(ctx, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get recipe index: %w", err)
	}

	result := make([]*RecipeIndexEntry, 0, len(recipeIDs))
	for _, recipeID := range recipeIDs {
		if metadata, exists := index.RecipeMetadata[recipeID]; exists {
			result = append(result, metadata)
		}
	}

	return result, nil
}

// InvalidateUserIndex removes the cached index for a user
func (r *recipeIndexService) InvalidateUserIndex(ctx context.Context, userID uuid.UUID) error {
	cacheKey := fmt.Sprintf("recipe_index:%s", userID.String())
	return r.cache.Delete(ctx, cacheKey)
}

// Helper methods

func (r *recipeIndexService) calculateBaseScore(recipe models.Recipe) float64 {
	score := 0.0

	// Base score from rating
	score += recipe.AverageRating * 20

	// Popularity bonus (more ratings = more reliable)
	if recipe.TotalRatings > 0 {
		popularityBonus := float64(recipe.TotalRatings) / 10.0
		if popularityBonus > 10 {
			popularityBonus = 10 // Cap at 10 points
		}
		score += popularityBonus
	}

	// Complexity scoring
	switch recipe.Complexity {
	case "simple":
		score += 15 // Simple recipes get higher base score
	case "moderate":
		score += 10
	case "complex":
		score += 5 // Complex recipes get lower base score but can be boosted by preferences
	}

	// Time efficiency bonus
	totalTime := recipe.PrepTime + recipe.CookTime
	if totalTime <= 30 {
		score += 10 // Quick meal bonus
	} else if totalTime <= 45 {
		score += 5 // Reasonable time bonus
	}

	// Meal type versatility (recipes that work for multiple meal types)
	if len(recipe.MealType) > 1 {
		score += 3
	}

	return score
}

func (r *recipeIndexService) sortIndicesByScore(index *RecipeIndex) {
	// Sort each index by pre-computed score for better performance
	for mealType, recipeIDs := range index.ByMealType {
		sort.Slice(recipeIDs, func(i, j int) bool {
			metaI := index.RecipeMetadata[recipeIDs[i]]
			metaJ := index.RecipeMetadata[recipeIDs[j]]
			if metaI != nil && metaJ != nil {
				return metaI.Score > metaJ.Score
			}
			return false
		})
		index.ByMealType[mealType] = recipeIDs
	}

	// Sort special collections
	sort.Slice(index.HighRated, func(i, j int) bool {
		metaI := index.RecipeMetadata[index.HighRated[i]]
		metaJ := index.RecipeMetadata[index.HighRated[j]]
		if metaI != nil && metaJ != nil {
			return metaI.Score > metaJ.Score
		}
		return false
	})

	sort.Slice(index.QuickMeals, func(i, j int) bool {
		metaI := index.RecipeMetadata[index.QuickMeals[i]]
		metaJ := index.RecipeMetadata[index.QuickMeals[j]]
		if metaI != nil && metaJ != nil {
			return metaI.Score > metaJ.Score
		}
		return false
	})
}
