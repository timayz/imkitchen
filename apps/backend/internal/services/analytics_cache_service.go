package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/google/uuid"
	"github.com/go-redis/redis/v8"
)

// AnalyticsCacheService provides optimized caching for analytics operations
type AnalyticsCacheService struct {
	cache      *CacheService
	defaultTTL time.Duration
}

// NewAnalyticsCacheService creates a new analytics cache service
func NewAnalyticsCacheService(cacheService *CacheService) *AnalyticsCacheService {
	return &AnalyticsCacheService{
		cache:      cacheService,
		defaultTTL: 30 * time.Minute, // Analytics cache for 30 minutes
	}
}

// AnalyticsCacheKey generates consistent cache keys for analytics data
func (a *AnalyticsCacheService) AnalyticsCacheKey(userID uuid.UUID, weeks int, includeDetailed bool) string {
	return fmt.Sprintf("analytics:rotation:%s:weeks_%d:detailed_%t", userID.String(), weeks, includeDetailed)
}

// VarietyScoreCacheKey generates cache key for variety score calculations
func (a *AnalyticsCacheService) VarietyScoreCacheKey(userID uuid.UUID, weeks int) string {
	return fmt.Sprintf("analytics:variety_score:%s:weeks_%d", userID.String(), weeks)
}

// DebugLogsCacheKey generates cache key for debug logs
func (a *AnalyticsCacheService) DebugLogsCacheKey(userID uuid.UUID, limit int) string {
	return fmt.Sprintf("analytics:debug_logs:%s:limit_%d", userID.String(), limit)
}

// GetCachedRotationAnalytics retrieves cached rotation analytics or returns nil if not found
func (a *AnalyticsCacheService) GetCachedRotationAnalytics(ctx context.Context, userID uuid.UUID, weeks int, includeDetailed bool) (*RotationAnalytics, error) {
	key := a.AnalyticsCacheKey(userID, weeks, includeDetailed)
	
	cached, err := a.cache.Get(ctx, key)
	if err != nil {
		if err == redis.Nil {
			return nil, nil // Cache miss
		}
		log.Printf("Error retrieving analytics cache: %v", err)
		return nil, nil // Return nil on error to allow fallback
	}
	
	var analytics RotationAnalytics
	if err := json.Unmarshal([]byte(cached), &analytics); err != nil {
		log.Printf("Error unmarshaling cached analytics: %v", err)
		// Delete corrupted cache entry
		_ = a.cache.Delete(ctx, key)
		return nil, nil
	}
	
	log.Printf("Analytics cache hit for user %s", userID.String())
	return &analytics, nil
}

// CacheRotationAnalytics stores rotation analytics in cache
func (a *AnalyticsCacheService) CacheRotationAnalytics(ctx context.Context, userID uuid.UUID, weeks int, includeDetailed bool, analytics *RotationAnalytics) error {
	key := a.AnalyticsCacheKey(userID, weeks, includeDetailed)
	
	// Use shorter TTL for detailed analytics (more likely to change)
	ttl := a.defaultTTL
	if includeDetailed {
		ttl = 15 * time.Minute
	}
	
	if err := a.cache.Set(ctx, key, analytics, ttl); err != nil {
		log.Printf("Failed to cache analytics for user %s: %v", userID.String(), err)
		return err
	}
	
	log.Printf("Cached analytics for user %s (weeks=%d, detailed=%t)", userID.String(), weeks, includeDetailed)
	return nil
}

// GetCachedVarietyScore retrieves cached variety score calculation
func (a *AnalyticsCacheService) GetCachedVarietyScore(ctx context.Context, userID uuid.UUID, weeks int) (*float64, error) {
	key := a.VarietyScoreCacheKey(userID, weeks)
	
	cached, err := a.cache.Get(ctx, key)
	if err != nil {
		if err == redis.Nil {
			return nil, nil // Cache miss
		}
		return nil, err
	}
	
	var score float64
	if err := json.Unmarshal([]byte(cached), &score); err != nil {
		_ = a.cache.Delete(ctx, key)
		return nil, nil
	}
	
	return &score, nil
}

// CacheVarietyScore stores variety score calculation in cache
func (a *AnalyticsCacheService) CacheVarietyScore(ctx context.Context, userID uuid.UUID, weeks int, score float64) error {
	key := a.VarietyScoreCacheKey(userID, weeks)
	
	// Variety scores can be cached longer as they're expensive to calculate
	if err := a.cache.Set(ctx, key, score, time.Hour); err != nil {
		log.Printf("Failed to cache variety score for user %s: %v", userID.String(), err)
		return err
	}
	
	return nil
}

// InvalidateUserAnalyticsCache removes all analytics cache entries for a user
func (a *AnalyticsCacheService) InvalidateUserAnalyticsCache(ctx context.Context, userID uuid.UUID) error {
	// Pattern to match all analytics cache keys for this user
	pattern := fmt.Sprintf("analytics:*:%s:*", userID.String())
	
	// Get all matching keys
	keys, err := a.cache.client.Keys(ctx, pattern).Result()
	if err != nil {
		log.Printf("Failed to get cache keys for user %s: %v", userID.String(), err)
		return err
	}
	
	if len(keys) == 0 {
		return nil
	}
	
	// Delete all matching keys
	if err := a.cache.client.Del(ctx, keys...).Err(); err != nil {
		log.Printf("Failed to invalidate analytics cache for user %s: %v", userID.String(), err)
		return err
	}
	
	log.Printf("Invalidated %d analytics cache entries for user %s", len(keys), userID.String())
	return nil
}

// CacheRotationAnalyticsWithOptions provides advanced caching with incremental updates
func (a *AnalyticsCacheService) CacheRotationAnalyticsWithOptions(ctx context.Context, userID uuid.UUID, weeks int, analytics *RotationAnalytics, options CacheOptions) error {
	key := a.AnalyticsCacheKey(userID, weeks, options.IncludeDetailed)
	
	// Calculate adaptive TTL based on data freshness
	ttl := a.calculateAdaptiveTTL(analytics, options)
	
	if err := a.cache.Set(ctx, key, analytics, ttl); err != nil {
		return err
	}
	
	// Cache individual components for faster partial updates
	if options.CacheComponents {
		if err := a.cacheAnalyticsComponents(ctx, userID, weeks, analytics); err != nil {
			log.Printf("Failed to cache analytics components: %v", err)
		}
	}
	
	return nil
}

// CacheOptions configures analytics caching behavior
type CacheOptions struct {
	IncludeDetailed  bool
	CacheComponents  bool
	ForceRefresh     bool
	AdaptiveTTL      bool
}

// calculateAdaptiveTTL determines cache TTL based on data characteristics
func (a *AnalyticsCacheService) calculateAdaptiveTTL(analytics *RotationAnalytics, options CacheOptions) time.Duration {
	if !options.AdaptiveTTL {
		return a.defaultTTL
	}
	
	// Base TTL
	ttl := a.defaultTTL
	
	// Reduce TTL for recent data (more likely to change)
	if analytics.CalculatedAt != "" {
		if calculatedAt, err := time.Parse(time.RFC3339, analytics.CalculatedAt); err == nil {
			age := time.Since(calculatedAt)
			if age < time.Hour {
				ttl = 10 * time.Minute // Very recent data
			} else if age < 24*time.Hour {
				ttl = 20 * time.Minute // Recent data
			}
		}
	}
	
	// Increase TTL for stable, historical data
	if analytics.WeeksAnalyzed > 12 {
		ttl = 2 * time.Hour // Historical data changes less frequently
	}
	
	return ttl
}

// cacheAnalyticsComponents caches individual components for partial updates
func (a *AnalyticsCacheService) cacheAnalyticsComponents(ctx context.Context, userID uuid.UUID, weeks int, analytics *RotationAnalytics) error {
	// Cache variety score separately for faster access
	varietyKey := fmt.Sprintf("analytics:component:variety:%s:weeks_%d", userID.String(), weeks)
	_ = a.cache.Set(ctx, varietyKey, analytics.VarietyScore, time.Hour)
	
	// Cache complexity distribution for faster chart updates
	complexityKey := fmt.Sprintf("analytics:component:complexity:%s:weeks_%d", userID.String(), weeks)
	_ = a.cache.Set(ctx, complexityKey, analytics.ComplexityDistribution, time.Hour)
	
	// Cache favorites data
	favoritesKey := fmt.Sprintf("analytics:component:favorites:%s:weeks_%d", userID.String(), weeks)
	_ = a.cache.Set(ctx, favoritesKey, analytics.FavoritesFrequency, time.Hour)
	
	return nil
}

// GetCachedDebugLogs retrieves cached debug logs
func (a *AnalyticsCacheService) GetCachedDebugLogs(ctx context.Context, userID uuid.UUID, limit int) ([]RotationDebugLog, error) {
	key := a.DebugLogsCacheKey(userID, limit)
	
	cached, err := a.cache.Get(ctx, key)
	if err != nil {
		if err == redis.Nil {
			return nil, nil // Cache miss
		}
		return nil, err
	}
	
	var logs []RotationDebugLog
	if err := json.Unmarshal([]byte(cached), &logs); err != nil {
		_ = a.cache.Delete(ctx, key)
		return nil, nil
	}
	
	return logs, nil
}

// CacheDebugLogs stores debug logs in cache
func (a *AnalyticsCacheService) CacheDebugLogs(ctx context.Context, userID uuid.UUID, limit int, logs []RotationDebugLog) error {
	key := a.DebugLogsCacheKey(userID, limit)
	
	// Debug logs cache for shorter time as they're frequently updated
	if err := a.cache.Set(ctx, key, logs, 5*time.Minute); err != nil {
		log.Printf("Failed to cache debug logs for user %s: %v", userID.String(), err)
		return err
	}
	
	return nil
}

// Placeholder types for analytics data structures
type RotationAnalytics struct {
	VarietyScore           float64                 `json:"varietyScore"`
	RotationEfficiency     float64                 `json:"rotationEfficiency"`
	WeeksAnalyzed          int                     `json:"weeksAnalyzed"`
	ComplexityDistribution map[string]float64      `json:"complexityDistribution"`
	ComplexityTrends       []ComplexityTrendData   `json:"complexityTrends"`
	FavoritesFrequency     map[string]int          `json:"favoritesFrequency"`
	FavoritesImpact        float64                 `json:"favoritesImpact"`
	WeeklyPatterns         []WeeklyAnalysisData    `json:"weeklyPatterns"`
	CalculatedAt           string                  `json:"calculatedAt"`
}

type ComplexityTrendData struct {
	Week               string  `json:"week"`
	AverageComplexity  float64 `json:"averageComplexity"`
	PrepTimeMinutes    float64 `json:"prepTimeMinutes"`
	RecipeCount        int     `json:"recipeCount"`
}

type WeeklyAnalysisData struct {
	WeekNumber       int     `json:"weekNumber"`
	WeekStartDate    string  `json:"weekStartDate"`
	VarietyScore     float64 `json:"varietyScore"`
	PatternAdherence float64 `json:"patternAdherence"`
	FavoritesUsed    int     `json:"favoritesUsed"`
	TotalMeals       int     `json:"totalMeals"`
}

type RotationDebugLog struct {
	ID                 string    `json:"id"`
	Timestamp          string    `json:"timestamp"`
	DecisionType       string    `json:"decisionType"`
	RecipeID           *string   `json:"recipeId,omitempty"`
	RecipeName         *string   `json:"recipeName,omitempty"`
	ConstraintViolated *string   `json:"constraintViolated,omitempty"`
	FallbackReason     *string   `json:"fallbackReason,omitempty"`
	AlgorithmVersion   string    `json:"algorithmVersion"`
}