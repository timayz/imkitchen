package services

import (
	"context"
	"fmt"
	"log"
	"math/rand"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// OptimizedRotationService provides high-performance meal plan generation
type OptimizedRotationService interface {
	// Enhanced meal plan generation with performance optimization
	GenerateMealPlanFast(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, error)
	GenerateMealPlanWithPatternsFast(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error)
	GenerateMealPlanWithConstraintsFast(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, *RotationConstraintReport, error)

	// Performance utilities
	PrewarmUserCache(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) error
	GetGenerationMetrics(ctx context.Context, userID uuid.UUID) (*MealPlanGenerationMetrics, error)

	// Streaming generation for progress tracking
	GenerateMealPlanStreaming(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, progressChan chan<- *MealPlanProgress) (*models.WeeklyMeals, error)
}

// MealPlanGenerationMetrics tracks performance metrics
type MealPlanGenerationMetrics struct {
	UserID                uuid.UUID     `json:"userId"`
	AverageGenerationTime time.Duration `json:"averageGenerationTime"`
	CacheHitRate          float64       `json:"cacheHitRate"`
	TotalGenerations      int           `json:"totalGenerations"`
	FastestGeneration     time.Duration `json:"fastestGeneration"`
	SlowestGeneration     time.Duration `json:"slowestGeneration"`
	LastGenerated         time.Time     `json:"lastGenerated"`
}

// MealPlanProgress represents streaming progress updates
type MealPlanProgress struct {
	Step       string    `json:"step"`
	Progress   float64   `json:"progress"` // 0.0 to 1.0
	Message    string    `json:"message"`
	Timestamp  time.Time `json:"timestamp"`
	MealsReady int       `json:"mealsReady"` // Number of meals selected so far
	TotalMeals int       `json:"totalMeals"` // Total meals to select (21)
}

type optimizedRotationService struct {
	rotationService    RotationService
	recipeIndexService RecipeIndexService
	mealPlanCache      MealPlanCacheService
	cache              *CacheService
	recipeRepo         repositories.RecipeRepository
	timeout            time.Duration
}

func NewOptimizedRotationService(
	rotationService RotationService,
	recipeIndexService RecipeIndexService,
	mealPlanCache MealPlanCacheService,
	cache *CacheService,
	recipeRepo repositories.RecipeRepository,
) OptimizedRotationService {
	return &optimizedRotationService{
		rotationService:    rotationService,
		recipeIndexService: recipeIndexService,
		mealPlanCache:      mealPlanCache,
		cache:              cache,
		recipeRepo:         recipeRepo,
		timeout:            2 * time.Second, // 2-second target
	}
}

// GenerateMealPlanFast generates a meal plan optimized for speed
func (o *optimizedRotationService) GenerateMealPlanFast(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, error) {
	startTime := time.Now()

	// Create cache key
	cacheKey := CreateMealPlanCacheKey(userID, preferences, nil, 0, "fast")

	// Try cache first
	result, err := o.mealPlanCache.GetOrGenerateMealPlan(ctx, cacheKey, func() (*models.WeeklyMeals, *RotationConstraintReport, error) {
		return o.generateMealPlanOptimized(ctx, userID, preferences, nil)
	})

	if err != nil {
		return nil, fmt.Errorf("failed to generate meal plan: %w", err)
	}

	generationTime := time.Since(startTime)
	log.Printf("Generated meal plan for user %s in %v (target: %v)", userID.String(), generationTime, o.timeout)

	// Track metrics
	go o.recordGenerationMetrics(ctx, userID, generationTime, result.CacheKey != "")

	// Warn if we exceeded target time
	if generationTime > o.timeout {
		log.Printf("WARNING: Meal plan generation exceeded target time: %v > %v", generationTime, o.timeout)
	}

	return result.MealPlan, nil
}

// GenerateMealPlanWithPatternsFast generates a pattern-aware meal plan optimized for speed
func (o *optimizedRotationService) GenerateMealPlanWithPatternsFast(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error) {
	startTime := time.Now()

	// Create cache key
	cacheKey := CreateMealPlanCacheKey(userID, preferences, patterns, 0, "pattern-aware")

	// Try cache first
	result, err := o.mealPlanCache.GetOrGenerateMealPlan(ctx, cacheKey, func() (*models.WeeklyMeals, *RotationConstraintReport, error) {
		return o.generateMealPlanOptimized(ctx, userID, preferences, patterns)
	})

	if err != nil {
		return nil, fmt.Errorf("failed to generate pattern-aware meal plan: %w", err)
	}

	generationTime := time.Since(startTime)
	log.Printf("Generated pattern-aware meal plan for user %s in %v", userID.String(), generationTime)

	// Track metrics
	go o.recordGenerationMetrics(ctx, userID, generationTime, result.CacheKey != "")

	return result.MealPlan, nil
}

// GenerateMealPlanWithConstraintsFast generates a meal plan with constraint reporting
func (o *optimizedRotationService) GenerateMealPlanWithConstraintsFast(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, *RotationConstraintReport, error) {
	startTime := time.Now()

	// Create cache key
	cacheKey := CreateMealPlanCacheKey(userID, preferences, nil, 0, "constraint-handling")

	// Try cache first
	result, err := o.mealPlanCache.GetOrGenerateMealPlan(ctx, cacheKey, func() (*models.WeeklyMeals, *RotationConstraintReport, error) {
		return o.generateMealPlanWithConstraints(ctx, userID, preferences)
	})

	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate meal plan with constraints: %w", err)
	}

	generationTime := time.Since(startTime)
	log.Printf("Generated constraint-aware meal plan for user %s in %v", userID.String(), generationTime)

	// Track metrics
	go o.recordGenerationMetrics(ctx, userID, generationTime, result.CacheKey != "")

	return result.MealPlan, result.ConstraintReport, nil
}

// generateMealPlanOptimized uses optimized algorithms for fast generation
func (o *optimizedRotationService) generateMealPlanOptimized(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, *RotationConstraintReport, error) {
	// Set timeout context
	ctx, cancel := context.WithTimeout(ctx, o.timeout)
	defer cancel()

	// Get rotation state
	rotationState, err := o.rotationService.GetRotationState(userID)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to get rotation state: %w", err)
	}

	// Get recipe index for fast filtering
	recipeIndex, err := o.recipeIndexService.GetUserIndex(ctx, userID)
	if err != nil {
		log.Printf("Failed to get recipe index, falling back to standard generation: %v", err)
		// Fallback to standard generation if index is unavailable
		if patterns != nil {
			weeklyMeals, err := o.rotationService.SelectRecipesForWeekWithPatterns(userID, preferences, patterns)
			return weeklyMeals, nil, err
		} else {
			weeklyMeals, err := o.rotationService.SelectRecipesForWeek(userID, preferences)
			return weeklyMeals, nil, err
		}
	}

	// Initialize result
	weeklyMeals := &models.WeeklyMeals{
		Monday:    make([]models.MealSlot, 3),
		Tuesday:   make([]models.MealSlot, 3),
		Wednesday: make([]models.MealSlot, 3),
		Thursday:  make([]models.MealSlot, 3),
		Friday:    make([]models.MealSlot, 3),
		Saturday:  make([]models.MealSlot, 3),
		Sunday:    make([]models.MealSlot, 3),
	}

	usedRecipesThisWeek := make(map[string]bool)
	days := []string{"monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"}
	mealTypes := []string{"breakfast", "lunch", "dinner"}

	// Fast generation using recipe indices
	for dayIndex, day := range days {
		dayMeals := make([]models.MealSlot, 3)

		for mealIndex, mealType := range mealTypes {
			// Check timeout
			select {
			case <-ctx.Done():
				return nil, nil, fmt.Errorf("meal plan generation timed out")
			default:
			}

			// Create fast criteria
			criteria := &RecipeSelectionCriteria{
				MealType:            mealType,
				Day:                 day,
				DayOfWeek:           dayIndex,
				MaxPrepTime:         &preferences.MaxPrepTimePerMeal,
				DietaryRestrictions: preferences.DietaryRestrictions,
				CuisinePreferences:  preferences.CuisinePreferences,
				UsedThisWeek:        usedRecipesThisWeek,
			}

			// Add rotation avoidance
			for recipeID := range rotationState.UsedRecipes {
				criteria.AvoidRecipeIDs = append(criteria.AvoidRecipeIDs, recipeID)
			}

			// Apply complexity preferences
			criteria.PreferredComplexity = o.getOptimizedComplexity(preferences, dayIndex, mealIndex)

			// Fast candidate selection using index
			candidateIDs, err := o.recipeIndexService.FindCandidates(ctx, userID, criteria)
			if err != nil {
				return nil, nil, fmt.Errorf("failed to find candidates for %s %s: %w", day, mealType, err)
			}

			if len(candidateIDs) == 0 {
				return nil, nil, fmt.Errorf("no suitable recipes found for %s %s", day, mealType)
			}

			// Fast selection from candidates
			selectedID := o.fastSelectFromCandidates(candidateIDs, recipeIndex.RecipeMetadata, criteria)
			selectedMetadata := recipeIndex.RecipeMetadata[selectedID]

			// Create meal slot with essential data
			dayMeals[mealIndex] = models.MealSlot{
				Day:      day,
				MealType: mealType,
				RecipeID: &selectedID,
				Servings: selectedMetadata.Servings,
				// Note: Full Recipe object would be populated later if needed
			}

			// Track usage
			usedRecipesThisWeek[selectedID] = true
			rotationState.UsedRecipes[selectedID] = time.Now()
		}

		// Assign to correct day
		switch day {
		case "monday":
			weeklyMeals.Monday = dayMeals
		case "tuesday":
			weeklyMeals.Tuesday = dayMeals
		case "wednesday":
			weeklyMeals.Wednesday = dayMeals
		case "thursday":
			weeklyMeals.Thursday = dayMeals
		case "friday":
			weeklyMeals.Friday = dayMeals
		case "saturday":
			weeklyMeals.Saturday = dayMeals
		case "sunday":
			weeklyMeals.Sunday = dayMeals
		}
	}

	// Update rotation state asynchronously
	go func() {
		if err := o.rotationService.UpdateRotationState(userID, rotationState); err != nil {
			log.Printf("Warning: failed to update rotation state: %v", err)
		}
	}()

	return weeklyMeals, nil, nil
}

// generateMealPlanWithConstraints generates a meal plan with detailed constraint reporting
func (o *optimizedRotationService) generateMealPlanWithConstraints(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, *RotationConstraintReport, error) {
	// Use standard rotation service for constraint handling
	return o.rotationService.SelectRecipesForWeekWithConstraintHandling(userID, preferences)
}

// GenerateMealPlanStreaming provides streaming progress updates
func (o *optimizedRotationService) GenerateMealPlanStreaming(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, progressChan chan<- *MealPlanProgress) (*models.WeeklyMeals, error) {
	defer close(progressChan)

	// Send initial progress
	progressChan <- &MealPlanProgress{
		Step:       "initializing",
		Progress:   0.0,
		Message:    "Starting meal plan generation...",
		Timestamp:  time.Now(),
		MealsReady: 0,
		TotalMeals: 21,
	}

	// Check cache first
	cacheKey := CreateMealPlanCacheKey(userID, preferences, nil, 0, "streaming")
	if cached, err := o.mealPlanCache.GetCachedMealPlan(ctx, cacheKey); err == nil {
		progressChan <- &MealPlanProgress{
			Step:       "cache_hit",
			Progress:   1.0,
			Message:    "Retrieved from cache",
			Timestamp:  time.Now(),
			MealsReady: 21,
			TotalMeals: 21,
		}
		return cached.MealPlan, nil
	}

	// Generate with progress tracking
	progressChan <- &MealPlanProgress{
		Step:       "indexing",
		Progress:   0.1,
		Message:    "Loading recipe index...",
		Timestamp:  time.Now(),
		MealsReady: 0,
		TotalMeals: 21,
	}

	startTime := time.Now()
	mealPlan, _, err := o.generateMealPlanOptimized(ctx, userID, preferences, nil)
	if err != nil {
		progressChan <- &MealPlanProgress{
			Step:      "error",
			Progress:  0.0,
			Message:   fmt.Sprintf("Generation failed: %v", err),
			Timestamp: time.Now(),
		}
		return nil, err
	}

	// Cache result
	go func() {
		cacheKey := CreateMealPlanCacheKey(userID, preferences, nil, 0, "streaming")
		o.mealPlanCache.CacheMealPlan(context.Background(), cacheKey, mealPlan, nil, 2*time.Hour)
	}()

	progressChan <- &MealPlanProgress{
		Step:       "completed",
		Progress:   1.0,
		Message:    fmt.Sprintf("Meal plan generated in %v", time.Since(startTime)),
		Timestamp:  time.Now(),
		MealsReady: 21,
		TotalMeals: 21,
	}

	return mealPlan, nil
}

// PrewarmUserCache pre-generates common meal plan scenarios
func (o *optimizedRotationService) PrewarmUserCache(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) error {
	return o.mealPlanCache.PrewarmCache(ctx, userID, preferences, patterns)
}

// GetGenerationMetrics returns performance metrics for a user
func (o *optimizedRotationService) GetGenerationMetrics(ctx context.Context, userID uuid.UUID) (*MealPlanGenerationMetrics, error) {
	cacheKey := fmt.Sprintf("meal_plan_metrics:%s", userID.String())

	cached, err := o.cache.Get(ctx, cacheKey)
	if err != nil {
		// Return default metrics if none cached
		return &MealPlanGenerationMetrics{
			UserID:                userID,
			AverageGenerationTime: 1500 * time.Millisecond, // Default estimate
			CacheHitRate:          0.0,
			TotalGenerations:      0,
			FastestGeneration:     0,
			SlowestGeneration:     0,
			LastGenerated:         time.Time{},
		}, nil
	}

	var metrics MealPlanGenerationMetrics
	if err := json.Unmarshal([]byte(cached), &metrics); err != nil {
		return nil, fmt.Errorf("failed to unmarshal metrics: %w", err)
	}

	return &metrics, nil
}

// Helper methods

func (o *optimizedRotationService) getOptimizedComplexity(preferences *models.UserPreferences, dayIndex, mealIndex int) []string {
	// Simplified complexity assignment for speed
	if mealIndex == 0 { // Breakfast
		return []string{"simple"}
	}

	// Apply skill level constraints
	switch preferences.CookingSkillLevel {
	case "beginner":
		return []string{"simple"}
	case "advanced":
		if dayIndex >= 5 { // Weekend
			return []string{"simple", "moderate", "complex"}
		}
		return []string{"simple", "moderate"}
	default: // intermediate
		return []string{"simple", "moderate"}
	}
}

func (o *optimizedRotationService) fastSelectFromCandidates(candidateIDs []string, metadata map[string]*RecipeIndexEntry, criteria *RecipeSelectionCriteria) string {
	if len(candidateIDs) == 0 {
		return ""
	}

	// For speed, select from top candidates with some randomness
	topCount := 5
	if len(candidateIDs) < topCount {
		topCount = len(candidateIDs)
	}

	// Add some weighted randomness - higher scored recipes have better chance
	weights := make([]float64, topCount)
	totalWeight := 0.0

	for i := 0; i < topCount; i++ {
		meta := metadata[candidateIDs[i]]
		if meta != nil {
			// Higher position in sorted list = higher weight
			weight := float64(topCount-i) * meta.Score
			weights[i] = weight
			totalWeight += weight
		}
	}

	// Weighted random selection
	if totalWeight > 0 {
		randomValue := rand.Float64() * totalWeight
		currentWeight := 0.0

		for i, weight := range weights {
			currentWeight += weight
			if randomValue <= currentWeight {
				return candidateIDs[i]
			}
		}
	}

	// Fallback to first candidate
	return candidateIDs[0]
}

func (o *optimizedRotationService) recordGenerationMetrics(ctx context.Context, userID uuid.UUID, generationTime time.Duration, cacheHit bool) {
	cacheKey := fmt.Sprintf("meal_plan_metrics:%s", userID.String())

	// Get existing metrics
	metrics, _ := o.GetGenerationMetrics(ctx, userID)

	// Update metrics
	metrics.TotalGenerations++
	metrics.LastGenerated = time.Now()

	// Update timing metrics
	if metrics.FastestGeneration == 0 || generationTime < metrics.FastestGeneration {
		metrics.FastestGeneration = generationTime
	}
	if generationTime > metrics.SlowestGeneration {
		metrics.SlowestGeneration = generationTime
	}

	// Update average (simple moving average)
	if metrics.AverageGenerationTime == 0 {
		metrics.AverageGenerationTime = generationTime
	} else {
		metrics.AverageGenerationTime = (metrics.AverageGenerationTime + generationTime) / 2
	}

	// Update cache hit rate
	if cacheHit {
		metrics.CacheHitRate = (metrics.CacheHitRate*float64(metrics.TotalGenerations-1) + 1.0) / float64(metrics.TotalGenerations)
	} else {
		metrics.CacheHitRate = (metrics.CacheHitRate * float64(metrics.TotalGenerations-1)) / float64(metrics.TotalGenerations)
	}

	// Store updated metrics
	o.cache.Set(ctx, cacheKey, metrics, 24*time.Hour)
}
