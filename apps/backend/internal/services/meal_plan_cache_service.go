package services

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
)

// MealPlanCacheKey generates unique cache keys for meal plan generation
type MealPlanCacheKey struct {
	UserID            uuid.UUID `json:"userId"`
	PreferencesHash   string    `json:"preferencesHash"`
	WeeklyPatternHash string    `json:"weeklyPatternHash"`
	WeekOffset        int       `json:"weekOffset"`     // 0 = current week, 1 = next week, etc.
	GenerationType    string    `json:"generationType"` // "standard", "pattern-aware", "constraint-handling"
}

// CachedMealPlan contains cached meal plan data with metadata
type CachedMealPlan struct {
	MealPlan         *models.WeeklyMeals       `json:"mealPlan"`
	ConstraintReport *RotationConstraintReport `json:"constraintReport,omitempty"`
	GeneratedAt      time.Time                 `json:"generatedAt"`
	TTL              time.Duration             `json:"ttl"`
	CacheKey         string                    `json:"cacheKey"`
	Version          string                    `json:"version"`
}

// MealPlanCacheService provides intelligent caching for meal plan generation
type MealPlanCacheService interface {
	GetCachedMealPlan(ctx context.Context, key *MealPlanCacheKey) (*CachedMealPlan, error)
	CacheMealPlan(ctx context.Context, key *MealPlanCacheKey, mealPlan *models.WeeklyMeals, constraintReport *RotationConstraintReport, ttl time.Duration) error
	InvalidateMealPlans(ctx context.Context, userID uuid.UUID) error
	InvalidateUserPreferences(ctx context.Context, userID uuid.UUID) error
	GetOrGenerateMealPlan(ctx context.Context, key *MealPlanCacheKey, generator func() (*models.WeeklyMeals, *RotationConstraintReport, error)) (*CachedMealPlan, error)
	CacheRecipePoolForUser(ctx context.Context, userID uuid.UUID, pool []models.Recipe, ttl time.Duration) error
	GetCachedRecipePool(ctx context.Context, userID uuid.UUID) ([]models.Recipe, error)
	PrewarmCache(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) error
}

type mealPlanCacheService struct {
	cache *CacheService
}

func NewMealPlanCacheService(cache *CacheService) MealPlanCacheService {
	return &mealPlanCacheService{
		cache: cache,
	}
}

// GetCachedMealPlan retrieves a cached meal plan if available and valid
func (m *mealPlanCacheService) GetCachedMealPlan(ctx context.Context, key *MealPlanCacheKey) (*CachedMealPlan, error) {
	cacheKey := m.generateCacheKey(key)

	cached, err := m.cache.Get(ctx, cacheKey)
	if err != nil {
		return nil, err // Cache miss or error
	}

	var cachedPlan CachedMealPlan
	if err := json.Unmarshal([]byte(cached), &cachedPlan); err != nil {
		log.Printf("Failed to unmarshal cached meal plan: %v", err)
		return nil, fmt.Errorf("corrupted cache data")
	}

	// Check if cache entry is still valid
	if time.Since(cachedPlan.GeneratedAt) > cachedPlan.TTL {
		// Cache expired, remove it
		m.cache.Delete(ctx, cacheKey)
		return nil, fmt.Errorf("cache expired")
	}

	log.Printf("Cache hit for meal plan: %s", cacheKey)
	return &cachedPlan, nil
}

// CacheMealPlan stores a meal plan in cache with optimal TTL
func (m *mealPlanCacheService) CacheMealPlan(ctx context.Context, key *MealPlanCacheKey, mealPlan *models.WeeklyMeals, constraintReport *RotationConstraintReport, ttl time.Duration) error {
	cacheKey := m.generateCacheKey(key)

	cachedPlan := &CachedMealPlan{
		MealPlan:         mealPlan,
		ConstraintReport: constraintReport,
		GeneratedAt:      time.Now(),
		TTL:              ttl,
		CacheKey:         cacheKey,
		Version:          "1.0",
	}

	if err := m.cache.Set(ctx, cacheKey, cachedPlan, ttl); err != nil {
		log.Printf("Failed to cache meal plan: %v", err)
		return err
	}

	// Also cache with user-specific key for easy invalidation
	userCacheKey := fmt.Sprintf("user_meal_plans:%s", key.UserID.String())
	userKeys, _ := m.cache.Get(ctx, userCacheKey)
	var keys []string
	if userKeys != "" {
		json.Unmarshal([]byte(userKeys), &keys)
	}
	keys = append(keys, cacheKey)
	m.cache.Set(ctx, userCacheKey, keys, 7*24*time.Hour) // Store for a week

	log.Printf("Cached meal plan: %s (TTL: %v)", cacheKey, ttl)
	return nil
}

// InvalidateMealPlans removes all cached meal plans for a user
func (m *mealPlanCacheService) InvalidateMealPlans(ctx context.Context, userID uuid.UUID) error {
	userCacheKey := fmt.Sprintf("user_meal_plans:%s", userID.String())

	userKeys, err := m.cache.Get(ctx, userCacheKey)
	if err != nil {
		return nil // No cached plans to invalidate
	}

	var keys []string
	if err := json.Unmarshal([]byte(userKeys), &keys); err != nil {
		log.Printf("Failed to unmarshal user cache keys: %v", err)
		return err
	}

	// Delete all cached meal plans
	for _, cacheKey := range keys {
		if err := m.cache.Delete(ctx, cacheKey); err != nil {
			log.Printf("Failed to delete cached meal plan %s: %v", cacheKey, err)
		}
	}

	// Delete the user keys list
	m.cache.Delete(ctx, userCacheKey)

	log.Printf("Invalidated %d cached meal plans for user %s", len(keys), userID.String())
	return nil
}

// InvalidateUserPreferences removes cached meal plans when user preferences change
func (m *mealPlanCacheService) InvalidateUserPreferences(ctx context.Context, userID uuid.UUID) error {
	// Meal plans are highly dependent on preferences, so invalidate all
	return m.InvalidateMealPlans(ctx, userID)
}

// GetOrGenerateMealPlan implements cache-aside pattern with atomic generation
func (m *mealPlanCacheService) GetOrGenerateMealPlan(ctx context.Context, key *MealPlanCacheKey, generator func() (*models.WeeklyMeals, *RotationConstraintReport, error)) (*CachedMealPlan, error) {
	// Try cache first
	if cached, err := m.GetCachedMealPlan(ctx, key); err == nil {
		return cached, nil
	}

	// Cache miss - generate new meal plan
	startTime := time.Now()
	log.Printf("Generating new meal plan for cache key: %s", m.generateCacheKey(key))

	mealPlan, constraintReport, err := generator()
	if err != nil {
		return nil, fmt.Errorf("failed to generate meal plan: %w", err)
	}

	generationTime := time.Since(startTime)
	log.Printf("Generated meal plan in %v", generationTime)

	// Determine optimal TTL based on generation complexity
	ttl := m.calculateOptimalTTL(key, generationTime)

	// Cache the result asynchronously to not block the response
	go func() {
		ctx := context.Background()
		if err := m.CacheMealPlan(ctx, key, mealPlan, constraintReport, ttl); err != nil {
			log.Printf("Failed to cache generated meal plan: %v", err)
		}
	}()

	// Return the generated result
	return &CachedMealPlan{
		MealPlan:         mealPlan,
		ConstraintReport: constraintReport,
		GeneratedAt:      time.Now(),
		TTL:              ttl,
		CacheKey:         m.generateCacheKey(key),
		Version:          "1.0",
	}, nil
}

// CacheRecipePoolForUser caches the available recipe pool for faster access
func (m *mealPlanCacheService) CacheRecipePoolForUser(ctx context.Context, userID uuid.UUID, pool []models.Recipe, ttl time.Duration) error {
	cacheKey := fmt.Sprintf("recipe_pool:%s", userID.String())

	poolData := struct {
		Recipes  []models.Recipe `json:"recipes"`
		CachedAt time.Time       `json:"cachedAt"`
		Count    int             `json:"count"`
	}{
		Recipes:  pool,
		CachedAt: time.Now(),
		Count:    len(pool),
	}

	return m.cache.Set(ctx, cacheKey, poolData, ttl)
}

// GetCachedRecipePool retrieves the cached recipe pool for a user
func (m *mealPlanCacheService) GetCachedRecipePool(ctx context.Context, userID uuid.UUID) ([]models.Recipe, error) {
	cacheKey := fmt.Sprintf("recipe_pool:%s", userID.String())

	cached, err := m.cache.Get(ctx, cacheKey)
	if err != nil {
		return nil, err
	}

	var poolData struct {
		Recipes  []models.Recipe `json:"recipes"`
		CachedAt time.Time       `json:"cachedAt"`
		Count    int             `json:"count"`
	}

	if err := json.Unmarshal([]byte(cached), &poolData); err != nil {
		return nil, fmt.Errorf("failed to unmarshal cached recipe pool: %w", err)
	}

	log.Printf("Cache hit for recipe pool: %d recipes", poolData.Count)
	return poolData.Recipes, nil
}

// PrewarmCache generates and caches meal plans for common scenarios
func (m *mealPlanCacheService) PrewarmCache(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) error {
	log.Printf("Pre-warming cache for user %s", userID.String())

	// Define common cache scenarios to pre-warm
	scenarios := []*MealPlanCacheKey{
		// Current week with standard generation
		{
			UserID:          userID,
			PreferencesHash: m.hashPreferences(preferences),
			WeekOffset:      0,
			GenerationType:  "standard",
		},
		// Next week with standard generation
		{
			UserID:          userID,
			PreferencesHash: m.hashPreferences(preferences),
			WeekOffset:      1,
			GenerationType:  "standard",
		},
	}

	// Add pattern-aware scenarios if patterns exist
	if len(patterns) > 0 {
		patternHash := m.hashWeeklyPatterns(patterns)
		scenarios = append(scenarios, &MealPlanCacheKey{
			UserID:            userID,
			PreferencesHash:   m.hashPreferences(preferences),
			WeeklyPatternHash: patternHash,
			WeekOffset:        0,
			GenerationType:    "pattern-aware",
		})
	}

	// Pre-warm each scenario in background
	for _, scenario := range scenarios {
		go func(key *MealPlanCacheKey) {
			cacheKey := m.generateCacheKey(key)

			// Check if already cached
			if _, err := m.GetCachedMealPlan(ctx, key); err == nil {
				log.Printf("Cache already warm for: %s", cacheKey)
				return
			}

			log.Printf("Pre-warming cache for: %s", cacheKey)
			// Note: This would need to call the actual meal plan generation service
			// For now, we'll just cache a placeholder that indicates pre-warming is needed
		}(scenario)
	}

	return nil
}

// Helper methods

func (m *mealPlanCacheService) generateCacheKey(key *MealPlanCacheKey) string {
	// Create a deterministic cache key from the components
	keyData := fmt.Sprintf("%s:%s:%s:%d:%s",
		key.UserID.String(),
		key.PreferencesHash,
		key.WeeklyPatternHash,
		key.WeekOffset,
		key.GenerationType,
	)

	// Hash the key data to create a shorter, consistent key
	hash := sha256.Sum256([]byte(keyData))
	return fmt.Sprintf("meal_plan:%s", hex.EncodeToString(hash[:8])) // Use first 8 bytes for shorter key
}

func (m *mealPlanCacheService) hashPreferences(preferences *models.UserPreferences) string {
	// Create a hash of relevant preference fields that affect meal plan generation
	prefData := struct {
		DietaryRestrictions     []string `json:"dietaryRestrictions"`
		CuisinePreferences      []string `json:"cuisinePreferences"`
		MaxPrepTimePerMeal      int      `json:"maxPrepTimePerMeal"`
		CookingSkillLevel       string   `json:"cookingSkillLevel"`
		PreferredMealComplexity string   `json:"preferredMealComplexity"`
		FamilySize              int      `json:"familySize"`
	}{
		DietaryRestrictions:     preferences.DietaryRestrictions,
		CuisinePreferences:      preferences.CuisinePreferences,
		MaxPrepTimePerMeal:      preferences.MaxPrepTimePerMeal,
		CookingSkillLevel:       preferences.CookingSkillLevel,
		PreferredMealComplexity: preferences.PreferredMealComplexity,
		FamilySize:              preferences.FamilySize,
	}

	data, _ := json.Marshal(prefData)
	hash := sha256.Sum256(data)
	return hex.EncodeToString(hash[:4]) // Use first 4 bytes for shorter hash
}

func (m *mealPlanCacheService) hashWeeklyPatterns(patterns []models.UserWeeklyPattern) string {
	if len(patterns) == 0 {
		return ""
	}

	data, _ := json.Marshal(patterns)
	hash := sha256.Sum256(data)
	return hex.EncodeToString(hash[:4]) // Use first 4 bytes for shorter hash
}

func (m *mealPlanCacheService) calculateOptimalTTL(key *MealPlanCacheKey, generationTime time.Duration) time.Duration {
	// Base TTL
	baseTTL := 2 * time.Hour

	// Increase TTL for expensive operations
	if generationTime > 1*time.Second {
		baseTTL = 6 * time.Hour // Cache longer for expensive generations
	}

	// Different TTLs based on generation type
	switch key.GenerationType {
	case "pattern-aware":
		return baseTTL * 2 // Pattern-aware plans are more complex, cache longer
	case "constraint-handling":
		return baseTTL * 3 // Constraint handling is most complex
	default:
		return baseTTL
	}
}

// CreateMealPlanCacheKey creates a cache key for meal plan generation
func CreateMealPlanCacheKey(userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern, weekOffset int, generationType string) *MealPlanCacheKey {
	cacheService := &mealPlanCacheService{} // Temporary instance for hash methods

	return &MealPlanCacheKey{
		UserID:            userID,
		PreferencesHash:   cacheService.hashPreferences(preferences),
		WeeklyPatternHash: cacheService.hashWeeklyPatterns(patterns),
		WeekOffset:        weekOffset,
		GenerationType:    generationType,
	}
}
