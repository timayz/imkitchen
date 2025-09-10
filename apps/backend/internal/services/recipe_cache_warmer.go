package services

import (
	"context"
	"fmt"
	"log"
	"sort"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

type RecipeCacheWarmer interface {
	WarmPopularSearches(ctx context.Context) error
	WarmUserFavoriteSearches(ctx context.Context, userID uuid.UUID) error
	WarmTrendingRecipes(ctx context.Context) error
	WarmCuisineSearches(ctx context.Context) error
	WarmDietaryFilterSearches(ctx context.Context) error
	ScheduleWarmingJobs(ctx context.Context) error
	GetWarmingMetrics(ctx context.Context) (*CacheWarmingMetrics, error)
}

type CacheWarmingMetrics struct {
	TotalWarmingJobs     int64         `json:"total_warming_jobs"`
	SuccessfulWarmings   int64         `json:"successful_warmings"`
	FailedWarmings       int64         `json:"failed_warmings"`
	AverageWarmingTime   time.Duration `json:"average_warming_time"`
	LastWarmingTime      time.Time     `json:"last_warming_time"`
	PopularSearchesCount int          `json:"popular_searches_count"`
	CacheHitImprovement  float64      `json:"cache_hit_improvement"`
	WarmingEfficiency    float64      `json:"warming_efficiency"`
}

type WarmingJob struct {
	SearchKey     string                     `json:"search_key"`
	SearchParams  *models.RecipeSearchParams `json:"search_params"`
	UserID        uuid.UUID                  `json:"user_id"`
	Priority      int                       `json:"priority"` // 1-10, higher = more priority
	EstimatedTTL  time.Duration             `json:"estimated_ttl"`
	JobType       string                    `json:"job_type"` // "popular", "trending", "user_favorite", etc.
}

type recipeCacheWarmer struct {
	cacheService         EnhancedRecipeCacheService
	recipeRepository     repositories.RecipeRepository
	userRepository       repositories.UserRepository
	searchAnalytics      WarmerSearchAnalytics
	warmingQueue         chan *WarmingJob
	metrics             *CacheWarmingMetrics
	popularSearchTerms   []string
	trendingRecipes      []uuid.UUID
	cuisinePreferences   map[string]int
	dietaryPreferences   map[string]int
}

type WarmerSearchAnalytics interface {
	GetPopularSearchTerms(ctx context.Context, limit int, timeframe time.Duration) ([]PopularSearch, error)
	GetUserSearchHistory(ctx context.Context, userID uuid.UUID, limit int) ([]models.RecipeSearchParams, error)
	GetTrendingRecipes(ctx context.Context, limit int) ([]uuid.UUID, error)
	GetPopularCuisines(ctx context.Context) (map[string]int, error)
	GetPopularDietaryFilters(ctx context.Context) (map[string]int, error)
}

type PopularSearch struct {
	SearchParams *models.RecipeSearchParams `json:"search_params"`
	Frequency    int                       `json:"frequency"`
	LastUsed     time.Time                 `json:"last_used"`
	UserCount    int                       `json:"user_count"`
}

func NewRecipeCacheWarmer(
	cacheService EnhancedRecipeCacheService,
	recipeRepo repositories.RecipeRepository,
	userRepo repositories.UserRepository,
	searchAnalytics WarmerSearchAnalytics,
) RecipeCacheWarmer {
	warmer := &recipeCacheWarmer{
		cacheService:       cacheService,
		recipeRepository:   recipeRepo,
		userRepository:     userRepo,
		searchAnalytics:    searchAnalytics,
		warmingQueue:       make(chan *WarmingJob, 100),
		metrics:           &CacheWarmingMetrics{},
		cuisinePreferences: make(map[string]int),
		dietaryPreferences: make(map[string]int),
	}
	
	// Start warming worker
	go warmer.processWarmingQueue()
	
	return warmer
}

func (w *recipeCacheWarmer) WarmPopularSearches(ctx context.Context) error {
	log.Printf("Starting popular searches cache warming...")
	
	// Get popular search terms from analytics
	popularSearches, err := w.searchAnalytics.GetPopularSearchTerms(ctx, 50, 24*time.Hour)
	if err != nil {
		return fmt.Errorf("failed to get popular search terms: %w", err)
	}
	
	warmingJobs := make([]*WarmingJob, 0, len(popularSearches))
	
	for _, popular := range popularSearches {
		// Create warming job for each popular search
		job := &WarmingJob{
			SearchParams: popular.SearchParams,
			UserID:       uuid.New(), // Use system user for popular searches
			Priority:     calculatePriority(popular.Frequency, popular.UserCount),
			EstimatedTTL: time.Hour * 4, // Popular searches cached longer
			JobType:      "popular",
		}
		
		job.SearchKey = w.generateSearchKey(popular.SearchParams, job.UserID)
		warmingJobs = append(warmingJobs, job)
	}
	
	// Sort by priority (highest first)
	sort.Slice(warmingJobs, func(i, j int) bool {
		return warmingJobs[i].Priority > warmingJobs[j].Priority
	})
	
	// Queue warming jobs
	for _, job := range warmingJobs {
		select {
		case w.warmingQueue <- job:
			log.Printf("Queued popular search warming job: %s (priority: %d)", job.SearchKey, job.Priority)
		default:
			log.Printf("Warming queue full, skipping job: %s", job.SearchKey)
		}
	}
	
	log.Printf("Queued %d popular search warming jobs", len(warmingJobs))
	return nil
}

func (w *recipeCacheWarmer) WarmUserFavoriteSearches(ctx context.Context, userID uuid.UUID) error {
	log.Printf("Warming user favorite searches for user: %s", userID.String())
	
	// Get user's search history
	searchHistory, err := w.searchAnalytics.GetUserSearchHistory(ctx, userID, 20)
	if err != nil {
		return fmt.Errorf("failed to get user search history: %w", err)
	}
	
	// Create warming jobs for user's frequent searches
	for _, searchParams := range searchHistory {
		job := &WarmingJob{
			SearchParams: &searchParams,
			UserID:       userID,
			Priority:     7, // High priority for user-specific warming
			EstimatedTTL: time.Hour * 2, // User searches moderate TTL
			JobType:      "user_favorite",
		}
		
		job.SearchKey = w.generateSearchKey(&searchParams, userID)
		
		select {
		case w.warmingQueue <- job:
			log.Printf("Queued user favorite warming job for user %s: %s", userID.String(), job.SearchKey)
		default:
			log.Printf("Warming queue full, skipping user favorite job: %s", job.SearchKey)
		}
	}
	
	return nil
}

func (w *recipeCacheWarmer) WarmTrendingRecipes(ctx context.Context) error {
	log.Printf("Warming trending recipes cache...")
	
	// Get trending recipes from analytics
	trendingRecipeIDs, err := w.searchAnalytics.GetTrendingRecipes(ctx, 30)
	if err != nil {
		return fmt.Errorf("failed to get trending recipes: %w", err)
	}
	
	w.trendingRecipes = trendingRecipeIDs
	
	// Create search params for trending recipe searches
	trendingSearches := []*models.RecipeSearchParams{
		{
			SortBy: "rating",
			Limit:  20,
		},
		{
			SortBy: "popularity",
			Limit:  20,
		},
		{
			SortBy: "created_at",
			Limit:  20,
		},
	}
	
	for _, searchParams := range trendingSearches {
		job := &WarmingJob{
			SearchParams: searchParams,
			UserID:       uuid.New(), // System user for trending
			Priority:     8, // High priority for trending
			EstimatedTTL: time.Minute * 30, // Trending changes frequently
			JobType:      "trending",
		}
		
		job.SearchKey = w.generateSearchKey(searchParams, job.UserID)
		
		select {
		case w.warmingQueue <- job:
			log.Printf("Queued trending recipes warming job: %s", job.SearchKey)
		default:
			log.Printf("Warming queue full, skipping trending job: %s", job.SearchKey)
		}
	}
	
	log.Printf("Queued %d trending recipe warming jobs", len(trendingSearches))
	return nil
}

func (w *recipeCacheWarmer) WarmCuisineSearches(ctx context.Context) error {
	log.Printf("Warming popular cuisine searches...")
	
	// Get popular cuisines from analytics
	popularCuisines, err := w.searchAnalytics.GetPopularCuisines(ctx)
	if err != nil {
		return fmt.Errorf("failed to get popular cuisines: %w", err)
	}
	
	w.cuisinePreferences = popularCuisines
	
	// Create warming jobs for each popular cuisine
	for cuisine, frequency := range popularCuisines {
		if frequency < 5 { // Skip cuisines with low frequency
			continue
		}
		
		searchParams := &models.RecipeSearchParams{
			Cuisine: cuisine,
			Limit:   30,
			SortBy:  "rating",
		}
		
		job := &WarmingJob{
			SearchParams: searchParams,
			UserID:       uuid.New(), // System user for cuisine searches
			Priority:     calculatePriority(frequency, 1),
			EstimatedTTL: time.Hour * 6, // Cuisine searches stable
			JobType:      "cuisine",
		}
		
		job.SearchKey = w.generateSearchKey(searchParams, job.UserID)
		
		select {
		case w.warmingQueue <- job:
			log.Printf("Queued cuisine warming job for %s: %s", cuisine, job.SearchKey)
		default:
			log.Printf("Warming queue full, skipping cuisine job: %s", job.SearchKey)
		}
	}
	
	return nil
}

func (w *recipeCacheWarmer) WarmDietaryFilterSearches(ctx context.Context) error {
	log.Printf("Warming popular dietary filter searches...")
	
	// Get popular dietary filters from analytics
	popularFilters, err := w.searchAnalytics.GetPopularDietaryFilters(ctx)
	if err != nil {
		return fmt.Errorf("failed to get popular dietary filters: %w", err)
	}
	
	w.dietaryPreferences = popularFilters
	
	// Create warming jobs for each popular dietary filter
	for filter, frequency := range popularFilters {
		if frequency < 10 { // Skip filters with low frequency
			continue
		}
		
		searchParams := &models.RecipeSearchParams{
			DietaryRestrictions: []string{filter},
			Limit:              30,
			SortBy:             "rating",
		}
		
		job := &WarmingJob{
			SearchParams: searchParams,
			UserID:       uuid.New(), // System user for dietary searches
			Priority:     calculatePriority(frequency, 1),
			EstimatedTTL: time.Hour * 8, // Dietary searches very stable
			JobType:      "dietary",
		}
		
		job.SearchKey = w.generateSearchKey(searchParams, job.UserID)
		
		select {
		case w.warmingQueue <- job:
			log.Printf("Queued dietary filter warming job for %s: %s", filter, job.SearchKey)
		default:
			log.Printf("Warming queue full, skipping dietary job: %s", job.SearchKey)
		}
	}
	
	return nil
}

func (w *recipeCacheWarmer) ScheduleWarmingJobs(ctx context.Context) error {
	log.Printf("Scheduling regular cache warming jobs...")
	
	// Create a ticker for periodic warming
	ticker := time.NewTicker(time.Hour) // Run warming every hour
	
	go func() {
		defer ticker.Stop()
		
		for {
			select {
			case <-ticker.C:
				w.runScheduledWarming(ctx)
			case <-ctx.Done():
				log.Printf("Cache warming scheduler stopped")
				return
			}
		}
	}()
	
	log.Printf("Cache warming scheduler started (runs every hour)")
	return nil
}

func (w *recipeCacheWarmer) GetWarmingMetrics(ctx context.Context) (*CacheWarmingMetrics, error) {
	// Update efficiency calculation
	if w.metrics.TotalWarmingJobs > 0 {
		successRate := float64(w.metrics.SuccessfulWarmings) / float64(w.metrics.TotalWarmingJobs)
		w.metrics.WarmingEfficiency = successRate * 100
	}
	
	return w.metrics, nil
}

// Helper methods

func (w *recipeCacheWarmer) processWarmingQueue() {
	log.Printf("Starting cache warming queue processor...")
	
	for job := range w.warmingQueue {
		startTime := time.Now()
		
		// Execute the warming job
		err := w.executeWarmingJob(context.Background(), job)
		
		duration := time.Since(startTime)
		w.updateWarmingMetrics(err == nil, duration)
		
		if err != nil {
			log.Printf("Warming job failed for %s: %v", job.SearchKey, err)
			w.metrics.FailedWarmings++
		} else {
			log.Printf("Warming job completed successfully for %s in %v", job.SearchKey, duration)
			w.metrics.SuccessfulWarmings++
		}
		
		w.metrics.TotalWarmingJobs++
		w.metrics.LastWarmingTime = time.Now()
		
		// Small delay to prevent overwhelming the system
		time.Sleep(100 * time.Millisecond)
	}
}

func (w *recipeCacheWarmer) executeWarmingJob(ctx context.Context, job *WarmingJob) error {
	// Check if already cached and fresh
	cached, err := w.cacheService.GetRecipeSearchResults(ctx, job.SearchParams, job.UserID)
	if err == nil && time.Since(cached.CachedAt) < job.EstimatedTTL/2 {
		// Already cached and fresh, skip
		return nil
	}
	
	// Execute the search to warm the cache
	recipes, err := w.recipeRepository.SearchRecipes(ctx, job.UserID, job.SearchParams)
	if err != nil {
		return fmt.Errorf("failed to execute search for warming: %w", err)
	}
	
	// Get total count for the search
	totalCount, err := w.recipeRepository.CountRecipes(ctx, job.UserID, job.SearchParams)
	if err != nil {
		log.Printf("Failed to get count for warming job %s, using recipes length: %v", job.SearchKey, err)
		totalCount = int64(len(recipes))
	}
	
	// Cache the results
	err = w.cacheService.CacheRecipeSearchResults(ctx, job.SearchParams, job.UserID, recipes, totalCount)
	if err != nil {
		return fmt.Errorf("failed to cache warming results: %w", err)
	}
	
	return nil
}

func (w *recipeCacheWarmer) runScheduledWarming(ctx context.Context) {
	log.Printf("Running scheduled cache warming...")
	
	// Run different warming strategies
	warmingTasks := []func(context.Context) error{
		w.WarmPopularSearches,
		w.WarmTrendingRecipes,
		w.WarmCuisineSearches,
		w.WarmDietaryFilterSearches,
	}
	
	for _, task := range warmingTasks {
		if err := task(ctx); err != nil {
			log.Printf("Scheduled warming task failed: %v", err)
		}
	}
	
	log.Printf("Scheduled cache warming completed")
}

func (w *recipeCacheWarmer) generateSearchKey(searchParams *models.RecipeSearchParams, userID uuid.UUID) string {
	// Reuse the same key generation logic from enhanced cache service
	// This is a simplified version - would typically delegate to the cache service
	return fmt.Sprintf("recipe_search:%s:%s", userID.String(), searchParams.Query)
}

func (w *recipeCacheWarmer) updateWarmingMetrics(success bool, duration time.Duration) {
	// Update average warming time
	if w.metrics.TotalWarmingJobs == 0 {
		w.metrics.AverageWarmingTime = duration
	} else {
		totalTime := w.metrics.AverageWarmingTime * time.Duration(w.metrics.TotalWarmingJobs)
		totalTime += duration
		w.metrics.AverageWarmingTime = totalTime / time.Duration(w.metrics.TotalWarmingJobs+1)
	}
}

func calculatePriority(frequency, userCount int) int {
	// Calculate priority based on frequency and user count
	priority := frequency/10 + userCount/5
	
	if priority > 10 {
		priority = 10
	} else if priority < 1 {
		priority = 1
	}
	
	return priority
}