package services

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/models"
)

// AdvancedSearchParams provides comprehensive search parameters
type AdvancedSearchParams struct {
	// Basic search
	Query  string `json:"query"`
	UserID uuid.UUID `json:"userId"`
	
	// Filtering
	MealTypes         []string  `json:"mealTypes"`
	DietaryLabels     []string  `json:"dietaryLabels"`
	CuisineTypes      []string  `json:"cuisineTypes"`
	Complexities      []string  `json:"complexities"`
	MaxPrepTime       *int      `json:"maxPrepTime"`
	MaxCookTime       *int      `json:"maxCookTime"`
	MaxTotalTime      *int      `json:"maxTotalTime"`
	MinRating         *float64  `json:"minRating"`
	RequiredIngredients []string `json:"requiredIngredients"`
	ExcludedIngredients []string `json:"excludedIngredients"`
	
	// Nutritional filters
	MaxCalories       *int      `json:"maxCalories"`
	MinProtein        *float64  `json:"minProtein"`
	MaxCarbs          *float64  `json:"maxCarbs"`
	MaxFat            *float64  `json:"maxFat"`
	
	// Sorting and pagination
	SortBy            string    `json:"sortBy"` // relevance, rating, time, created_at, popularity
	SortOrder         string    `json:"sortOrder"` // asc, desc
	Page              int       `json:"page"`
	Limit             int       `json:"limit"`
	
	// Advanced options
	IncludeUserRecipes   bool     `json:"includeUserRecipes"`
	IncludePublicRecipes bool     `json:"includePublicRecipes"`
	OnlyFavorites        bool     `json:"onlyFavorites"`
	ExcludeRecipeIDs     []uuid.UUID `json:"excludeRecipeIds"`
	BoostRecentlyRated   bool     `json:"boostRecentlyRated"`
	
	// Cache control
	UseCache          bool      `json:"useCache"`
	CacheTTL          time.Duration `json:"cacheTtl"`
}

// SearchResult represents a search result with additional metadata
type SearchResult struct {
	Recipe          models.Recipe `json:"recipe"`
	RelevanceScore  float64       `json:"relevanceScore"`
	PopularityScore float64       `json:"popularityScore"`
	UserRating      *int          `json:"userRating,omitempty"`
	IsFavorite      bool          `json:"isFavorite"`
	TimesCooked     int           `json:"timesCooked"`
	MatchedFields   []string      `json:"matchedFields"`
}

// AdvancedSearchResponse provides comprehensive search results
type AdvancedSearchResponse struct {
	Results       []SearchResult    `json:"results"`
	Total         int64             `json:"total"`
	Page          int               `json:"page"`
	Limit         int               `json:"limit"`
	TotalPages    int               `json:"totalPages"`
	SearchTime    time.Duration     `json:"searchTime"`
	FromCache     bool              `json:"fromCache"`
	Suggestions   []SearchSuggestion `json:"suggestions,omitempty"`
	Facets        SearchFacets      `json:"facets"`
}

// SearchSuggestion provides alternative search suggestions
type SearchSuggestion struct {
	Text        string `json:"text"`
	Type        string `json:"type"` // spelling, related, popular
	ResultCount int    `json:"resultCount"`
}

// SearchFacets provides aggregated search filters
type SearchFacets struct {
	MealTypes     map[string]int `json:"mealTypes"`
	Complexities  map[string]int `json:"complexities"`
	CuisineTypes  map[string]int `json:"cuisineTypes"`
	AvgCookTime   float64        `json:"avgCookTime"`
	AvgRating     float64        `json:"avgRating"`
}

// AdvancedRecipeSearchService provides sophisticated recipe search capabilities
type AdvancedRecipeSearchService interface {
	Search(ctx context.Context, params *AdvancedSearchParams) (*AdvancedSearchResponse, error)
	SearchWithAutoComplete(ctx context.Context, query string, limit int) ([]string, error)
	GetPopularSearches(ctx context.Context, limit int) ([]string, error)
	GetSearchSuggestions(ctx context.Context, query string, limit int) ([]SearchSuggestion, error)
	WarmCache(ctx context.Context, commonSearches []AdvancedSearchParams) error
	ClearSearchCache(ctx context.Context) error
	GetSearchAnalytics(ctx context.Context, since time.Duration) (*SearchAnalytics, error)
}

// SearchAnalytics provides insights into search behavior
type SearchAnalytics struct {
	TotalSearches     int64                 `json:"totalSearches"`
	UniqueSearches    int64                 `json:"uniqueSearches"`
	AvgResultsPerPage int                   `json:"avgResultsPerPage"`
	TopQueries        []PopularQuery        `json:"topQueries"`
	NoResultQueries   []string              `json:"noResultQueries"`
	SearchTrends      map[string]int        `json:"searchTrends"`
	PerformanceStats  SearchPerformanceStats `json:"performanceStats"`
}

// PopularQuery represents a frequently searched query
type PopularQuery struct {
	Query       string  `json:"query"`
	Count       int64   `json:"count"`
	AvgResults  int     `json:"avgResults"`
	SuccessRate float64 `json:"successRate"`
}

// SearchPerformanceStats tracks search performance metrics
type SearchPerformanceStats struct {
	AvgSearchTime    time.Duration `json:"avgSearchTime"`
	CacheHitRate     float64       `json:"cacheHitRate"`
	SlowSearchCount  int64         `json:"slowSearchCount"`
	ErrorRate        float64       `json:"errorRate"`
}

type advancedRecipeSearchService struct {
	db                *gorm.DB
	cache             *CacheService
	performanceService QueryPerformanceService
	searchTTL         time.Duration
	maxCacheSize      int
}

func NewAdvancedRecipeSearchService(db *gorm.DB, cache *CacheService, performanceService QueryPerformanceService) AdvancedRecipeSearchService {
	return &advancedRecipeSearchService{
		db:                 db,
		cache:              cache,
		performanceService: performanceService,
		searchTTL:          15 * time.Minute, // Cache search results for 15 minutes
		maxCacheSize:       1000,             // Maximum cached search results
	}
}

// Search performs advanced recipe search with caching and performance optimization
func (s *advancedRecipeSearchService) Search(ctx context.Context, params *AdvancedSearchParams) (*AdvancedSearchResponse, error) {
	startTime := time.Now()
	
	// Generate cache key
	cacheKey := s.generateCacheKey(params)
	
	// Try cache first if enabled
	if params.UseCache {
		if cachedResult, err := s.getFromCache(ctx, cacheKey); err == nil {
			cachedResult.SearchTime = time.Since(startTime)
			cachedResult.FromCache = true
			return cachedResult, nil
		}
	}
	
	// Validate and normalize parameters
	if err := s.validateAndNormalizeParams(params); err != nil {
		return nil, fmt.Errorf("invalid search parameters: %w", err)
	}
	
	// Build optimized query
	query := s.buildOptimizedQuery(params)
	
	// Execute search with performance monitoring
	results, total, err := s.executeSearch(ctx, query, params)
	if err != nil {
		return nil, fmt.Errorf("search execution failed: %w", err)
	}
	
	// Enhance results with additional metadata
	enhancedResults, err := s.enhanceResults(ctx, results, params)
	if err != nil {
		log.Printf("Failed to enhance search results: %v", err)
		// Continue with basic results
	}
	
	// Calculate pagination
	totalPages := int(total) / params.Limit
	if int(total)%params.Limit > 0 {
		totalPages++
	}
	
	// Generate search facets
	facets, err := s.generateSearchFacets(ctx, params)
	if err != nil {
		log.Printf("Failed to generate search facets: %v", err)
		facets = SearchFacets{}
	}
	
	// Generate suggestions for better search results
	suggestions, err := s.generateSearchSuggestions(ctx, params)
	if err != nil {
		log.Printf("Failed to generate search suggestions: %v", err)
	}
	
	response := &AdvancedSearchResponse{
		Results:     enhancedResults,
		Total:       total,
		Page:        params.Page,
		Limit:       params.Limit,
		TotalPages:  totalPages,
		SearchTime:  time.Since(startTime),
		FromCache:   false,
		Suggestions: suggestions,
		Facets:      facets,
	}
	
	// Cache the result if enabled
	if params.UseCache && len(enhancedResults) > 0 {
		cacheTTL := params.CacheTTL
		if cacheTTL == 0 {
			cacheTTL = s.searchTTL
		}
		
		go func() {
			if err := s.cacheResult(context.Background(), cacheKey, response, cacheTTL); err != nil {
				log.Printf("Failed to cache search result: %v", err)
			}
		}()
	}
	
	return response, nil
}

func (s *advancedRecipeSearchService) generateCacheKey(params *AdvancedSearchParams) string {
	// Create a deterministic cache key from search parameters
	data, _ := json.Marshal(params)
	hash := sha256.Sum256(data)
	return fmt.Sprintf("recipe_search:%s", hex.EncodeToString(hash[:8]))
}

func (s *advancedRecipeSearchService) validateAndNormalizeParams(params *AdvancedSearchParams) error {
	// Set defaults
	if params.Page < 1 {
		params.Page = 1
	}
	if params.Limit < 1 || params.Limit > 100 {
		params.Limit = 20
	}
	if params.SortBy == "" {
		params.SortBy = "relevance"
	}
	if params.SortOrder == "" {
		params.SortOrder = "desc"
	}
	
	// Normalize text query
	params.Query = strings.TrimSpace(params.Query)
	
	// Validate sort fields
	validSortFields := map[string]bool{
		"relevance": true, "rating": true, "time": true, 
		"created_at": true, "popularity": true, "total_time": true,
	}
	if !validSortFields[params.SortBy] {
		return fmt.Errorf("invalid sort field: %s", params.SortBy)
	}
	
	return nil
}

func (s *advancedRecipeSearchService) buildOptimizedQuery(params *AdvancedSearchParams) *gorm.DB {
	query := s.db.Model(&models.Recipe{}).Where("deleted_at IS NULL")
	
	// User access control
	if params.IncludeUserRecipes && params.IncludePublicRecipes {
		query = query.Where("(user_id = ? OR is_public = true)", params.UserID)
	} else if params.IncludeUserRecipes {
		query = query.Where("user_id = ?", params.UserID)
	} else if params.IncludePublicRecipes {
		query = query.Where("is_public = true")
	}
	
	// Text search with ranking
	if params.Query != "" {
		// Use PostgreSQL full-text search for better performance
		query = query.Where(
			"to_tsvector('english', title || ' ' || COALESCE(description, '')) @@ plainto_tsquery('english', ?)",
			params.Query,
		).Select(
			"*, ts_rank(to_tsvector('english', title || ' ' || COALESCE(description, '')), plainto_tsquery('english', ?)) as relevance_score",
			params.Query,
		)
	} else {
		query = query.Select("*, 1.0 as relevance_score")
	}
	
	// Apply filters
	query = s.applyAdvancedFilters(query, params)
	
	// Apply sorting
	query = s.applySorting(query, params)
	
	return query
}

func (s *advancedRecipeSearchService) applyAdvancedFilters(query *gorm.DB, params *AdvancedSearchParams) *gorm.DB {
	// Meal types
	if len(params.MealTypes) > 0 {
		query = query.Where("meal_type && ?", params.MealTypes)
	}
	
	// Dietary labels
	if len(params.DietaryLabels) > 0 {
		query = query.Where("dietary_labels && ?", params.DietaryLabels)
	}
	
	// Cuisine types
	if len(params.CuisineTypes) > 0 {
		query = query.Where("cuisine_type = ANY(?)", params.CuisineTypes)
	}
	
	// Complexities
	if len(params.Complexities) > 0 {
		query = query.Where("complexity = ANY(?)", params.Complexities)
	}
	
	// Time filters
	if params.MaxPrepTime != nil {
		query = query.Where("prep_time <= ?", *params.MaxPrepTime)
	}
	if params.MaxCookTime != nil {
		query = query.Where("cook_time <= ?", *params.MaxCookTime)
	}
	if params.MaxTotalTime != nil {
		query = query.Where("total_time <= ?", *params.MaxTotalTime)
	}
	
	// Rating filter
	if params.MinRating != nil {
		query = query.Where("average_rating >= ?", *params.MinRating)
	}
	
	// Ingredient filters
	if len(params.RequiredIngredients) > 0 {
		for _, ingredient := range params.RequiredIngredients {
			query = query.Where("ingredients::text ILIKE ?", "%"+ingredient+"%")
		}
	}
	
	if len(params.ExcludedIngredients) > 0 {
		for _, ingredient := range params.ExcludedIngredients {
			query = query.Where("NOT (ingredients::text ILIKE ?)", "%"+ingredient+"%")
		}
	}
	
	// Nutritional filters
	if params.MaxCalories != nil {
		query = query.Where("(nutrition->>'calories')::int <= ?", *params.MaxCalories)
	}
	if params.MinProtein != nil {
		query = query.Where("(nutrition->>'protein')::float >= ?", *params.MinProtein)
	}
	if params.MaxCarbs != nil {
		query = query.Where("(nutrition->>'carbs')::float <= ?", *params.MaxCarbs)
	}
	if params.MaxFat != nil {
		query = query.Where("(nutrition->>'fat')::float <= ?", *params.MaxFat)
	}
	
	// Exclude specific recipes
	if len(params.ExcludeRecipeIDs) > 0 {
		query = query.Where("id NOT IN (?)", params.ExcludeRecipeIDs)
	}
	
	return query
}

func (s *advancedRecipeSearchService) applySorting(query *gorm.DB, params *AdvancedSearchParams) *gorm.DB {
	switch params.SortBy {
	case "relevance":
		if params.Query != "" {
			query = query.Order("relevance_score DESC, average_rating DESC")
		} else {
			query = query.Order("average_rating DESC, total_ratings DESC")
		}
	case "rating":
		query = query.Order("average_rating DESC, total_ratings DESC")
	case "time":
		query = query.Order("total_time ASC")
	case "total_time":
		if params.SortOrder == "asc" {
			query = query.Order("total_time ASC")
		} else {
			query = query.Order("total_time DESC")
		}
	case "created_at":
		if params.SortOrder == "asc" {
			query = query.Order("created_at ASC")
		} else {
			query = query.Order("created_at DESC")
		}
	case "popularity":
		query = query.Order("total_ratings DESC, average_rating DESC")
	default:
		query = query.Order("created_at DESC")
	}
	
	return query
}

func (s *advancedRecipeSearchService) executeSearch(ctx context.Context, query *gorm.DB, params *AdvancedSearchParams) ([]models.Recipe, int64, error) {
	var total int64
	var recipes []models.Recipe
	
	// Count total results
	countQuery := query.Session(&gorm.Session{})
	if err := countQuery.Count(&total).Error; err != nil {
		return nil, 0, fmt.Errorf("failed to count results: %w", err)
	}
	
	// Apply pagination and execute
	offset := (params.Page - 1) * params.Limit
	if err := query.Offset(offset).Limit(params.Limit).Find(&recipes).Error; err != nil {
		return nil, 0, fmt.Errorf("failed to execute search: %w", err)
	}
	
	return recipes, total, nil
}

func (s *advancedRecipeSearchService) enhanceResults(ctx context.Context, recipes []models.Recipe, params *AdvancedSearchParams) ([]SearchResult, error) {
	results := make([]SearchResult, len(recipes))
	
	for i, recipe := range recipes {
		result := SearchResult{
			Recipe:         recipe,
			RelevanceScore: 1.0, // Would be calculated from query if available
			PopularityScore: s.calculatePopularityScore(recipe),
			MatchedFields:  s.getMatchedFields(recipe, params.Query),
		}
		
		// Get user-specific data if user is authenticated
		if params.UserID != uuid.Nil {
			// Check if user has rated this recipe
			var rating models.RecipeRating
			if err := s.db.Where("recipe_id = ? AND user_id = ?", recipe.ID, params.UserID).First(&rating).Error; err == nil {
				ratingValue := rating.Rating
			result.UserRating = &ratingValue
			}
			
			// Check if recipe is favorited (would need favorites table)
			// result.IsFavorite = s.isFavorite(recipe.ID, params.UserID)
			
			// Get times cooked by user (would track in meal plan history)
			// result.TimesCooked = s.getTimesCooked(recipe.ID, params.UserID)
		}
		
		results[i] = result
	}
	
	return results, nil
}

func (s *advancedRecipeSearchService) calculatePopularityScore(recipe models.Recipe) float64 {
	// Simple popularity calculation based on ratings
	if recipe.TotalRatings == 0 {
		return 0.0
	}
	
	// Weighted score considering both rating and number of ratings
	return float64(recipe.AverageRating) * (1.0 + float64(recipe.TotalRatings)/100.0)
}

func (s *advancedRecipeSearchService) getMatchedFields(recipe models.Recipe, query string) []string {
	if query == "" {
		return []string{}
	}
	
	var matched []string
	query = strings.ToLower(query)
	
	if strings.Contains(strings.ToLower(recipe.Title), query) {
		matched = append(matched, "title")
	}
	if recipe.Description != nil && strings.Contains(strings.ToLower(*recipe.Description), query) {
		matched = append(matched, "description")
	}
	
	return matched
}

func (s *advancedRecipeSearchService) generateSearchFacets(ctx context.Context, params *AdvancedSearchParams) (SearchFacets, error) {
	facets := SearchFacets{
		MealTypes:    make(map[string]int),
		Complexities: make(map[string]int),
		CuisineTypes: make(map[string]int),
	}
	
	// Build base query without pagination and specific filters for facet counting
	baseQuery := s.db.Model(&models.Recipe{}).Where("deleted_at IS NULL")
	
	// Apply only user access filters for facets
	if params.IncludeUserRecipes && params.IncludePublicRecipes {
		baseQuery = baseQuery.Where("(user_id = ? OR is_public = true)", params.UserID)
	} else if params.IncludeUserRecipes {
		baseQuery = baseQuery.Where("user_id = ?", params.UserID)
	} else if params.IncludePublicRecipes {
		baseQuery = baseQuery.Where("is_public = true")
	}
	
	// Get complexity distribution
	var complexityResults []struct {
		Complexity string `json:"complexity"`
		Count      int    `json:"count"`
	}
	
	if err := baseQuery.Select("complexity, COUNT(*) as count").
		Group("complexity").
		Scan(&complexityResults).Error; err == nil {
		for _, result := range complexityResults {
			facets.Complexities[result.Complexity] = result.Count
		}
	}
	
	// Get average cooking time and rating
	var avgStats struct {
		AvgCookTime float64 `json:"avg_cook_time"`
		AvgRating   float64 `json:"avg_rating"`
	}
	
	if err := baseQuery.Select("AVG(total_time) as avg_cook_time, AVG(average_rating) as avg_rating").
		Scan(&avgStats).Error; err == nil {
		facets.AvgCookTime = avgStats.AvgCookTime
		facets.AvgRating = avgStats.AvgRating
	}
	
	return facets, nil
}

func (s *advancedRecipeSearchService) generateSearchSuggestions(ctx context.Context, params *AdvancedSearchParams) ([]SearchSuggestion, error) {
	var suggestions []SearchSuggestion
	
	// If no results or few results, suggest alternatives
	if params.Query != "" {
		// Simple spelling suggestion (in real implementation, use a spell-check library)
		if strings.Contains(params.Query, "chiken") {
			suggestions = append(suggestions, SearchSuggestion{
				Text: strings.Replace(params.Query, "chiken", "chicken", -1),
				Type: "spelling",
				ResultCount: 0, // Would be calculated
			})
		}
		
		// Suggest broader searches
		suggestions = append(suggestions, SearchSuggestion{
			Text: "popular " + params.Query + " recipes",
			Type: "related",
			ResultCount: 0,
		})
	}
	
	return suggestions, nil
}

func (s *advancedRecipeSearchService) getFromCache(ctx context.Context, cacheKey string) (*AdvancedSearchResponse, error) {
	cached, err := s.cache.Get(ctx, cacheKey)
	if err != nil {
		return nil, err
	}
	
	var result AdvancedSearchResponse
	if err := json.Unmarshal([]byte(cached), &result); err != nil {
		return nil, err
	}
	
	return &result, nil
}

func (s *advancedRecipeSearchService) cacheResult(ctx context.Context, cacheKey string, result *AdvancedSearchResponse, ttl time.Duration) error {
	return s.cache.Set(ctx, cacheKey, result, ttl)
}

// SearchWithAutoComplete provides auto-complete suggestions
func (s *advancedRecipeSearchService) SearchWithAutoComplete(ctx context.Context, query string, limit int) ([]string, error) {
	if query == "" || len(query) < 2 {
		return []string{}, nil
	}
	
	var suggestions []string
	
	// Search recipe titles
	var recipes []models.Recipe
	if err := s.db.Model(&models.Recipe{}).
		Where("deleted_at IS NULL AND title ILIKE ?", query+"%").
		Limit(limit).
		Find(&recipes).Error; err == nil {
		
		for _, recipe := range recipes {
			suggestions = append(suggestions, recipe.Title)
		}
	}
	
	return suggestions, nil
}

// GetPopularSearches returns frequently searched terms
func (s *advancedRecipeSearchService) GetPopularSearches(ctx context.Context, limit int) ([]string, error) {
	// This would typically come from search analytics
	// For now, return static popular searches
	popular := []string{
		"chicken recipes",
		"vegetarian meals",
		"quick dinner",
		"healthy breakfast",
		"pasta dishes",
		"desserts",
		"salad recipes",
		"soup recipes",
	}
	
	if limit > 0 && limit < len(popular) {
		popular = popular[:limit]
	}
	
	return popular, nil
}

// Placeholder implementations for other interface methods
func (s *advancedRecipeSearchService) GetSearchSuggestions(ctx context.Context, query string, limit int) ([]SearchSuggestion, error) {
	return []SearchSuggestion{}, nil
}

func (s *advancedRecipeSearchService) WarmCache(ctx context.Context, commonSearches []AdvancedSearchParams) error {
	return nil
}

func (s *advancedRecipeSearchService) ClearSearchCache(ctx context.Context) error {
	return nil
}

func (s *advancedRecipeSearchService) GetSearchAnalytics(ctx context.Context, since time.Duration) (*SearchAnalytics, error) {
	return &SearchAnalytics{}, nil
}