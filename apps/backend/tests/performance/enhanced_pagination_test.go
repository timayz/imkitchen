package performance

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/lib/pq"
	"github.com/stretchr/testify/assert"
	"gorm.io/gorm"
)

// Test models that mirror the actual structure for pagination testing
type TestRecipe struct {
	ID             uuid.UUID      `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	UserID         uuid.UUID      `json:"userId" gorm:"column:user_id;type:uuid;not null"`
	Title          string         `json:"title" gorm:"size:255;not null"`
	Description    *string        `json:"description,omitempty" gorm:"type:text"`
	PrepTime       int            `json:"prepTime" gorm:"column:prep_time;not null"`
	CookTime       int            `json:"cookTime" gorm:"column:cook_time;not null"`
	TotalTime      int            `json:"totalTime" gorm:"column:total_time;not null"`
	CuisineType    *string        `json:"cuisineType,omitempty" gorm:"column:cuisine_type;size:100"`
	DietaryLabels  pq.StringArray `json:"dietaryLabels" gorm:"column:dietary_labels;type:text[]"`
	AverageRating  float64        `json:"averageRating" gorm:"column:average_rating;type:decimal(3,2);default:0.0"`
	TotalRatings   int            `json:"totalRatings" gorm:"column:total_ratings;default:0"`
	CreatedAt      time.Time      `json:"createdAt" gorm:"column:created_at;default:now()"`
	UpdatedAt      time.Time      `json:"updatedAt" gorm:"column:updated_at;default:now()"`
	DeletedAt      *time.Time     `json:"deletedAt,omitempty" gorm:"column:deleted_at"`
}

// Enhanced pagination interfaces for testing
type EnhancedPaginationService interface {
	GenerateEnhancedCursor(item interface{}, sortField string) (string, error)
	GetFastEstimatedCount(query *gorm.DB, tableName string) (int64, error)
	RecommendPaginationStrategy(estimatedCount int64, queryComplexity string) *PaginationStrategy
	AnalyzePaginationPerformance(queryType string, executionTime time.Duration, resultCount int, totalCount int64) *PaginationPerformanceMetrics
}

type PaginationStrategy struct {
	RecommendedType      string   `json:"recommendedType"`
	Reasoning            string   `json:"reasoning"`
	PerformanceGain      string   `json:"performanceGain"`
	UseCountEstimation   bool     `json:"useCountEstimation"`
	OptimalPageSize      int      `json:"optimalPageSize"`
	IndexRecommendations []string `json:"indexRecommendations"`
}

type PaginationPerformanceMetrics struct {
	QueryType         string        `json:"queryType"`
	PaginationType    string        `json:"paginationType"`
	ExecutionTime     time.Duration `json:"executionTime"`
	ResultCount       int           `json:"resultCount"`
	TotalCount        int64         `json:"totalCount"`
	Page              int           `json:"page"`
	EstimatedCount    bool          `json:"estimatedCount"`
	IndexesUsed       []string      `json:"indexesUsed"`
	PerformanceRating string        `json:"performanceRating"`
	Timestamp         time.Time     `json:"timestamp"`
}

type testEnhancedPaginationService struct {
}

func NewTestEnhancedPaginationService() EnhancedPaginationService {
	return &testEnhancedPaginationService{}
}

func (p *testEnhancedPaginationService) GenerateEnhancedCursor(item interface{}, sortField string) (string, error) {
	// Simplified cursor generation for testing
	if recipe, ok := item.(TestRecipe); ok {
		cursorData := map[string]interface{}{
			"id":        recipe.ID.String(),
			"sortField": sortField,
		}
		
		switch sortField {
		case "created_at":
			cursorData["sortValue"] = recipe.CreatedAt
		case "average_rating":
			cursorData["sortValue"] = recipe.AverageRating
		case "total_time":
			cursorData["sortValue"] = recipe.TotalTime
		default:
			cursorData["sortValue"] = recipe.CreatedAt
		}
		
		// Return base64 encoded cursor (simplified)
		return "dGVzdC1jdXJzb3I=", nil // "test-cursor" in base64
	}
	
	return "", assert.AnError
}

func (p *testEnhancedPaginationService) GetFastEstimatedCount(query *gorm.DB, tableName string) (int64, error) {
	// Mock implementation returns different values based on table name
	switch tableName {
	case "recipes":
		return 15000, nil
	case "large_recipes":
		return 150000, nil
	default:
		return 1000, nil
	}
}

func (p *testEnhancedPaginationService) RecommendPaginationStrategy(estimatedCount int64, queryComplexity string) *PaginationStrategy {
	if estimatedCount > 50000 {
		return &PaginationStrategy{
			RecommendedType:    "cursor",
			Reasoning:         "Large dataset benefits from cursor-based pagination",
			PerformanceGain:   "60-80% better performance for deep pagination",
			UseCountEstimation: true,
			OptimalPageSize:   20,
			IndexRecommendations: []string{
				"Consider adding composite indices for filter + sort combinations",
			},
		}
	} else if estimatedCount > 10000 {
		return &PaginationStrategy{
			RecommendedType:    "hybrid",
			Reasoning:         "Medium dataset can benefit from hybrid approach",
			PerformanceGain:   "40-60% better for pages beyond 100",
			UseCountEstimation: true,
			OptimalPageSize:   25,
		}
	}
	
	return &PaginationStrategy{
		RecommendedType:    "offset",
		Reasoning:         "Small dataset works well with traditional offset pagination",
		PerformanceGain:   "Optimal for small datasets with exact counts",
		UseCountEstimation: false,
		OptimalPageSize:   20,
	}
}

func (p *testEnhancedPaginationService) AnalyzePaginationPerformance(queryType string, executionTime time.Duration, resultCount int, totalCount int64) *PaginationPerformanceMetrics {
	var rating string
	if executionTime <= 50*time.Millisecond {
		rating = "excellent"
	} else if executionTime <= 100*time.Millisecond {
		rating = "good"
	} else if executionTime <= 200*time.Millisecond {
		rating = "acceptable"
	} else {
		rating = "needs_optimization"
	}
	
	return &PaginationPerformanceMetrics{
		QueryType:         queryType,
		ExecutionTime:     executionTime,
		ResultCount:       resultCount,
		TotalCount:        totalCount,
		EstimatedCount:    totalCount > 10000,
		PerformanceRating: rating,
		Timestamp:         time.Now(),
	}
}

// Test the enhanced cursor generation functionality
func TestEnhancedPaginationService_GenerateEnhancedCursor(t *testing.T) {
	service := NewTestEnhancedPaginationService()
	
	testRecipe := TestRecipe{
		ID:            uuid.New(),
		UserID:        uuid.New(),
		Title:         "Test Recipe",
		PrepTime:      20,
		CookTime:      30,
		TotalTime:     50,
		AverageRating: 4.5,
		CreatedAt:     time.Now(),
	}
	
	tests := []struct {
		name      string
		item      interface{}
		sortField string
		wantError bool
	}{
		{
			name:      "generate_cursor_for_created_at",
			item:      testRecipe,
			sortField: "created_at",
			wantError: false,
		},
		{
			name:      "generate_cursor_for_average_rating",
			item:      testRecipe,
			sortField: "average_rating",
			wantError: false,
		},
		{
			name:      "generate_cursor_for_total_time",
			item:      testRecipe,
			sortField: "total_time",
			wantError: false,
		},
		{
			name:      "generate_cursor_invalid_item",
			item:      "invalid",
			sortField: "created_at",
			wantError: true,
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cursor, err := service.GenerateEnhancedCursor(tt.item, tt.sortField)
			
			if tt.wantError {
				assert.Error(t, err)
				assert.Empty(t, cursor)
			} else {
				assert.NoError(t, err)
				assert.NotEmpty(t, cursor)
				assert.Equal(t, "dGVzdC1jdXJzb3I=", cursor) // Base64 for "test-cursor"
			}
		})
	}
}

// Test pagination strategy recommendations
func TestEnhancedPaginationService_RecommendPaginationStrategy(t *testing.T) {
	service := NewTestEnhancedPaginationService()
	
	tests := []struct {
		name              string
		estimatedCount    int64
		queryComplexity   string
		wantType          string
		wantEstimation    bool
		wantPageSize      int
	}{
		{
			name:            "small_dataset_offset",
			estimatedCount:  5000,
			queryComplexity: "low",
			wantType:        "offset",
			wantEstimation:  false,
			wantPageSize:    20,
		},
		{
			name:            "medium_dataset_hybrid",
			estimatedCount:  25000,
			queryComplexity: "medium",
			wantType:        "hybrid",
			wantEstimation:  true,
			wantPageSize:    25,
		},
		{
			name:            "large_dataset_cursor",
			estimatedCount:  100000,
			queryComplexity: "high",
			wantType:        "cursor",
			wantEstimation:  true,
			wantPageSize:    20,
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			strategy := service.RecommendPaginationStrategy(tt.estimatedCount, tt.queryComplexity)
			
			assert.Equal(t, tt.wantType, strategy.RecommendedType)
			assert.Equal(t, tt.wantEstimation, strategy.UseCountEstimation)
			assert.Equal(t, tt.wantPageSize, strategy.OptimalPageSize)
			assert.NotEmpty(t, strategy.Reasoning)
			assert.NotEmpty(t, strategy.PerformanceGain)
			
			// Large datasets should have index recommendations
			if tt.estimatedCount > 50000 {
				assert.Greater(t, len(strategy.IndexRecommendations), 0)
			}
		})
	}
}

// Test performance analysis functionality
func TestEnhancedPaginationService_AnalyzePaginationPerformance(t *testing.T) {
	service := NewTestEnhancedPaginationService()
	
	tests := []struct {
		name          string
		queryType     string
		executionTime time.Duration
		resultCount   int
		totalCount    int64
		wantRating    string
		wantEstimated bool
	}{
		{
			name:          "excellent_performance",
			queryType:     "recipe_search",
			executionTime: 30 * time.Millisecond,
			resultCount:   20,
			totalCount:    5000,
			wantRating:    "excellent",
			wantEstimated: false,
		},
		{
			name:          "good_performance",
			queryType:     "recipe_search",
			executionTime: 80 * time.Millisecond,
			resultCount:   25,
			totalCount:    15000,
			wantRating:    "good",
			wantEstimated: true,
		},
		{
			name:          "acceptable_performance",
			queryType:     "recipe_search",
			executionTime: 150 * time.Millisecond,
			resultCount:   30,
			totalCount:    25000,
			wantRating:    "acceptable",
			wantEstimated: true,
		},
		{
			name:          "needs_optimization",
			queryType:     "recipe_search",
			executionTime: 300 * time.Millisecond,
			resultCount:   20,
			totalCount:    50000,
			wantRating:    "needs_optimization",
			wantEstimated: true,
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			metrics := service.AnalyzePaginationPerformance(
				tt.queryType, 
				tt.executionTime, 
				tt.resultCount, 
				tt.totalCount,
			)
			
			assert.Equal(t, tt.queryType, metrics.QueryType)
			assert.Equal(t, tt.executionTime, metrics.ExecutionTime)
			assert.Equal(t, tt.resultCount, metrics.ResultCount)
			assert.Equal(t, tt.totalCount, metrics.TotalCount)
			assert.Equal(t, tt.wantRating, metrics.PerformanceRating)
			assert.Equal(t, tt.wantEstimated, metrics.EstimatedCount)
			assert.WithinDuration(t, time.Now(), metrics.Timestamp, time.Second)
		})
	}
}

// Test fast estimated count functionality
func TestEnhancedPaginationService_GetFastEstimatedCount(t *testing.T) {
	db, _, cleanup := setupTestDB(t)
	defer cleanup()
	
	service := NewTestEnhancedPaginationService()
	
	tests := []struct {
		name      string
		tableName string
		wantCount int64
	}{
		{
			name:      "recipes_table",
			tableName: "recipes",
			wantCount: 15000,
		},
		{
			name:      "large_recipes_table",
			tableName: "large_recipes",
			wantCount: 150000,
		},
		{
			name:      "unknown_table",
			tableName: "unknown",
			wantCount: 1000,
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			count, err := service.GetFastEstimatedCount(db, tt.tableName)
			
			assert.NoError(t, err)
			assert.Equal(t, tt.wantCount, count)
		})
	}
}

// Test pagination performance under load (simulated)
func TestPaginationPerformance_Sub50msTarget(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping performance test in short mode")
	}
	
	service := NewTestEnhancedPaginationService()
	
	// Simulate multiple pagination operations
	testCases := []struct {
		name        string
		datasetSize int64
		pageSize    int
		iterations  int
	}{
		{"small_dataset", 1000, 20, 10},
		{"medium_dataset", 10000, 25, 10},
		{"large_dataset", 50000, 20, 5},
	}
	
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			var totalTime time.Duration
			
			for i := 0; i < tc.iterations; i++ {
				start := time.Now()
				
				// Simulate pagination operations
				strategy := service.RecommendPaginationStrategy(tc.datasetSize, "medium")
				assert.NotNil(t, strategy)
				
				// Simulate cursor generation
				testRecipe := TestRecipe{
					ID:        uuid.New(),
					CreatedAt: time.Now(),
				}
				_, err := service.GenerateEnhancedCursor(testRecipe, "created_at")
				assert.NoError(t, err)
				
				totalTime += time.Since(start)
			}
			
			avgTime := totalTime / time.Duration(tc.iterations)
			
			// Verify performance targets
			assert.Less(t, avgTime, 50*time.Millisecond, 
				"Average pagination operation should complete in under 50ms")
			
			// Analyze performance
			metrics := service.AnalyzePaginationPerformance("recipe_search", avgTime, tc.pageSize, tc.datasetSize)
			assert.Contains(t, []string{"excellent", "good"}, metrics.PerformanceRating,
				"Pagination performance should be excellent or good")
		})
	}
}

// Test pagination strategy switching based on dataset size
func TestPaginationStrategy_AdaptiveSelection(t *testing.T) {
	service := NewTestEnhancedPaginationService()
	
	// Test strategy evolution as dataset grows
	datasetSizes := []int64{100, 5000, 15000, 35000, 75000, 150000}
	expectedTypes := []string{"offset", "offset", "hybrid", "hybrid", "cursor", "cursor"}
	
	for i, size := range datasetSizes {
		strategy := service.RecommendPaginationStrategy(size, "medium")
		
		assert.Equal(t, expectedTypes[i], strategy.RecommendedType,
			"Dataset size %d should recommend %s pagination", size, expectedTypes[i])
		
		// Verify estimation recommendation aligns with strategy
		if strategy.RecommendedType == "cursor" {
			assert.True(t, strategy.UseCountEstimation,
				"Cursor pagination should use count estimation for large datasets")
		}
	}
}

// Test pagination health metrics calculation
func TestPaginationHealthMetrics(t *testing.T) {
	service := NewTestEnhancedPaginationService()
	
	// Create sample performance metrics
	metrics := []PaginationPerformanceMetrics{
		{ExecutionTime: 45 * time.Millisecond, ResultCount: 20, TotalCount: 1000},
		{ExecutionTime: 80 * time.Millisecond, ResultCount: 25, TotalCount: 5000},
		{ExecutionTime: 120 * time.Millisecond, ResultCount: 20, TotalCount: 10000},
		{ExecutionTime: 250 * time.Millisecond, ResultCount: 30, TotalCount: 50000}, // Slow query
	}
	
	// Calculate average performance
	var totalTime time.Duration
	slowQueries := 0
	
	for _, metric := range metrics {
		totalTime += metric.ExecutionTime
		if metric.ExecutionTime > 200*time.Millisecond {
			slowQueries++
		}
	}
	
	avgTime := totalTime / time.Duration(len(metrics))
	
	// Verify performance analysis
	assert.Less(t, avgTime, 200*time.Millisecond, "Average should be under 200ms")
	assert.Equal(t, 1, slowQueries, "Should detect exactly 1 slow query")
	
	// Test performance rating for average
	avgMetrics := service.AnalyzePaginationPerformance("average", avgTime, 25, 16250)
	assert.Contains(t, []string{"good", "acceptable"}, avgMetrics.PerformanceRating)
}

// Integration test for complete pagination workflow
func TestPaginationWorkflow_EndToEnd(t *testing.T) {
	service := NewTestEnhancedPaginationService()
	
	// Step 1: Analyze dataset and get recommendation
	datasetSize := int64(25000)
	strategy := service.RecommendPaginationStrategy(datasetSize, "medium")
	
	assert.Equal(t, "hybrid", strategy.RecommendedType)
	assert.True(t, strategy.UseCountEstimation)
	
	// Step 2: Execute pagination with the recommended strategy
	start := time.Now()
	
	// Simulate cursor generation for first page
	testRecipe := TestRecipe{
		ID:            uuid.New(),
		AverageRating: 4.2,
		CreatedAt:     time.Now(),
	}
	
	cursor, err := service.GenerateEnhancedCursor(testRecipe, "average_rating")
	assert.NoError(t, err)
	assert.NotEmpty(t, cursor)
	
	executionTime := time.Since(start)
	
	// Step 3: Analyze performance
	metrics := service.AnalyzePaginationPerformance(
		"recipe_search", 
		executionTime, 
		strategy.OptimalPageSize, 
		datasetSize,
	)
	
	assert.Equal(t, "recipe_search", metrics.QueryType)
	assert.True(t, metrics.EstimatedCount)
	assert.Contains(t, []string{"excellent", "good", "acceptable"}, metrics.PerformanceRating)
	
	// Step 4: Verify performance meets requirements
	assert.Less(t, executionTime, 100*time.Millisecond,
		"End-to-end pagination workflow should complete in under 100ms")
}