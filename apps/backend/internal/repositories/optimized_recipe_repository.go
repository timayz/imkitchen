package repositories

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
)

// CacheServiceInterface defines the basic cache operations needed
type CacheServiceInterface interface {
	Get(key string) string
	Set(key string, value string, expiration time.Duration)
}

// OptimizedRecipeRepository provides high-performance recipe queries
type OptimizedRecipeRepository interface {
	// Fast search methods optimized for meal plan generation
	SearchOptimized(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error)
	SearchByMealTypeOptimized(ctx context.Context, userID uuid.UUID, mealType string, filters *models.RecipeFilters, limit int) ([]models.Recipe, error)
	SearchForMealPlanGeneration(ctx context.Context, userID uuid.UUID, requirements *MealPlanSearchRequirements) (*OptimizedRecipePool, error)

	// Bulk operations for performance
	GetRecipesByIDs(ctx context.Context, userID uuid.UUID, recipeIDs []uuid.UUID) ([]models.Recipe, error)
	GetRecipeMetadataByIDs(ctx context.Context, userID uuid.UUID, recipeIDs []uuid.UUID) ([]RecipeMetadata, error)

	// Cache-aware operations
	GetRecipesWithCaching(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams, cache CacheServiceInterface) (*models.RecipeSearchResponse, error)
	PreloadRecipePool(ctx context.Context, userID uuid.UUID, maxRecipes int) ([]models.Recipe, error)

	// Index management
	EnsureOptimalIndices(ctx context.Context) error
	AnalyzeQueryPerformance(ctx context.Context, userID uuid.UUID) (*QueryPerformanceReport, error)
}

// MealPlanSearchRequirements defines optimized search criteria for meal plan generation
type MealPlanSearchRequirements struct {
	UserID              uuid.UUID
	DietaryRestrictions []string
	CuisinePreferences  []string
	SkillLevel          string
	MaxPrepTimePerMeal  int
	AvoidRecipeIDs      []string
	RequiredMealTypes   []string
	MinRecipesPerType   int
}

// OptimizedRecipePool contains pre-filtered recipes organized for fast access
type OptimizedRecipePool struct {
	ByMealType   map[string][]models.Recipe `json:"byMealType"`
	ByComplexity map[string][]models.Recipe `json:"byComplexity"`
	HighRated    []models.Recipe            `json:"highRated"`
	QuickMeals   []models.Recipe            `json:"quickMeals"`
	TotalRecipes int                        `json:"totalRecipes"`
	GeneratedAt  time.Time                  `json:"generatedAt"`
	UserID       uuid.UUID                  `json:"userId"`
}

// RecipeMetadata contains minimal recipe data for fast operations
type RecipeMetadata struct {
	ID            uuid.UUID `json:"id"`
	Title         string    `json:"title"`
	MealType      []string  `json:"mealType"`
	Complexity    string    `json:"complexity"`
	PrepTime      int       `json:"prepTime"`
	CookTime      int       `json:"cookTime"`
	TotalTime     int       `json:"totalTime"`
	AverageRating float64   `json:"averageRating"`
	DietaryLabels []string  `json:"dietaryLabels"`
	CuisineType   *string   `json:"cuisineType"`
	Servings      int       `json:"servings"`
}

// QueryPerformanceReport contains query performance analysis
type QueryPerformanceReport struct {
	UserID                   uuid.UUID       `json:"userId"`
	AverageQueryTime         time.Duration   `json:"averageQueryTime"`
	SlowestQuery             time.Duration   `json:"slowestQuery"`
	TotalRecipes             int             `json:"totalRecipes"`
	IndexUsage               map[string]bool `json:"indexUsage"`
	RecommendedOptimizations []string        `json:"recommendedOptimizations"`
	GeneratedAt              time.Time       `json:"generatedAt"`
}

type optimizedRecipeRepository struct {
	db   *gorm.DB
	base RecipeRepository
}

func NewOptimizedRecipeRepository(db *gorm.DB, base RecipeRepository) OptimizedRecipeRepository {
	return &optimizedRecipeRepository{
		db:   db,
		base: base,
	}
}

// SearchOptimized provides performance-optimized recipe search
func (r *optimizedRecipeRepository) SearchOptimized(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {
	startTime := time.Now()

	var recipes []models.Recipe
	var total int64

	// Use optimized query with proper indexing
	query := r.db.WithContext(ctx).Model(&models.Recipe{}).
		Select("id, title, meal_type, complexity, prep_time, cook_time, average_rating, dietary_labels, cuisine_type, servings, created_at, updated_at").
		Where("deleted_at IS NULL AND (user_id = ? OR is_public = true)", userID)

	// Apply filters with index-friendly conditions
	query = r.applyOptimizedFilters(query, &params.RecipeFilters)

	// Get count with same filters (but without select to include all columns for counting)
	countQuery := r.db.WithContext(ctx).Model(&models.Recipe{}).
		Where("deleted_at IS NULL AND (user_id = ? OR is_public = true)", userID)
	countQuery = r.applyOptimizedFilters(countQuery, &params.RecipeFilters)

	if err := countQuery.Count(&total).Error; err != nil {
		return nil, fmt.Errorf("failed to count recipes: %w", err)
	}

	// Apply sorting with index optimization
	query = r.applyOptimizedSorting(query, params.SortBy, params.SortOrder)

	// Apply pagination
	offset := (params.Page - 1) * params.Limit
	if err := query.Offset(offset).Limit(params.Limit).Find(&recipes).Error; err != nil {
		return nil, fmt.Errorf("failed to fetch recipes: %w", err)
	}

	queryTime := time.Since(startTime)
	if queryTime > 500*time.Millisecond {
		log.Printf("SLOW QUERY WARNING: Recipe search took %v for user %s", queryTime, userID.String())
	}

	totalPages := int(total) / params.Limit
	if int(total)%params.Limit > 0 {
		totalPages++
	}

	return &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      total,
		Page:       params.Page,
		Limit:      params.Limit,
		TotalPages: totalPages,
	}, nil
}

// SearchByMealTypeOptimized provides fast meal-type specific searches
func (r *optimizedRecipeRepository) SearchByMealTypeOptimized(ctx context.Context, userID uuid.UUID, mealType string, filters *models.RecipeFilters, limit int) ([]models.Recipe, error) {
	var recipes []models.Recipe

	// Optimized query for single meal type with compound index usage
	query := r.db.WithContext(ctx).Model(&models.Recipe{}).
		Select("id, title, meal_type, complexity, prep_time, cook_time, average_rating, dietary_labels, cuisine_type, servings").
		Where("deleted_at IS NULL AND (user_id = ? OR is_public = true) AND ? = ANY(meal_type)", userID, mealType).
		Order("average_rating DESC, total_time ASC").
		Limit(limit)

	// Apply additional filters
	query = r.applyOptimizedFilters(query, filters)

	if err := query.Find(&recipes).Error; err != nil {
		return nil, fmt.Errorf("failed to search recipes by meal type: %w", err)
	}

	return recipes, nil
}

// SearchForMealPlanGeneration provides optimized search specifically for meal plan generation
func (r *optimizedRecipeRepository) SearchForMealPlanGeneration(ctx context.Context, userID uuid.UUID, requirements *MealPlanSearchRequirements) (*OptimizedRecipePool, error) {
	startTime := time.Now()

	pool := &OptimizedRecipePool{
		ByMealType:   make(map[string][]models.Recipe),
		ByComplexity: make(map[string][]models.Recipe),
		HighRated:    make([]models.Recipe, 0),
		QuickMeals:   make([]models.Recipe, 0),
		GeneratedAt:  time.Now(),
		UserID:       userID,
	}

	// Base query optimized for meal plan generation
	baseQuery := r.db.WithContext(ctx).Model(&models.Recipe{}).
		Select("id, title, meal_type, complexity, prep_time, cook_time, total_time, average_rating, dietary_labels, cuisine_type, servings").
		Where("deleted_at IS NULL AND (user_id = ? OR is_public = true)", userID)

	// Apply dietary restrictions (hard constraint)
	if len(requirements.DietaryRestrictions) > 0 {
		baseQuery = baseQuery.Where("dietary_labels @> ?", requirements.DietaryRestrictions)
	}

	// Apply prep time constraint
	if requirements.MaxPrepTimePerMeal > 0 {
		baseQuery = baseQuery.Where("total_time <= ?", requirements.MaxPrepTimePerMeal)
	}

	// Exclude avoided recipes
	if len(requirements.AvoidRecipeIDs) > 0 {
		baseQuery = baseQuery.Where("id NOT IN ?", requirements.AvoidRecipeIDs)
	}

	// Apply skill level constraints
	if requirements.SkillLevel == "beginner" {
		baseQuery = baseQuery.Where("complexity = 'simple'")
	} else if requirements.SkillLevel == "intermediate" {
		baseQuery = baseQuery.Where("complexity IN ('simple', 'moderate')")
	}

	// Get all matching recipes
	var allRecipes []models.Recipe
	if err := baseQuery.Order("average_rating DESC").Find(&allRecipes).Error; err != nil {
		return nil, fmt.Errorf("failed to fetch recipes for meal plan generation: %w", err)
	}

	// Organize recipes by meal type and complexity
	for _, recipe := range allRecipes {
		// Organize by meal type
		for _, mealType := range recipe.MealType {
			pool.ByMealType[mealType] = append(pool.ByMealType[mealType], recipe)
		}

		// Organize by complexity
		pool.ByComplexity[recipe.Complexity] = append(pool.ByComplexity[recipe.Complexity], recipe)

		// Special collections
		if recipe.AverageRating >= 4.0 {
			pool.HighRated = append(pool.HighRated, recipe)
		}

		totalTime := recipe.PrepTime + recipe.CookTime
		if totalTime <= 30 {
			pool.QuickMeals = append(pool.QuickMeals, recipe)
		}
	}

	pool.TotalRecipes = len(allRecipes)

	log.Printf("Generated optimized recipe pool with %d recipes in %v for user %s",
		pool.TotalRecipes, time.Since(startTime), userID.String())

	return pool, nil
}

// GetRecipesByIDs efficiently fetches multiple recipes by ID
func (r *optimizedRecipeRepository) GetRecipesByIDs(ctx context.Context, userID uuid.UUID, recipeIDs []uuid.UUID) ([]models.Recipe, error) {
	if len(recipeIDs) == 0 {
		return []models.Recipe{}, nil
	}

	var recipes []models.Recipe
	err := r.db.WithContext(ctx).
		Where("id IN ? AND (user_id = ? OR is_public = true) AND deleted_at IS NULL", recipeIDs, userID).
		Find(&recipes).Error

	if err != nil {
		return nil, fmt.Errorf("failed to fetch recipes by IDs: %w", err)
	}

	return recipes, nil
}

// GetRecipeMetadataByIDs efficiently fetches minimal recipe data
func (r *optimizedRecipeRepository) GetRecipeMetadataByIDs(ctx context.Context, userID uuid.UUID, recipeIDs []uuid.UUID) ([]RecipeMetadata, error) {
	if len(recipeIDs) == 0 {
		return []RecipeMetadata{}, nil
	}

	var recipes []models.Recipe
	err := r.db.WithContext(ctx).
		Select("id, title, meal_type, complexity, prep_time, cook_time, total_time, average_rating, dietary_labels, cuisine_type, servings").
		Where("id IN ? AND (user_id = ? OR is_public = true) AND deleted_at IS NULL", recipeIDs, userID).
		Find(&recipes).Error

	if err != nil {
		return nil, fmt.Errorf("failed to fetch recipe metadata: %w", err)
	}

	// Convert to metadata format
	metadata := make([]RecipeMetadata, len(recipes))
	for i, recipe := range recipes {
		metadata[i] = RecipeMetadata{
			ID:            recipe.ID,
			Title:         recipe.Title,
			MealType:      recipe.MealType,
			Complexity:    recipe.Complexity,
			PrepTime:      recipe.PrepTime,
			CookTime:      recipe.CookTime,
			TotalTime:     recipe.PrepTime + recipe.CookTime,
			AverageRating: recipe.AverageRating,
			DietaryLabels: recipe.DietaryLabels,
			CuisineType:   recipe.CuisineType,
			Servings:      recipe.Servings,
		}
	}

	return metadata, nil
}

// GetRecipesWithCaching implements cache-aside pattern for recipe searches
func (r *optimizedRecipeRepository) GetRecipesWithCaching(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams, cache CacheServiceInterface) (*models.RecipeSearchResponse, error) {
	// Generate cache key based on search parameters
	cacheKey := r.generateSearchCacheKey(userID, params)

	// Try cache first
	if cached := cache.Get(cacheKey); cached != "" {
		var response models.RecipeSearchResponse
		if err := json.Unmarshal([]byte(cached), &response); err == nil {
			log.Printf("Cache hit for recipe search: %s", cacheKey)
			return &response, nil
		}
	}

	// Cache miss - fetch from database
	response, err := r.SearchOptimized(ctx, userID, params)
	if err != nil {
		return nil, err
	}

	// Cache the result asynchronously
	go func() {
		if data, err := json.Marshal(response); err == nil {
			ttl := 15 * time.Minute // Shorter TTL for search results
			cache.Set(cacheKey, string(data), ttl)
		}
	}()

	return response, nil
}

// PreloadRecipePool loads a comprehensive recipe pool for a user
func (r *optimizedRecipeRepository) PreloadRecipePool(ctx context.Context, userID uuid.UUID, maxRecipes int) ([]models.Recipe, error) {
	var recipes []models.Recipe

	query := r.db.WithContext(ctx).
		Where("deleted_at IS NULL AND (user_id = ? OR is_public = true)", userID).
		Order("average_rating DESC, total_ratings DESC, created_at DESC").
		Limit(maxRecipes)

	if err := query.Find(&recipes).Error; err != nil {
		return nil, fmt.Errorf("failed to preload recipe pool: %w", err)
	}

	log.Printf("Preloaded %d recipes for user %s", len(recipes), userID.String())
	return recipes, nil
}

// EnsureOptimalIndices creates database indices for optimal query performance
func (r *optimizedRecipeRepository) EnsureOptimalIndices(ctx context.Context) error {
	indices := []string{
		// Compound index for meal plan generation queries
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_meal_plan_generation ON recipes (user_id, deleted_at, meal_type, complexity, total_time, average_rating) WHERE deleted_at IS NULL",

		// Index for dietary restrictions filtering (GIN index for array operations)
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_dietary_labels_gin ON recipes USING GIN (dietary_labels) WHERE deleted_at IS NULL",

		// Index for meal type array operations
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_meal_type_gin ON recipes USING GIN (meal_type) WHERE deleted_at IS NULL",

		// Index for public recipes filtering
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_public_access ON recipes (is_public, deleted_at, average_rating) WHERE deleted_at IS NULL AND is_public = true",

		// Index for time-based filtering
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_time_constraints ON recipes (prep_time, cook_time, total_time, deleted_at) WHERE deleted_at IS NULL",

		// Index for cuisine and complexity filtering
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_cuisine_complexity ON recipes (cuisine_type, complexity, deleted_at) WHERE deleted_at IS NULL",

		// Index for sorting by rating and popularity
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_popularity ON recipes (average_rating DESC, total_ratings DESC, created_at DESC) WHERE deleted_at IS NULL",
	}

	for _, indexSQL := range indices {
		if err := r.db.WithContext(ctx).Exec(indexSQL).Error; err != nil {
			log.Printf("Warning: Failed to create index: %v", err)
			// Continue with other indices even if one fails
		}
	}

	log.Printf("Ensured optimal database indices for recipe queries")
	return nil
}

// AnalyzeQueryPerformance analyzes query performance for a user
func (r *optimizedRecipeRepository) AnalyzeQueryPerformance(ctx context.Context, userID uuid.UUID) (*QueryPerformanceReport, error) {
	report := &QueryPerformanceReport{
		UserID:                   userID,
		IndexUsage:               make(map[string]bool),
		RecommendedOptimizations: make([]string, 0),
		GeneratedAt:              time.Now(),
	}

	// Get total recipes for user
	var totalRecipes int64
	r.db.WithContext(ctx).Model(&models.Recipe{}).
		Where("deleted_at IS NULL AND (user_id = ? OR is_public = true)", userID).
		Count(&totalRecipes)
	report.TotalRecipes = int(totalRecipes)

	// Test query performance
	testQueries := []struct {
		name  string
		query func() *gorm.DB
	}{
		{
			"meal_type_search",
			func() *gorm.DB {
				return r.db.WithContext(ctx).Model(&models.Recipe{}).
					Where("deleted_at IS NULL AND (user_id = ? OR is_public = true) AND ? = ANY(meal_type)", userID, "dinner")
			},
		},
		{
			"complexity_filter",
			func() *gorm.DB {
				return r.db.WithContext(ctx).Model(&models.Recipe{}).
					Where("deleted_at IS NULL AND (user_id = ? OR is_public = true) AND complexity = ?", userID, "simple")
			},
		},
		{
			"dietary_filter",
			func() *gorm.DB {
				return r.db.WithContext(ctx).Model(&models.Recipe{}).
					Where("deleted_at IS NULL AND (user_id = ? OR is_public = true) AND dietary_labels @> ?", userID, []string{"vegetarian"})
			},
		},
	}

	var totalTime time.Duration
	var maxTime time.Duration

	for _, test := range testQueries {
		start := time.Now()
		var count int64
		test.query().Count(&count)
		queryTime := time.Since(start)

		totalTime += queryTime
		if queryTime > maxTime {
			maxTime = queryTime
		}

		// Check if query is slow
		if queryTime > 100*time.Millisecond {
			report.RecommendedOptimizations = append(report.RecommendedOptimizations,
				fmt.Sprintf("Optimize %s query (took %v)", test.name, queryTime))
		}
	}

	report.AverageQueryTime = totalTime / time.Duration(len(testQueries))
	report.SlowestQuery = maxTime

	// Add general recommendations based on data size
	if report.TotalRecipes > 10000 {
		report.RecommendedOptimizations = append(report.RecommendedOptimizations,
			"Consider partitioning recipes table by user_id for large datasets")
	}

	if report.AverageQueryTime > 50*time.Millisecond {
		report.RecommendedOptimizations = append(report.RecommendedOptimizations,
			"Consider increasing cache TTL or implementing query result caching")
	}

	return report, nil
}

// Helper methods

func (r *optimizedRecipeRepository) applyOptimizedFilters(query *gorm.DB, filters *models.RecipeFilters) *gorm.DB {
	// Meal type filter - optimized for array operations
	if filters.MealType != nil && len(*filters.MealType) > 0 {
		// Use ANY for better index usage with single meal type
		if len(*filters.MealType) == 1 {
			query = query.Where("? = ANY(meal_type)", (*filters.MealType)[0])
		} else {
			query = query.Where("meal_type && ?", *filters.MealType)
		}
	}

	// Complexity filter - use IN for multiple values
	if filters.Complexity != nil && len(*filters.Complexity) > 0 {
		if len(*filters.Complexity) == 1 {
			query = query.Where("complexity = ?", (*filters.Complexity)[0])
		} else {
			query = query.Where("complexity IN ?", *filters.Complexity)
		}
	}

	// Time filters - optimize with compound conditions
	timeConditions := make([]string, 0)
	timeValues := make([]interface{}, 0)

	if filters.MaxPrepTime != nil {
		timeConditions = append(timeConditions, "prep_time <= ?")
		timeValues = append(timeValues, *filters.MaxPrepTime)
	}
	if filters.MaxCookTime != nil {
		timeConditions = append(timeConditions, "cook_time <= ?")
		timeValues = append(timeValues, *filters.MaxCookTime)
	}
	if filters.MaxTotalTime != nil {
		timeConditions = append(timeConditions, "total_time <= ?")
		timeValues = append(timeValues, *filters.MaxTotalTime)
	}

	if len(timeConditions) > 0 {
		query = query.Where(strings.Join(timeConditions, " AND "), timeValues...)
	}

	// Cuisine type filter - optimize with exact match when possible
	if filters.CuisineType != nil {
		query = query.Where("cuisine_type = ?", *filters.CuisineType)
	}

	// Dietary labels filter - use GIN index with @> operator
	if len(filters.DietaryLabels) > 0 {
		query = query.Where("dietary_labels @> ?", filters.DietaryLabels)
	}

	// Full-text search if implemented
	if filters.Search != nil && *filters.Search != "" {
		searchTerm := "%" + *filters.Search + "%"
		query = query.Where("title ILIKE ? OR description ILIKE ?", searchTerm, searchTerm)
	}

	return query
}

func (r *optimizedRecipeRepository) applyOptimizedSorting(query *gorm.DB, sortBy, sortOrder string) *gorm.DB {
	// Default sorting optimized for most common use cases
	orderClause := "average_rating DESC, total_ratings DESC"

	if sortBy != "" {
		direction := "DESC"
		if sortOrder == "asc" {
			direction = "ASC"
		}

		switch sortBy {
		case "created_at":
			orderClause = fmt.Sprintf("created_at %s", direction)
		case "updated_at":
			orderClause = fmt.Sprintf("updated_at %s", direction)
		case "total_time":
			orderClause = fmt.Sprintf("total_time %s, average_rating DESC", direction)
		case "average_rating":
			orderClause = fmt.Sprintf("average_rating %s, total_ratings DESC", direction)
		case "title":
			orderClause = fmt.Sprintf("title %s", direction)
		}
	}

	return query.Order(orderClause)
}

func (r *optimizedRecipeRepository) generateSearchCacheKey(userID uuid.UUID, params *models.RecipeSearchParams) string {
	// Create a deterministic cache key from search parameters
	key := fmt.Sprintf("recipe_search:%s:page_%d:limit_%d:sort_%s_%s",
		userID.String(),
		params.Page,
		params.Limit,
		params.SortBy,
		params.SortOrder,
	)

	// Add filter parameters to key
	if params.MealType != nil && *params.MealType != "" {
		key += fmt.Sprintf(":meal_%s", *params.MealType)
	}
	if params.Complexity != nil && *params.Complexity != "" {
		key += fmt.Sprintf(":complexity_%s", *params.Complexity)
	}
	if params.MaxTotalTime != nil {
		key += fmt.Sprintf(":maxtime_%d", *params.MaxTotalTime)
	}
	if params.CuisineType != nil {
		key += fmt.Sprintf(":cuisine_%s", *params.CuisineType)
	}
	if len(params.DietaryLabels) > 0 {
		key += fmt.Sprintf(":dietary_%s", strings.Join(params.DietaryLabels, "_"))
	}

	return key
}
