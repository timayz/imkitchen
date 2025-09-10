package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
)

// StreamingMealPlanService provides real-time progress tracking for meal plan generation
type StreamingMealPlanService interface {
	// Streaming generation with progress updates
	GenerateWithProgress(ctx context.Context, req *StreamingMealPlanRequest) (<-chan *MealPlanProgressUpdate, error)

	// Stream management
	GetActiveStreams(userID uuid.UUID) []string
	CancelStream(streamID string) error
	CleanupExpiredStreams() int

	// Timeout handling
	GenerateWithTimeout(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, timeout time.Duration) (*models.WeeklyMeals, error)
	GenerateWithFallbacks(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*MealPlanGenerationResult, error)
}

// StreamingMealPlanRequest contains request parameters for streaming generation
type StreamingMealPlanRequest struct {
	UserID          uuid.UUID                  `json:"userId"`
	Preferences     *models.UserPreferences    `json:"preferences"`
	WeeklyPatterns  []models.UserWeeklyPattern `json:"weeklyPatterns,omitempty"`
	GenerationType  string                     `json:"generationType"` // "standard", "pattern-aware", "constraint-handling"
	StreamID        string                     `json:"streamId"`
	TimeoutSeconds  int                        `json:"timeoutSeconds"`
	EnableFallbacks bool                       `json:"enableFallbacks"`
}

// MealPlanProgressUpdate represents a progress update during generation
type MealPlanProgressUpdate struct {
	StreamID               string        `json:"streamId"`
	UserID                 uuid.UUID     `json:"userId"`
	Stage                  string        `json:"stage"`
	Progress               float64       `json:"progress"` // 0.0 to 1.0
	Message                string        `json:"message"`
	CurrentStep            string        `json:"currentStep"`
	StepsCompleted         int           `json:"stepsCompleted"`
	TotalSteps             int           `json:"totalSteps"`
	MealsGenerated         int           `json:"mealsGenerated"`
	TotalMeals             int           `json:"totalMeals"`
	ElapsedTime            time.Duration `json:"elapsedTime"`
	EstimatedTimeRemaining time.Duration `json:"estimatedTimeRemaining"`
	Timestamp              time.Time     `json:"timestamp"`
	Error                  string        `json:"error,omitempty"`

	// Optional result data
	MealPlan         *models.WeeklyMeals       `json:"mealPlan,omitempty"`
	ConstraintReport *RotationConstraintReport `json:"constraintReport,omitempty"`
	Completed        bool                      `json:"completed"`
}

// MealPlanGenerationResult contains the result of generation with fallback information
type MealPlanGenerationResult struct {
	MealPlan         *models.WeeklyMeals       `json:"mealPlan"`
	ConstraintReport *RotationConstraintReport `json:"constraintReport,omitempty"`
	GenerationTime   time.Duration             `json:"generationTime"`
	FallbacksUsed    []string                  `json:"fallbacksUsed"`
	CacheHit         bool                      `json:"cacheHit"`
	Success          bool                      `json:"success"`
	Error            string                    `json:"error,omitempty"`
}

// ActiveStream tracks ongoing meal plan generation streams
type ActiveStream struct {
	StreamID   string                       `json:"streamId"`
	UserID     uuid.UUID                    `json:"userId"`
	StartTime  time.Time                    `json:"startTime"`
	LastUpdate time.Time                    `json:"lastUpdate"`
	Context    context.Context              `json:"-"`
	Cancel     context.CancelFunc           `json:"-"`
	Channel    chan *MealPlanProgressUpdate `json:"-"`
	Request    *StreamingMealPlanRequest    `json:"request"`
	Mutex      sync.RWMutex                 `json:"-"`
}

type streamingMealPlanService struct {
	optimizedRotation OptimizedRotationService
	rotationService   RotationService
	mealPlanCache     MealPlanCacheService
	cache             *CacheService

	// Stream management
	activeStreams   map[string]*ActiveStream
	streamsMutex    sync.RWMutex
	cleanupInterval time.Duration
}

func NewStreamingMealPlanService(
	optimizedRotation OptimizedRotationService,
	rotationService RotationService,
	mealPlanCache MealPlanCacheService,
	cache *CacheService,
) StreamingMealPlanService {
	service := &streamingMealPlanService{
		optimizedRotation: optimizedRotation,
		rotationService:   rotationService,
		mealPlanCache:     mealPlanCache,
		cache:             cache,
		activeStreams:     make(map[string]*ActiveStream),
		cleanupInterval:   5 * time.Minute,
	}

	// Start periodic cleanup
	go service.periodicCleanup()

	return service
}

// GenerateWithProgress generates a meal plan with real-time progress updates
func (s *streamingMealPlanService) GenerateWithProgress(ctx context.Context, req *StreamingMealPlanRequest) (<-chan *MealPlanProgressUpdate, error) {
	// Create stream
	streamCtx, cancel := context.WithCancel(ctx)

	stream := &ActiveStream{
		StreamID:   req.StreamID,
		UserID:     req.UserID,
		StartTime:  time.Now(),
		LastUpdate: time.Now(),
		Context:    streamCtx,
		Cancel:     cancel,
		Channel:    make(chan *MealPlanProgressUpdate, 100),
		Request:    req,
	}

	// Register stream
	s.streamsMutex.Lock()
	s.activeStreams[req.StreamID] = stream
	s.streamsMutex.Unlock()

	// Start generation in background
	go s.generateWithProgressTracking(stream)

	return stream.Channel, nil
}

// generateWithProgressTracking performs the actual generation with progress tracking
func (s *streamingMealPlanService) generateWithProgressTracking(stream *ActiveStream) {
	defer func() {
		close(stream.Channel)
		s.removeStream(stream.StreamID)
	}()

	startTime := time.Now()
	req := stream.Request

	// Send initial progress
	s.sendProgress(stream, &MealPlanProgressUpdate{
		StreamID:       req.StreamID,
		UserID:         req.UserID,
		Stage:          "initializing",
		Progress:       0.0,
		Message:        "Starting meal plan generation...",
		CurrentStep:    "initialization",
		StepsCompleted: 0,
		TotalSteps:     10,
		MealsGenerated: 0,
		TotalMeals:     21,
		ElapsedTime:    time.Since(startTime),
		Timestamp:      time.Now(),
	})

	// Check cache first
	s.sendProgress(stream, &MealPlanProgressUpdate{
		StreamID:       req.StreamID,
		UserID:         req.UserID,
		Stage:          "cache_check",
		Progress:       0.1,
		Message:        "Checking cache for existing meal plans...",
		CurrentStep:    "cache lookup",
		StepsCompleted: 1,
		TotalSteps:     10,
		ElapsedTime:    time.Since(startTime),
		Timestamp:      time.Now(),
	})

	// Try cache
	cacheKey := CreateMealPlanCacheKey(req.UserID, req.Preferences, req.WeeklyPatterns, 0, req.GenerationType)
	if cached, err := s.mealPlanCache.GetCachedMealPlan(stream.Context, cacheKey); err == nil {
		s.sendProgress(stream, &MealPlanProgressUpdate{
			StreamID:         req.StreamID,
			UserID:           req.UserID,
			Stage:            "completed",
			Progress:         1.0,
			Message:          "Retrieved meal plan from cache",
			CurrentStep:      "cache hit",
			StepsCompleted:   10,
			TotalSteps:       10,
			MealsGenerated:   21,
			TotalMeals:       21,
			ElapsedTime:      time.Since(startTime),
			Timestamp:        time.Now(),
			MealPlan:         cached.MealPlan,
			ConstraintReport: cached.ConstraintReport,
			Completed:        true,
		})
		return
	}

	// Cache miss - generate new meal plan
	s.sendProgress(stream, &MealPlanProgressUpdate{
		StreamID:       req.StreamID,
		UserID:         req.UserID,
		Stage:          "generation",
		Progress:       0.2,
		Message:        "Generating new meal plan...",
		CurrentStep:    "recipe pool loading",
		StepsCompleted: 2,
		TotalSteps:     10,
		ElapsedTime:    time.Since(startTime),
		Timestamp:      time.Now(),
	})

	// Set timeout if specified
	generationCtx := stream.Context
	if req.TimeoutSeconds > 0 {
		var timeoutCancel context.CancelFunc
		generationCtx, timeoutCancel = context.WithTimeout(stream.Context, time.Duration(req.TimeoutSeconds)*time.Second)
		defer timeoutCancel()
	}

	// Generate meal plan with progress simulation
	mealPlan, constraintReport, err := s.generateWithSimulatedProgress(generationCtx, stream, startTime)

	if err != nil {
		// Handle generation error
		if req.EnableFallbacks {
			s.sendProgress(stream, &MealPlanProgressUpdate{
				StreamID:       req.StreamID,
				UserID:         req.UserID,
				Stage:          "fallback",
				Progress:       0.8,
				Message:        "Primary generation failed, trying fallback...",
				CurrentStep:    "fallback generation",
				StepsCompleted: 8,
				TotalSteps:     10,
				ElapsedTime:    time.Since(startTime),
				Timestamp:      time.Now(),
			})

			// Try fallback generation
			fallbackResult, fallbackErr := s.generateFallback(generationCtx, req.UserID, req.Preferences)
			if fallbackErr == nil {
				mealPlan = fallbackResult.MealPlan
				constraintReport = fallbackResult.ConstraintReport
				err = nil
			} else {
				err = fmt.Errorf("primary generation failed: %v, fallback failed: %v", err, fallbackErr)
			}
		}

		if err != nil {
			s.sendProgress(stream, &MealPlanProgressUpdate{
				StreamID:    req.StreamID,
				UserID:      req.UserID,
				Stage:       "error",
				Progress:    0.0,
				Message:     "Meal plan generation failed",
				CurrentStep: "error handling",
				ElapsedTime: time.Since(startTime),
				Timestamp:   time.Now(),
				Error:       err.Error(),
				Completed:   true,
			})
			return
		}
	}

	// Cache the result
	s.sendProgress(stream, &MealPlanProgressUpdate{
		StreamID:       req.StreamID,
		UserID:         req.UserID,
		Stage:          "caching",
		Progress:       0.95,
		Message:        "Caching generated meal plan...",
		CurrentStep:    "cache storage",
		StepsCompleted: 9,
		TotalSteps:     10,
		MealsGenerated: 21,
		TotalMeals:     21,
		ElapsedTime:    time.Since(startTime),
		Timestamp:      time.Now(),
	})

	// Cache asynchronously
	go func() {
		ttl := 2 * time.Hour
		s.mealPlanCache.CacheMealPlan(context.Background(), cacheKey, mealPlan, constraintReport, ttl)
	}()

	// Send completion
	s.sendProgress(stream, &MealPlanProgressUpdate{
		StreamID:         req.StreamID,
		UserID:           req.UserID,
		Stage:            "completed",
		Progress:         1.0,
		Message:          fmt.Sprintf("Meal plan generated successfully in %v", time.Since(startTime)),
		CurrentStep:      "completion",
		StepsCompleted:   10,
		TotalSteps:       10,
		MealsGenerated:   21,
		TotalMeals:       21,
		ElapsedTime:      time.Since(startTime),
		Timestamp:        time.Now(),
		MealPlan:         mealPlan,
		ConstraintReport: constraintReport,
		Completed:        true,
	})
}

// generateWithSimulatedProgress generates a meal plan while simulating progress updates
func (s *streamingMealPlanService) generateWithSimulatedProgress(ctx context.Context, stream *ActiveStream, startTime time.Time) (*models.WeeklyMeals, *RotationConstraintReport, error) {
	req := stream.Request

	// Simulate progress for different stages
	stages := []struct {
		name        string
		message     string
		progress    float64
		minDuration time.Duration
	}{
		{"recipe_loading", "Loading available recipes...", 0.3, 100 * time.Millisecond},
		{"rotation_analysis", "Analyzing rotation patterns...", 0.4, 150 * time.Millisecond},
		{"meal_selection", "Selecting optimal meals...", 0.6, 300 * time.Millisecond},
		{"constraint_checking", "Checking dietary constraints...", 0.8, 100 * time.Millisecond},
		{"finalization", "Finalizing meal plan...", 0.9, 50 * time.Millisecond},
	}

	for i, stage := range stages {
		select {
		case <-ctx.Done():
			return nil, nil, ctx.Err()
		default:
		}

		s.sendProgress(stream, &MealPlanProgressUpdate{
			StreamID:               req.StreamID,
			UserID:                 req.UserID,
			Stage:                  stage.name,
			Progress:               stage.progress,
			Message:                stage.message,
			CurrentStep:            stage.name,
			StepsCompleted:         i + 3,
			TotalSteps:             10,
			MealsGenerated:         int(stage.progress * 21),
			TotalMeals:             21,
			ElapsedTime:            time.Since(startTime),
			EstimatedTimeRemaining: time.Duration(float64(time.Since(startTime)) * (1.0 - stage.progress) / stage.progress),
			Timestamp:              time.Now(),
		})

		// Simulate work
		time.Sleep(stage.minDuration)
	}

	// Actual generation
	switch req.GenerationType {
	case "pattern-aware":
		if len(req.WeeklyPatterns) > 0 {
			mealPlan, err := s.optimizedRotation.GenerateMealPlanWithPatternsFast(ctx, req.UserID, req.Preferences, req.WeeklyPatterns)
			return mealPlan, nil, err
		}
		fallthrough
	case "constraint-handling":
		mealPlan, report, err := s.optimizedRotation.GenerateMealPlanWithConstraintsFast(ctx, req.UserID, req.Preferences)
		return mealPlan, report, err
	default:
		mealPlan, err := s.optimizedRotation.GenerateMealPlanFast(ctx, req.UserID, req.Preferences)
		return mealPlan, nil, err
	}
}

// GenerateWithTimeout generates a meal plan with a strict timeout
func (s *streamingMealPlanService) GenerateWithTimeout(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, timeout time.Duration) (*models.WeeklyMeals, error) {
	timeoutCtx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	start := time.Now()
	mealPlan, err := s.optimizedRotation.GenerateMealPlanFast(timeoutCtx, userID, preferences)

	elapsed := time.Since(start)
	if elapsed > timeout {
		log.Printf("WARNING: Meal plan generation exceeded timeout: %v > %v", elapsed, timeout)
	}

	return mealPlan, err
}

// GenerateWithFallbacks generates a meal plan with comprehensive fallback strategies
func (s *streamingMealPlanService) GenerateWithFallbacks(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*MealPlanGenerationResult, error) {
	result := &MealPlanGenerationResult{
		FallbacksUsed: make([]string, 0),
		Success:       false,
	}

	startTime := time.Now()

	// Strategy 1: Try optimized fast generation
	mealPlan, err := s.optimizedRotation.GenerateMealPlanFast(ctx, userID, preferences)
	if err == nil {
		result.MealPlan = mealPlan
		result.GenerationTime = time.Since(startTime)
		result.Success = true
		return result, nil
	}

	result.FallbacksUsed = append(result.FallbacksUsed, "optimized_generation_failed")

	// Strategy 2: Try standard rotation service
	mealPlan, err = s.rotationService.SelectRecipesForWeek(userID, preferences)
	if err == nil {
		result.MealPlan = mealPlan
		result.GenerationTime = time.Since(startTime)
		result.FallbacksUsed = append(result.FallbacksUsed, "standard_rotation_service")
		result.Success = true
		return result, nil
	}

	result.FallbacksUsed = append(result.FallbacksUsed, "standard_generation_failed")

	// Strategy 3: Try with relaxed preferences
	relaxedPrefs := *preferences
	relaxedPrefs.MaxPrepTimePerMeal = relaxedPrefs.MaxPrepTimePerMeal * 2 // Double time allowance
	relaxedPrefs.DietaryRestrictions = nil                                // Remove dietary restrictions

	mealPlan, err = s.rotationService.SelectRecipesForWeek(userID, &relaxedPrefs)
	if err == nil {
		result.MealPlan = mealPlan
		result.GenerationTime = time.Since(startTime)
		result.FallbacksUsed = append(result.FallbacksUsed, "relaxed_preferences")
		result.Success = true
		return result, nil
	}

	result.FallbacksUsed = append(result.FallbacksUsed, "relaxed_generation_failed")
	result.Error = fmt.Sprintf("all fallback strategies failed: %v", err)
	result.GenerationTime = time.Since(startTime)

	return result, err
}

// generateFallback provides a simple fallback meal plan generation
func (s *streamingMealPlanService) generateFallback(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences) (*MealPlanGenerationResult, error) {
	result := &MealPlanGenerationResult{
		FallbacksUsed: []string{"simple_fallback"},
		Success:       false,
	}

	startTime := time.Now()

	// Simple fallback: use standard rotation service with minimal constraints
	fallbackPrefs := &models.UserPreferences{
		MaxPrepTimePerMeal: 60, // Allow up to 1 hour
		CookingSkillLevel:  "intermediate",
		FamilySize:         preferences.FamilySize,
	}

	mealPlan, err := s.rotationService.SelectRecipesForWeek(userID, fallbackPrefs)
	if err != nil {
		result.Error = err.Error()
		result.GenerationTime = time.Since(startTime)
		return result, err
	}

	result.MealPlan = mealPlan
	result.GenerationTime = time.Since(startTime)
	result.Success = true

	return result, nil
}

// Stream management methods

func (s *streamingMealPlanService) GetActiveStreams(userID uuid.UUID) []string {
	s.streamsMutex.RLock()
	defer s.streamsMutex.RUnlock()

	var streams []string
	for streamID, stream := range s.activeStreams {
		if stream.UserID == userID {
			streams = append(streams, streamID)
		}
	}

	return streams
}

func (s *streamingMealPlanService) CancelStream(streamID string) error {
	s.streamsMutex.Lock()
	defer s.streamsMutex.Unlock()

	if stream, exists := s.activeStreams[streamID]; exists {
		stream.Cancel()
		delete(s.activeStreams, streamID)
		return nil
	}

	return fmt.Errorf("stream not found: %s", streamID)
}

func (s *streamingMealPlanService) CleanupExpiredStreams() int {
	s.streamsMutex.Lock()
	defer s.streamsMutex.Unlock()

	expiredCount := 0
	now := time.Now()

	for streamID, stream := range s.activeStreams {
		// Clean up streams that haven't been updated in the last 10 minutes
		if now.Sub(stream.LastUpdate) > 10*time.Minute {
			stream.Cancel()
			delete(s.activeStreams, streamID)
			expiredCount++
		}
	}

	if expiredCount > 0 {
		log.Printf("Cleaned up %d expired meal plan generation streams", expiredCount)
	}

	return expiredCount
}

// Helper methods

func (s *streamingMealPlanService) sendProgress(stream *ActiveStream, update *MealPlanProgressUpdate) {
	stream.Mutex.Lock()
	stream.LastUpdate = time.Now()
	stream.Mutex.Unlock()

	select {
	case stream.Channel <- update:
		// Progress sent successfully
	case <-stream.Context.Done():
		// Stream was cancelled
		return
	default:
		// Channel is full, drop the update
		log.Printf("Warning: Progress channel full for stream %s, dropping update", stream.StreamID)
	}
}

func (s *streamingMealPlanService) removeStream(streamID string) {
	s.streamsMutex.Lock()
	defer s.streamsMutex.Unlock()

	if stream, exists := s.activeStreams[streamID]; exists {
		stream.Cancel()
		delete(s.activeStreams, streamID)
	}
}

func (s *streamingMealPlanService) periodicCleanup() {
	ticker := time.NewTicker(s.cleanupInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			s.CleanupExpiredStreams()
		}
	}
}
