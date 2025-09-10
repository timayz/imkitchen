package tests

import (
	"context"
	"encoding/json"
	"fmt"
	"runtime"
	"sort"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// Performance benchmark tests for the meal plan generation
// These tests validate the <2 second generation requirement

func BenchmarkMealPlanGeneration(b *testing.B) {
	// Setup test environment
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Pre-setup mocks to avoid timing interference
	setupPerformanceMocks(mockRecipeRepo, mockUserRepo, mockCache, userID, recipes)

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_, err := rotationService.SelectRecipesForWeek(userID, preferences)
		if err != nil {
			b.Fatalf("Generation failed: %v", err)
		}
	}
}

func TestMealPlanGeneration_PerformanceRequirements(t *testing.T) {
	tests := []struct {
		name                 string
		recipeCount         int
		userCount           int
		expectedMaxDuration time.Duration
		description         string
	}{
		{
			name:                "Single User - Optimal Conditions",
			recipeCount:        50,
			userCount:          1,
			expectedMaxDuration: 2 * time.Second,
			description:        "Single user with 50 recipes should complete in <2s",
		},
		{
			name:                "Single User - Large Recipe Pool",
			recipeCount:        500,
			userCount:          1,
			expectedMaxDuration: 2 * time.Second,
			description:        "Single user with 500 recipes should still complete in <2s",
		},
		{
			name:                "Multiple Users - Concurrent Generation",
			recipeCount:        100,
			userCount:          5,
			expectedMaxDuration: 3 * time.Second,
			description:        "5 concurrent users should complete within 3s",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockRecipeRepo := new(MockRecipeRepository)
			mockUserRepo := new(MockUserRepository)
			mockCache := new(MockCacheService)

			rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

			// Generate test data
			recipes := generateLargeRecipeSet(tt.recipeCount)
			userPreferences := createTestUserPreferences()

			var wg sync.WaitGroup
			results := make([]time.Duration, tt.userCount)
			errors := make([]error, tt.userCount)

			startTime := time.Now()

			// Run concurrent generations
			for i := 0; i < tt.userCount; i++ {
				wg.Add(1)
				go func(userIndex int) {
					defer wg.Done()

					userID := uuid.New()
					setupPerformanceMocks(mockRecipeRepo, mockUserRepo, mockCache, userID, recipes)

					userStartTime := time.Now()
					_, err := rotationService.SelectRecipesForWeek(userID, userPreferences)
					results[userIndex] = time.Since(userStartTime)
					errors[userIndex] = err
				}(i)
			}

			wg.Wait()
			totalDuration := time.Since(startTime)

			// Validate results
			for i, err := range errors {
				assert.NoError(t, err, "User %d generation failed", i)
			}

			// Check individual user performance
			for i, duration := range results {
				if tt.userCount == 1 {
					assert.True(t, duration < tt.expectedMaxDuration,
						"User %d took %v, expected <%v", i, duration, tt.expectedMaxDuration)
				}
			}

			// Check overall performance for concurrent users
			if tt.userCount > 1 {
				assert.True(t, totalDuration < tt.expectedMaxDuration,
					"Concurrent generation took %v, expected <%v", totalDuration, tt.expectedMaxDuration)
			}

			// Report metrics
			t.Logf("%s:", tt.description)
			t.Logf("  Total Duration: %v", totalDuration)
			if tt.userCount == 1 {
				t.Logf("  Generation Time: %v", results[0])
			} else {
				avgDuration := time.Duration(0)
				maxDuration := time.Duration(0)
				for _, duration := range results {
					avgDuration += duration
					if duration > maxDuration {
						maxDuration = duration
					}
				}
				avgDuration /= time.Duration(len(results))
				t.Logf("  Average User Time: %v", avgDuration)
				t.Logf("  Max User Time: %v", maxDuration)
			}
		})
	}
}

func TestMealPlanGeneration_StressTest(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping stress test in short mode")
	}

	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	// Stress test parameters
	const (
		concurrentUsers = 10
		generationsPerUser = 5
		recipePoolSize = 200
	)

	recipes := generateLargeRecipeSet(recipePoolSize)
	userPreferences := createTestUserPreferences()

	var wg sync.WaitGroup
	results := make(chan time.Duration, concurrentUsers*generationsPerUser)
	errorCount := int32(0)

	startTime := time.Now()

	// Launch concurrent users
	for i := 0; i < concurrentUsers; i++ {
		wg.Add(1)
		go func(userIndex int) {
			defer wg.Done()

			userID := uuid.New()
			setupPerformanceMocks(mockRecipeRepo, mockUserRepo, mockCache, userID, recipes)

			// Each user generates multiple meal plans
			for j := 0; j < generationsPerUser; j++ {
				genStartTime := time.Now()
				_, err := rotationService.SelectRecipesForWeek(userID, userPreferences)
				duration := time.Since(genStartTime)
				results <- duration

				if err != nil {
					t.Errorf("Generation failed for user %d, iteration %d: %v", userIndex, j, err)
					continue
				}

				// Brief pause between generations
				time.Sleep(100 * time.Millisecond)
			}
		}(i)
	}

	wg.Wait()
	close(results)
	totalDuration := time.Since(startTime)

	// Analyze results
	var durations []time.Duration
	totalRequests := 0
	under2s := 0
	under5s := 0

	for duration := range results {
		durations = append(durations, duration)
		totalRequests++

		if duration < 2*time.Second {
			under2s++
		}
		if duration < 5*time.Second {
			under5s++
		}
	}

	// Performance assertions
	assert.Equal(t, concurrentUsers*generationsPerUser, totalRequests)
	
	// At least 80% should complete under 2 seconds
	under2sPercentage := float64(under2s) / float64(totalRequests) * 100
	assert.True(t, under2sPercentage >= 80.0, 
		"Only %f%% completed under 2s, expected >=80%%", under2sPercentage)

	// 95% should complete under 5 seconds
	under5sPercentage := float64(under5s) / float64(totalRequests) * 100
	assert.True(t, under5sPercentage >= 95.0,
		"Only %f%% completed under 5s, expected >=95%%", under5sPercentage)

	// Report detailed metrics
	t.Logf("Stress Test Results:")
	t.Logf("  Total Duration: %v", totalDuration)
	t.Logf("  Total Requests: %d", totalRequests)
	t.Logf("  Under 2s: %d (%.1f%%)", under2s, under2sPercentage)
	t.Logf("  Under 5s: %d (%.1f%%)", under5s, under5sPercentage)
	t.Logf("  Concurrent Users: %d", concurrentUsers)
	t.Logf("  Generations per User: %d", generationsPerUser)
	t.Logf("  Recipe Pool Size: %d", recipePoolSize)

	if len(durations) > 0 {
		// Calculate percentiles
		sort.Slice(durations, func(i, j int) bool {
			return durations[i] < durations[j]
		})

		p50 := durations[len(durations)*50/100]
		p95 := durations[len(durations)*95/100]
		p99 := durations[len(durations)*99/100]

		t.Logf("  Performance Percentiles:")
		t.Logf("    50th percentile: %v", p50)
		t.Logf("    95th percentile: %v", p95)
		t.Logf("    99th percentile: %v", p99)
	}
}

func TestMealPlanGeneration_MemoryUsage(t *testing.T) {
	// This test monitors memory usage during generation to ensure no memory leaks
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := generateLargeRecipeSet(100)

	setupPerformanceMocks(mockRecipeRepo, mockUserRepo, mockCache, userID, recipes)

	// Force garbage collection before test
	var m1, m2 runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&m1)

	// Run multiple generations
	for i := 0; i < 50; i++ {
		_, err := rotationService.SelectRecipesForWeek(userID, preferences)
		assert.NoError(t, err)
	}

	// Check memory after generations
	runtime.GC()
	runtime.ReadMemStats(&m2)

	memIncrease := int64(m2.Alloc - m1.Alloc)
	t.Logf("Memory usage increase: %d bytes", memIncrease)

	// Memory increase should be reasonable (less than 10MB for 50 generations)
	assert.True(t, memIncrease < 10*1024*1024, 
		"Memory increase too high: %d bytes", memIncrease)
}

// Helper functions for performance tests

func setupPerformanceMocks(mockRecipeRepo *MockRecipeRepository, mockUserRepo *MockUserRepository, 
	mockCache *MockCacheService, userID uuid.UUID, recipes []models.Recipe) {

	// Setup mocks for fast execution
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)
}

func generateLargeRecipeSet(count int) []models.Recipe {
	recipes := make([]models.Recipe, count)
	
	mealTypes := [][]string{
		{"breakfast"},
		{"lunch"},
		{"dinner"},
		{"lunch", "dinner"},
		{"breakfast", "lunch"},
	}
	complexities := []string{"simple", "moderate", "complex"}
	cuisines := []string{"italian", "french", "mexican", "asian", "american", "mediterranean", "indian", "japanese", "thai"}

	for i := 0; i < count; i++ {
		recipes[i] = models.Recipe{
			ID:          uuid.New(),
			Title:       fmt.Sprintf("Performance Test Recipe %d", i+1),
			PrepTime:    5 + (i % 55),     // 5-60 minutes prep
			CookTime:    10 + (i % 50),    // 10-60 minutes cook
			MealType:    mealTypes[i%len(mealTypes)],
			Complexity:  complexities[i%len(complexities)],
			CuisineType: stringPtr(cuisines[i%len(cuisines)]),
			Servings:    1 + (i % 6),      // 1-6 servings
			AverageRating: 3.0 + (float64(i%21)/10.0), // 3.0-5.0 rating
		}
	}

	return recipes
}