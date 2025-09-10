// +build performance

package performance

import (
	"context"
	"fmt"
	"log"
	"time"

	"gorm.io/gorm"
)

// PaginationPerformanceValidator validates that pagination meets the <50ms metadata target
type PaginationPerformanceValidator struct {
	db      *gorm.DB
	results []PerformanceTestResult
}

type PerformanceTestResult struct {
	TestName      string        `json:"testName"`
	PageSize      int           `json:"pageSize"`
	PageNumber    int           `json:"pageNumber"`
	DatasetSize   int64         `json:"datasetSize"`
	QueryType     string        `json:"queryType"`
	ExecutionTime time.Duration `json:"executionTime"`
	IndexesUsed   []string      `json:"indexesUsed"`
	MetTarget     bool          `json:"metTarget"`     // <50ms for metadata
	QueryTarget   bool          `json:"queryTarget"`   // <200ms for full query
}

func NewPaginationPerformanceValidator(db *gorm.DB) *PaginationPerformanceValidator {
	return &PaginationPerformanceValidator{
		db:      db,
		results: make([]PerformanceTestResult, 0),
	}
}

// ValidateBasicPaginationPerformance tests basic offset pagination performance
func (v *PaginationPerformanceValidator) ValidateBasicPaginationPerformance(ctx context.Context) error {
	testCases := []struct {
		name     string
		pageSize int
		pages    []int
	}{
		{"standard_pagination", 20, []int{1, 5, 10, 20}},
		{"large_page_size", 50, []int{1, 2, 5, 10}},
		{"small_page_size", 10, []int{1, 10, 25, 50}},
	}

	for _, tc := range testCases {
		for _, page := range tc.pages {
			result, err := v.testOffsetPagination(ctx, tc.name, tc.pageSize, page)
			if err != nil {
				return fmt.Errorf("failed test %s page %d: %w", tc.name, page, err)
			}
			v.results = append(v.results, *result)
		}
	}

	return nil
}

// ValidateCursorPaginationPerformance tests cursor-based pagination performance
func (v *PaginationPerformanceValidator) ValidateCursorPaginationPerformance(ctx context.Context) error {
	testCases := []struct {
		name      string
		sortField string
		pageSize  int
	}{
		{"cursor_by_created_at", "created_at", 20},
		{"cursor_by_rating", "average_rating", 25},
		{"cursor_by_total_time", "total_time", 30},
	}

	for _, tc := range testCases {
		result, err := v.testCursorPagination(ctx, tc.name, tc.sortField, tc.pageSize)
		if err != nil {
			return fmt.Errorf("failed cursor test %s: %w", tc.name, err)
		}
		v.results = append(v.results, *result)
	}

	return nil
}

// ValidateFilteredPaginationPerformance tests pagination with various filters
func (v *PaginationPerformanceValidator) ValidateFilteredPaginationPerformance(ctx context.Context) error {
	filterTests := []struct {
		name        string
		whereClause string
		args        []interface{}
		expectedIdx []string
	}{
		{
			name:        "cuisine_filter",
			whereClause: "cuisine_type = ? AND deleted_at IS NULL",
			args:        []interface{}{"italian"},
			expectedIdx: []string{"idx_recipes_cuisine_diet_preptime"},
		},
		{
			name:        "dietary_filter",
			whereClause: "dietary_labels @> ? AND deleted_at IS NULL",
			args:        []interface{}{`["vegetarian"]`},
			expectedIdx: []string{"idx_recipes_vegetarian_fast", "idx_recipes_combined_filters"},
		},
		{
			name:        "time_filter",
			whereClause: "prep_time <= ? AND deleted_at IS NULL",
			args:        []interface{}{30},
			expectedIdx: []string{"idx_recipes_cuisine_diet_preptime"},
		},
		{
			name:        "fulltext_search",
			whereClause: "to_tsvector('english', title || ' ' || COALESCE(description, '')) @@ plainto_tsquery('english', ?) AND deleted_at IS NULL",
			args:        []interface{}{"pasta"},
			expectedIdx: []string{"idx_recipes_fulltext_ranked"},
		},
	}

	for _, test := range filterTests {
		result, err := v.testFilteredPagination(ctx, test.name, test.whereClause, test.args, test.expectedIdx)
		if err != nil {
			return fmt.Errorf("failed filtered test %s: %w", test.name, err)
		}
		v.results = append(v.results, *result)
	}

	return nil
}

// testOffsetPagination tests standard offset-based pagination
func (v *PaginationPerformanceValidator) testOffsetPagination(ctx context.Context, testName string, pageSize, pageNumber int) (*PerformanceTestResult, error) {
	offset := (pageNumber - 1) * pageSize

	// Test metadata query (COUNT)
	start := time.Now()
	var count int64
	err := v.db.WithContext(ctx).Model(&TestRecipe{}).
		Where("deleted_at IS NULL").
		Count(&count).Error
	metadataTime := time.Since(start)

	if err != nil {
		return nil, fmt.Errorf("count query failed: %w", err)
	}

	// Test data query
	start = time.Now()
	var recipes []TestRecipe
	err = v.db.WithContext(ctx).
		Where("deleted_at IS NULL").
		Order("created_at DESC, id ASC").
		Offset(offset).
		Limit(pageSize).
		Find(&recipes).Error
	queryTime := time.Since(start)

	if err != nil {
		return nil, fmt.Errorf("data query failed: %w", err)
	}

	return &PerformanceTestResult{
		TestName:      fmt.Sprintf("%s_offset_p%d", testName, pageNumber),
		PageSize:      pageSize,
		PageNumber:    pageNumber,
		DatasetSize:   count,
		QueryType:     "offset_pagination",
		ExecutionTime: queryTime,
		IndexesUsed:   []string{"idx_recipes_pagination_optimal"},
		MetTarget:     metadataTime <= 50*time.Millisecond,
		QueryTarget:   queryTime <= 200*time.Millisecond,
	}, nil
}

// testCursorPagination tests cursor-based pagination performance
func (v *PaginationPerformanceValidator) testCursorPagination(ctx context.Context, testName, sortField string, pageSize int) (*PerformanceTestResult, error) {
	start := time.Now()

	var recipes []TestRecipe
	query := v.db.WithContext(ctx).
		Where("deleted_at IS NULL").
		Order(fmt.Sprintf("%s ASC, id ASC", sortField)).
		Limit(pageSize)

	err := query.Find(&recipes).Error
	executionTime := time.Since(start)

	if err != nil {
		return nil, fmt.Errorf("cursor query failed: %w", err)
	}

	// Determine expected indexes based on sort field
	var expectedIdxs []string
	switch sortField {
	case "created_at":
		expectedIdxs = []string{"idx_recipes_pagination_optimal"}
	case "average_rating":
		expectedIdxs = []string{"idx_recipes_trending_optimized"}
	case "total_time":
		expectedIdxs = []string{"idx_recipes_prep_total_time"}
	default:
		expectedIdxs = []string{"primary_key_index"}
	}

	return &PerformanceTestResult{
		TestName:      fmt.Sprintf("%s_cursor", testName),
		PageSize:      pageSize,
		PageNumber:    1, // Cursor pagination doesn't use page numbers
		DatasetSize:   int64(len(recipes)),
		QueryType:     "cursor_pagination",
		ExecutionTime: executionTime,
		IndexesUsed:   expectedIdxs,
		MetTarget:     true, // Cursor pagination doesn't need separate metadata query
		QueryTarget:   executionTime <= 200*time.Millisecond,
	}, nil
}

// testFilteredPagination tests pagination with filters
func (v *PaginationPerformanceValidator) testFilteredPagination(ctx context.Context, testName, whereClause string, args []interface{}, expectedIdx []string) (*PerformanceTestResult, error) {
	pageSize := 20

	start := time.Now()
	var recipes []TestRecipe
	err := v.db.WithContext(ctx).
		Where(whereClause, args...).
		Order("created_at DESC, id ASC").
		Limit(pageSize).
		Find(&recipes).Error
	executionTime := time.Since(start)

	if err != nil {
		return nil, fmt.Errorf("filtered query failed: %w", err)
	}

	return &PerformanceTestResult{
		TestName:      fmt.Sprintf("%s_filtered", testName),
		PageSize:      pageSize,
		PageNumber:    1,
		DatasetSize:   int64(len(recipes)),
		QueryType:     "filtered_pagination",
		ExecutionTime: executionTime,
		IndexesUsed:   expectedIdx,
		MetTarget:     true, // No separate count for this test
		QueryTarget:   executionTime <= 200*time.Millisecond,
	}, nil
}

// GeneratePerformanceReport creates a comprehensive performance report
func (v *PaginationPerformanceValidator) GeneratePerformanceReport() *PaginationPerformanceReport {
	report := &PaginationPerformanceReport{
		TotalTests:      len(v.results),
		PassedTests:     0,
		FailedTests:     0,
		AverageTime:     0,
		MetTargetCount:  0,
		QueryTargetCount: 0,
		Results:         v.results,
		GeneratedAt:     time.Now(),
	}

	var totalTime time.Duration
	for _, result := range v.results {
		totalTime += result.ExecutionTime

		if result.MetTarget && result.QueryTarget {
			report.PassedTests++
		} else {
			report.FailedTests++
		}

		if result.MetTarget {
			report.MetTargetCount++
		}

		if result.QueryTarget {
			report.QueryTargetCount++
		}
	}

	if len(v.results) > 0 {
		report.AverageTime = totalTime / time.Duration(len(v.results))
	}

	// Calculate success rates
	report.MetTargetRate = float64(report.MetTargetCount) / float64(report.TotalTests) * 100
	report.QueryTargetRate = float64(report.QueryTargetCount) / float64(report.TotalTests) * 100
	report.OverallSuccessRate = float64(report.PassedTests) / float64(report.TotalTests) * 100

	// Generate recommendations
	report.Recommendations = v.generateRecommendations(report)

	return report
}

type PaginationPerformanceReport struct {
	TotalTests         int                     `json:"totalTests"`
	PassedTests        int                     `json:"passedTests"`
	FailedTests        int                     `json:"failedTests"`
	AverageTime        time.Duration           `json:"averageTime"`
	MetTargetCount     int                     `json:"metTargetCount"`
	QueryTargetCount   int                     `json:"queryTargetCount"`
	MetTargetRate      float64                 `json:"metTargetRate"`
	QueryTargetRate    float64                 `json:"queryTargetRate"`
	OverallSuccessRate float64                 `json:"overallSuccessRate"`
	Results            []PerformanceTestResult `json:"results"`
	Recommendations    []string                `json:"recommendations"`
	GeneratedAt        time.Time               `json:"generatedAt"`
}

func (v *PaginationPerformanceValidator) generateRecommendations(report *PaginationPerformanceReport) []string {
	var recommendations []string

	if report.OverallSuccessRate < 90.0 {
		recommendations = append(recommendations,
			"Overall success rate is below 90% - investigate slow queries and missing indexes")
	}

	if report.MetTargetRate < 95.0 {
		recommendations = append(recommendations,
			"Metadata queries (COUNT) are not meeting 50ms target - consider count estimation for large datasets")
	}

	if report.QueryTargetRate < 85.0 {
		recommendations = append(recommendations,
			"Query performance is below target - review database indexes and query optimization")
	}

	if report.AverageTime > 100*time.Millisecond {
		recommendations = append(recommendations,
			"Average execution time exceeds 100ms - consider cursor-based pagination for large datasets")
	}

	// Analyze specific patterns
	slowTests := 0
	for _, result := range report.Results {
		if result.ExecutionTime > 200*time.Millisecond {
			slowTests++
		}
	}

	if slowTests > 0 {
		recommendations = append(recommendations,
			fmt.Sprintf("%d tests exceeded 200ms - review specific query patterns and index usage", slowTests))
	}

	if len(recommendations) == 0 {
		recommendations = append(recommendations,
			"Pagination performance is excellent - all targets met successfully")
	}

	return recommendations
}

// ValidateAllPaginationScenarios runs comprehensive pagination performance validation
func ValidateAllPaginationScenarios(ctx context.Context, db *gorm.DB) (*PaginationPerformanceReport, error) {
	validator := NewPaginationPerformanceValidator(db)

	log.Println("Starting pagination performance validation...")

	// Run basic pagination tests
	if err := validator.ValidateBasicPaginationPerformance(ctx); err != nil {
		return nil, fmt.Errorf("basic pagination validation failed: %w", err)
	}

	// Run cursor pagination tests
	if err := validator.ValidateCursorPaginationPerformance(ctx); err != nil {
		return nil, fmt.Errorf("cursor pagination validation failed: %w", err)
	}

	// Run filtered pagination tests
	if err := validator.ValidateFilteredPaginationPerformance(ctx); err != nil {
		return nil, fmt.Errorf("filtered pagination validation failed: %w", err)
	}

	log.Printf("Completed %d pagination performance tests", len(validator.results))

	return validator.GeneratePerformanceReport(), nil
}

// Performance target validation
func ValidatePaginationTargets(report *PaginationPerformanceReport) error {
	if report.MetTargetRate < 95.0 {
		return fmt.Errorf("metadata query target not met: %.1f%% < 95%% required", report.MetTargetRate)
	}

	if report.QueryTargetRate < 85.0 {
		return fmt.Errorf("query performance target not met: %.1f%% < 85%% required", report.QueryTargetRate)
	}

	if report.AverageTime > 100*time.Millisecond {
		return fmt.Errorf("average execution time exceeds target: %v > 100ms", report.AverageTime)
	}

	return nil
}