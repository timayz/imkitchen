package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
)

type EnhancedRecipeCacheService interface {
	CacheRecipeSearchResults(ctx context.Context, searchParams *models.RecipeSearchParams, userID uuid.UUID, recipes []models.Recipe, totalCount int64) error
	GetRecipeSearchResults(ctx context.Context, searchParams *models.RecipeSearchParams, userID uuid.UUID) (*CachedSearchResult, error)
	InvalidateRecipeSearches(ctx context.Context, recipeID uuid.UUID) error
	InvalidateUserSearches(ctx context.Context, userID uuid.UUID) error
	WarmPopularSearches(ctx context.Context) error
	GetSearchCacheMetrics(ctx context.Context) (*RecipeSearchCacheMetrics, error)
	PurgeExpiredEntries(ctx context.Context) error
	OptimizeCacheTTL(ctx context.Context, searchKey string, accessPattern AccessPattern) time.Duration
}

type CachedSearchResult struct {
	Recipes    []models.Recipe `json:"recipes"`
	TotalCount int64          `json:"total_count"`
	CachedAt   time.Time      `json:"cached_at"`
	TTL        time.Duration  `json:"ttl"`
	SearchKey  string         `json:"search_key"`
}

type RecipeSearchCacheMetrics struct {
	HitRate              float64           `json:"hit_rate"`
	MissRate             float64           `json:"miss_rate"`
	TotalRequests        int64            `json:"total_requests"`
	CacheSize            int64            `json:"cache_size"`
	AverageTTL           time.Duration    `json:"average_ttl"`
	PopularSearchTerms   []string         `json:"popular_search_terms"`
	CacheEfficiency      float64          `json:"cache_efficiency"`
	InvalidationCount    int64            `json:"invalidation_count"`
	TTLOptimizationGains map[string]float64 `json:"ttl_optimization_gains"`
}

type AccessPattern struct {
	Frequency    int           `json:"frequency"`
	LastAccessed time.Time     `json:"last_accessed"`
	AccessTimes  []time.Time   `json:"access_times"`
	QueryType    string        `json:"query_type"`
	ResultSize   int          `json:"result_size"`
	UserPattern  string       `json:"user_pattern"` // "frequent", "occasional", "new"
}

type enhancedRecipeCacheService struct {
	cacheService      CacheService
	queryCacheService QueryCacheService
	popularSearches   map[string]int
	accessPatterns    map[string]*AccessPattern
	metrics          *RecipeSearchCacheMetrics
}

func NewEnhancedRecipeCacheService(cacheService CacheService, queryCacheService QueryCacheService) EnhancedRecipeCacheService {
	return &enhancedRecipeCacheService{
		cacheService:      cacheService,
		queryCacheService: queryCacheService,
		popularSearches:   make(map[string]int),
		accessPatterns:    make(map[string]*AccessPattern),
		metrics: &RecipeSearchCacheMetrics{
			TTLOptimizationGains: make(map[string]float64),
		},
	}
}

func (e *enhancedRecipeCacheService) CacheRecipeSearchResults(ctx context.Context, searchParams *models.RecipeSearchParams, userID uuid.UUID, recipes []models.Recipe, totalCount int64) error {
	searchKey := e.generateSearchKey(searchParams, userID)
	
	// Determine optimal TTL based on search characteristics
	accessPattern := e.getOrCreateAccessPattern(searchKey)
	ttl := e.OptimizeCacheTTL(ctx, searchKey, *accessPattern)
	
	// Create cached result
	cachedResult := &CachedSearchResult{
		Recipes:    recipes,
		TotalCount: totalCount,
		CachedAt:   time.Now(),
		TTL:        ttl,
		SearchKey:  searchKey,
	}
	
	// Cache the result using the underlying query cache service
	err := e.queryCacheService.Set(ctx, searchKey, cachedResult, ttl)
	if err != nil {
		log.Printf("Failed to cache recipe search results for key %s: %v", searchKey, err)
		return fmt.Errorf("failed to cache recipe search results: %w", err)
	}
	
	// Update popularity tracking
	e.trackSearchPopularity(searchKey, searchParams)
	
	// Update access patterns
	e.updateAccessPattern(searchKey, "search", len(recipes))
	
	log.Printf("Cached recipe search results for key %s with TTL %v", searchKey, ttl)
	return nil
}

func (e *enhancedRecipeCacheService) GetRecipeSearchResults(ctx context.Context, searchParams *models.RecipeSearchParams, userID uuid.UUID) (*CachedSearchResult, error) {
	searchKey := e.generateSearchKey(searchParams, userID)
	
	// Try to get from cache
	cached, err := e.queryCacheService.Get(ctx, searchKey)
	if err != nil {
		// Cache miss
		e.updateMetrics("miss")
		return nil, err
	}
	
	// Parse cached result
	var cachedResult CachedSearchResult
	if err := json.Unmarshal([]byte(cached), &cachedResult); err != nil {
		log.Printf("Failed to unmarshal cached result for key %s: %v", searchKey, err)
		e.updateMetrics("miss")
		return nil, err
	}
	
	// Update access patterns and metrics
	e.updateAccessPattern(searchKey, "hit", len(cachedResult.Recipes))
	e.updateMetrics("hit")
	
	log.Printf("Cache hit for recipe search key: %s", searchKey)
	return &cachedResult, nil
}

func (e *enhancedRecipeCacheService) InvalidateRecipeSearches(ctx context.Context, recipeID uuid.UUID) error {
	// Pattern-based invalidation for recipe updates
	// This would invalidate all searches that might contain this recipe
	invalidationPatterns := []string{
		"recipe_search:*:all",
		fmt.Sprintf("recipe_search:*:recipe_%s", recipeID.String()),
		"recipe_search:*:trending",
		"recipe_search:*:popular",
	}
	
	var invalidationErrors []error
	invalidatedCount := 0
	
	for _, pattern := range invalidationPatterns {
		keys, err := e.findKeysMatchingPattern(ctx, pattern)
		if err != nil {
			log.Printf("Failed to find keys for pattern %s: %v", pattern, err)
			continue
		}
		
		for _, key := range keys {
			if err := e.queryCacheService.Invalidate(ctx, key, "recipe_update"); err != nil {
				invalidationErrors = append(invalidationErrors, err)
				log.Printf("Failed to invalidate key %s: %v", key, err)
			} else {
				invalidatedCount++
				log.Printf("Invalidated cache key: %s", key)
			}
		}
	}
	
	// Update invalidation metrics
	e.metrics.InvalidationCount += int64(invalidatedCount)
	
	if len(invalidationErrors) > 0 {
		return fmt.Errorf("partial invalidation failure: %d errors occurred", len(invalidationErrors))
	}
	
	log.Printf("Successfully invalidated %d cache keys for recipe %s", invalidatedCount, recipeID.String())
	return nil
}

func (e *enhancedRecipeCacheService) InvalidateUserSearches(ctx context.Context, userID uuid.UUID) error {
	// Invalidate all searches for a specific user
	userPattern := fmt.Sprintf("recipe_search:%s:*", userID.String())
	
	keys, err := e.findKeysMatchingPattern(ctx, userPattern)
	if err != nil {
		return fmt.Errorf("failed to find user search keys: %w", err)
	}
	
	invalidatedCount := 0
	for _, key := range keys {
		if err := e.queryCacheService.Invalidate(ctx, key, "user_preference_change"); err != nil {
			log.Printf("Failed to invalidate user key %s: %v", key, err)
		} else {
			invalidatedCount++
		}
	}
	
	log.Printf("Invalidated %d user search cache keys for user %s", invalidatedCount, userID.String())
	return nil
}

func (e *enhancedRecipeCacheService) WarmPopularSearches(ctx context.Context) error {
	// Get popular search terms from tracking
	popularSearches := e.getTopPopularSearches(10)
	
	warmedCount := 0
	for _, searchTerm := range popularSearches {
		// Parse search key to recreate search parameters
		searchParams, userID, err := e.parseSearchKey(searchTerm)
		if err != nil {
			log.Printf("Failed to parse search key for warming %s: %v", searchTerm, err)
			continue
		}
		
		// Check if already cached and not expired
		if _, err := e.GetRecipeSearchResults(ctx, searchParams, userID); err == nil {
			continue // Already cached
		}
		
		// Use cache warming from QueryCacheService for popular searches
		err = e.queryCacheService.WarmCache(ctx, searchTerm, time.Hour*4)
		if err != nil {
			log.Printf("Failed to warm cache for search %s: %v", searchTerm, err)
			continue
		}
		
		warmedCount++
		log.Printf("Warmed cache for popular search: %s", searchTerm)
	}
	
	log.Printf("Successfully warmed %d popular search caches", warmedCount)
	return nil
}

func (e *enhancedRecipeCacheService) GetSearchCacheMetrics(ctx context.Context) (*RecipeSearchCacheMetrics, error) {
	// Get base metrics from QueryCacheService
	baseMetrics, err := e.queryCacheService.GetMetrics(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get base cache metrics: %w", err)
	}
	
	// Calculate recipe-specific metrics
	totalRequests := baseMetrics.Hits + baseMetrics.Misses
	hitRate := 0.0
	if totalRequests > 0 {
		hitRate = float64(baseMetrics.Hits) / float64(totalRequests)
	}
	
	// Update our internal metrics
	e.metrics.HitRate = hitRate
	e.metrics.MissRate = 1.0 - hitRate
	e.metrics.TotalRequests = totalRequests
	e.metrics.CacheSize = baseMetrics.CacheSize
	e.metrics.AverageTTL = baseMetrics.AverageTTL
	e.metrics.PopularSearchTerms = e.getTopPopularSearches(10)
	e.metrics.CacheEfficiency = e.calculateCacheEfficiency()
	
	return e.metrics, nil
}

func (e *enhancedRecipeCacheService) PurgeExpiredEntries(ctx context.Context) error {
	// Use QueryCacheService purge functionality
	return e.queryCacheService.PurgeExpired(ctx)
}

func (e *enhancedRecipeCacheService) OptimizeCacheTTL(ctx context.Context, searchKey string, accessPattern AccessPattern) time.Duration {
	// Base TTL values for different query types
	baseTTLs := map[string]time.Duration{
		"trending":    time.Minute * 30, // Trending searches change frequently
		"popular":     time.Hour * 2,    // Popular searches more stable
		"filtered":    time.Hour * 1,    // Filtered searches moderate stability
		"text_search": time.Hour * 6,    // Text searches very stable
		"user_pref":   time.Hour * 4,    // User preference searches fairly stable
	}
	
	baseTTL, exists := baseTTLs[accessPattern.QueryType]
	if !exists {
		baseTTL = time.Hour // Default TTL
	}
	
	// Adjust TTL based on access frequency
	frequencyMultiplier := 1.0
	if accessPattern.Frequency > 10 {
		frequencyMultiplier = 2.0 // High frequency - cache longer
	} else if accessPattern.Frequency > 5 {
		frequencyMultiplier = 1.5 // Medium frequency
	} else if accessPattern.Frequency < 2 {
		frequencyMultiplier = 0.5 // Low frequency - cache shorter
	}
	
	// Adjust based on result size (larger results cached longer)
	sizeMultiplier := 1.0
	if accessPattern.ResultSize > 50 {
		sizeMultiplier = 1.3
	} else if accessPattern.ResultSize < 10 {
		sizeMultiplier = 0.8
	}
	
	// Calculate optimized TTL
	optimizedTTL := time.Duration(float64(baseTTL) * frequencyMultiplier * sizeMultiplier)
	
	// Ensure TTL bounds
	minTTL := time.Minute * 10
	maxTTL := time.Hour * 24
	
	if optimizedTTL < minTTL {
		optimizedTTL = minTTL
	} else if optimizedTTL > maxTTL {
		optimizedTTL = maxTTL
	}
	
	// Track optimization gains
	originalTTL := baseTTL
	gain := (float64(optimizedTTL) - float64(originalTTL)) / float64(originalTTL)
	e.metrics.TTLOptimizationGains[searchKey] = gain
	
	return optimizedTTL
}

// Helper methods

func (e *enhancedRecipeCacheService) generateSearchKey(searchParams *models.RecipeSearchParams, userID uuid.UUID) string {
	// Create a deterministic search key based on parameters
	keyParts := []string{
		"recipe_search",
		userID.String(),
	}
	
	if searchParams.Query != "" {
		keyParts = append(keyParts, "q:"+searchParams.Query)
	}
	if searchParams.Cuisine != "" {
		keyParts = append(keyParts, "cuisine:"+searchParams.Cuisine)
	}
	if len(searchParams.DietaryRestrictions) > 0 {
		restrictions := make([]string, len(searchParams.DietaryRestrictions))
		copy(restrictions, searchParams.DietaryRestrictions)
		sort.Strings(restrictions)
		keyParts = append(keyParts, "diet:"+strings.Join(restrictions, ","))
	}
	if searchParams.MaxPrepTime > 0 {
		keyParts = append(keyParts, fmt.Sprintf("preptime:%d", searchParams.MaxPrepTime))
	}
	if searchParams.MinRating > 0 {
		keyParts = append(keyParts, fmt.Sprintf("rating:%.1f", searchParams.MinRating))
	}
	if searchParams.SortBy != "" {
		keyParts = append(keyParts, "sort:"+searchParams.SortBy)
	}
	if searchParams.Limit > 0 {
		keyParts = append(keyParts, fmt.Sprintf("limit:%d", searchParams.Limit))
	}
	if searchParams.Offset > 0 {
		keyParts = append(keyParts, fmt.Sprintf("offset:%d", searchParams.Offset))
	}
	
	return strings.Join(keyParts, ":")
}

func (e *enhancedRecipeCacheService) trackSearchPopularity(searchKey string, searchParams *models.RecipeSearchParams) {
	// Extract search term for popularity tracking
	searchTerm := searchParams.Query
	if searchTerm == "" {
		searchTerm = fmt.Sprintf("cuisine:%s", searchParams.Cuisine)
	}
	
	e.popularSearches[searchTerm]++
}

func (e *enhancedRecipeCacheService) getOrCreateAccessPattern(searchKey string) *AccessPattern {
	if pattern, exists := e.accessPatterns[searchKey]; exists {
		return pattern
	}
	
	// Create new access pattern
	pattern := &AccessPattern{
		Frequency:    0,
		LastAccessed: time.Now(),
		AccessTimes:  []time.Time{},
		QueryType:    e.determineQueryType(searchKey),
		ResultSize:   0,
		UserPattern:  "new",
	}
	
	e.accessPatterns[searchKey] = pattern
	return pattern
}

func (e *enhancedRecipeCacheService) updateAccessPattern(searchKey, accessType string, resultSize int) {
	pattern := e.getOrCreateAccessPattern(searchKey)
	
	pattern.Frequency++
	pattern.LastAccessed = time.Now()
	pattern.AccessTimes = append(pattern.AccessTimes, time.Now())
	pattern.ResultSize = resultSize
	
	// Determine user pattern based on frequency
	if pattern.Frequency > 10 {
		pattern.UserPattern = "frequent"
	} else if pattern.Frequency > 3 {
		pattern.UserPattern = "occasional"
	}
	
	// Keep only recent access times (last 24 hours)
	cutoff := time.Now().Add(-24 * time.Hour)
	recentTimes := []time.Time{}
	for _, t := range pattern.AccessTimes {
		if t.After(cutoff) {
			recentTimes = append(recentTimes, t)
		}
	}
	pattern.AccessTimes = recentTimes
}

func (e *enhancedRecipeCacheService) determineQueryType(searchKey string) string {
	if strings.Contains(searchKey, "sort:rating") || strings.Contains(searchKey, "sort:trending") {
		return "trending"
	}
	if strings.Contains(searchKey, "q:") {
		return "text_search"
	}
	if strings.Contains(searchKey, "diet:") {
		return "filtered"
	}
	if strings.Contains(searchKey, "cuisine:") {
		return "filtered"
	}
	return "user_pref"
}

func (e *enhancedRecipeCacheService) getTopPopularSearches(limit int) []string {
	type searchCount struct {
		term  string
		count int
	}
	
	searches := make([]searchCount, 0, len(e.popularSearches))
	for term, count := range e.popularSearches {
		searches = append(searches, searchCount{term, count})
	}
	
	sort.Slice(searches, func(i, j int) bool {
		return searches[i].count > searches[j].count
	})
	
	result := make([]string, 0, limit)
	for i, search := range searches {
		if i >= limit {
			break
		}
		result = append(result, search.term)
	}
	
	return result
}

func (e *enhancedRecipeCacheService) calculateCacheEfficiency() float64 {
	// Calculate efficiency based on hit rate and resource usage
	if e.metrics.TotalRequests == 0 {
		return 0.0
	}
	
	// Base efficiency on hit rate
	efficiency := e.metrics.HitRate * 100
	
	// Adjust for TTL optimization gains
	totalGains := 0.0
	for _, gain := range e.metrics.TTLOptimizationGains {
		totalGains += gain
	}
	avgGain := totalGains / float64(len(e.metrics.TTLOptimizationGains))
	
	// Bonus for optimization gains
	efficiency += avgGain * 10
	
	if efficiency > 100 {
		efficiency = 100
	}
	
	return efficiency
}

func (e *enhancedRecipeCacheService) updateMetrics(eventType string) {
	switch eventType {
	case "hit":
		// Metrics updated by QueryCacheService
	case "miss":
		// Metrics updated by QueryCacheService
	}
}

func (e *enhancedRecipeCacheService) findKeysMatchingPattern(ctx context.Context, pattern string) ([]string, error) {
	// This would typically use Redis SCAN with pattern matching
	// For now, return empty slice as placeholder
	return []string{}, nil
}

func (e *enhancedRecipeCacheService) parseSearchKey(searchKey string) (*models.RecipeSearchParams, uuid.UUID, error) {
	// Parse search key back to parameters
	// This is a placeholder implementation
	parts := strings.Split(searchKey, ":")
	if len(parts) < 2 {
		return nil, uuid.Nil, fmt.Errorf("invalid search key format")
	}
	
	userID, err := uuid.Parse(parts[1])
	if err != nil {
		return nil, uuid.Nil, fmt.Errorf("invalid user ID in search key: %w", err)
	}
	
	// Create basic search params (simplified for warming)
	searchParams := &models.RecipeSearchParams{}
	
	return searchParams, userID, nil
}