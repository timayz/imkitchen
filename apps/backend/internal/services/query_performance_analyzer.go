package services

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
)

type QueryPerformanceAnalyzer interface {
	AnalyzeRecipeSearchPerformance(ctx context.Context, params *models.RecipeSearchParams) (*QueryPerformanceReport, error)
	AnalyzeDatabaseIndices(ctx context.Context) (*IndexAnalysisReport, error)
	GenerateSlowQueryReport(ctx context.Context, minDuration time.Duration) (*SlowQueryReport, error)
	BenchmarkSearchQueries(ctx context.Context, iterations int) (*BenchmarkReport, error)
}

type QueryPerformanceReport struct {
	QueryType       string                 `json:"query_type"`
	ExecutionTime   time.Duration          `json:"execution_time_ms"`
	RowsScanned     int64                  `json:"rows_scanned"`
	RowsReturned    int64                  `json:"rows_returned"`
	IndexesUsed     []string               `json:"indexes_used"`
	QueryPlan       string                 `json:"query_plan"`
	Recommendations []string               `json:"recommendations"`
	Timestamp       time.Time              `json:"timestamp"`
	QueryParameters map[string]interface{} `json:"query_parameters"`
}

type IndexAnalysisReport struct {
	TotalIndices    int              `json:"total_indices"`
	UnusedIndices   []IndexInfo      `json:"unused_indices"`
	MissingIndices  []string         `json:"missing_indices_suggestions"`
	IndexUsageStats []IndexUsageInfo `json:"index_usage_stats"`
	TableSizeInfo   []TableSizeInfo  `json:"table_size_info"`
	Timestamp       time.Time        `json:"timestamp"`
}

type IndexInfo struct {
	TableName string `json:"table_name"`
	IndexName string `json:"index_name"`
	IndexDef  string `json:"index_definition"`
	SizeBytes int64  `json:"size_bytes"`
}

type IndexUsageInfo struct {
	TableName    string     `json:"table_name"`
	IndexName    string     `json:"index_name"`
	ScansCount   int64      `json:"scans_count"`
	TupsRead     int64      `json:"tuples_read"`
	TupsReturned int64      `json:"tuples_returned"`
	LastUsed     *time.Time `json:"last_used"`
}

type TableSizeInfo struct {
	TableName      string `json:"table_name"`
	RowCount       int64  `json:"row_count"`
	TableSizeBytes int64  `json:"table_size_bytes"`
	IndexSizeBytes int64  `json:"index_size_bytes"`
}

type AnalyzerSlowQueryReport struct {
	SlowQueries  []SlowQueryInfo `json:"slow_queries"`
	TotalQueries int64           `json:"total_queries"`
	AvgDuration  time.Duration   `json:"avg_duration"`
	Timestamp    time.Time       `json:"timestamp"`
}

type SlowQueryInfo struct {
	Query     string        `json:"query"`
	Duration  time.Duration `json:"duration"`
	Calls     int64         `json:"calls"`
	MeanTime  time.Duration `json:"mean_time"`
	Timestamp time.Time     `json:"timestamp"`
}

type BenchmarkReport struct {
	TestCases    []BenchmarkTestCase `json:"test_cases"`
	TotalTime    time.Duration       `json:"total_time"`
	Iterations   int                 `json:"iterations"`
	AvgQueryTime time.Duration       `json:"avg_query_time"`
	Timestamp    time.Time           `json:"timestamp"`
}

type BenchmarkTestCase struct {
	TestName      string                 `json:"test_name"`
	QueryType     string                 `json:"query_type"`
	Parameters    map[string]interface{} `json:"parameters"`
	ExecutionTime time.Duration          `json:"execution_time"`
	RowsReturned  int64                  `json:"rows_returned"`
	Success       bool                   `json:"success"`
	Error         string                 `json:"error,omitempty"`
}

type queryPerformanceAnalyzer struct {
	db *gorm.DB
}

func NewQueryPerformanceAnalyzer(db *gorm.DB) QueryPerformanceAnalyzer {
	return &queryPerformanceAnalyzer{
		db: db,
	}
}

func (a *queryPerformanceAnalyzer) AnalyzeRecipeSearchPerformance(ctx context.Context, params *models.RecipeSearchParams) (*QueryPerformanceReport, error) {
	report := &QueryPerformanceReport{
		QueryType:       "recipe_search",
		Timestamp:       time.Now(),
		QueryParameters: make(map[string]interface{}),
	}

	// Capture query parameters
	if params != nil {
		if params.Search != nil {
			report.QueryParameters["search_query"] = *params.Search
		}
		if params.CuisineType != nil {
			report.QueryParameters["cuisine_type"] = *params.CuisineType
		}
		if params.MaxPrepTime != nil {
			report.QueryParameters["max_prep_time"] = *params.MaxPrepTime
		}
		if len(params.DietaryLabels) > 0 {
			report.QueryParameters["dietary_labels"] = params.DietaryLabels
		}
	}

	// Enable query plan analysis
	var queryPlan string
	err := a.db.WithContext(ctx).Raw("SET auto_explain.log_analyze = ON").Error
	if err != nil {
		log.Printf("Warning: Could not enable auto_explain: %v", err)
	}

	// Build and execute the search query while measuring performance
	startTime := time.Now()

	query := a.db.WithContext(ctx).Model(&models.Recipe{}).Where("deleted_at IS NULL")

	// Apply filters based on search parameters
	if params != nil {
		if params.Search != nil && *params.Search != "" {
			searchQuery := *params.Search
			query = query.Where(
				"to_tsvector('english', title || ' ' || COALESCE(description, '') || ' ' || COALESCE(cuisine_type, '')) @@ plainto_tsquery('english', ?)",
				searchQuery,
			)
		}

		if params.CuisineType != nil && *params.CuisineType != "" {
			query = query.Where("cuisine_type = ?", *params.CuisineType)
		}

		if params.MaxPrepTime != nil {
			query = query.Where("prep_time <= ?", *params.MaxPrepTime)
		}

		if len(params.DietaryLabels) > 0 {
			query = query.Where("dietary_labels @> ?", fmt.Sprintf("{%s}", strings.Join(params.DietaryLabels, ",")))
		}
	}

	var recipes []models.Recipe
	var totalCount int64

	// Execute count query
	countErr := query.Count(&totalCount).Error
	if countErr != nil {
		return nil, fmt.Errorf("failed to count recipes: %w", countErr)
	}

	// Execute main query
	err = query.Limit(50).Find(&recipes).Error
	executionTime := time.Since(startTime)

	if err != nil {
		return nil, fmt.Errorf("failed to execute search query: %w", err)
	}

	report.ExecutionTime = executionTime
	report.RowsReturned = int64(len(recipes))
	report.RowsScanned = totalCount

	// Get query execution plan
	planErr := a.db.WithContext(ctx).Raw("SELECT query, plan FROM pg_stat_statements WHERE query LIKE '%recipes%' ORDER BY last_exec DESC LIMIT 1").Scan(&queryPlan).Error
	if planErr == nil {
		report.QueryPlan = queryPlan
	}

	// Generate recommendations based on performance
	report.Recommendations = a.generateRecommendations(report)

	return report, nil
}

func (a *queryPerformanceAnalyzer) AnalyzeDatabaseIndices(ctx context.Context) (*IndexAnalysisReport, error) {
	report := &IndexAnalysisReport{
		Timestamp: time.Now(),
	}

	// Get all indices
	var indices []IndexInfo
	err := a.db.WithContext(ctx).Raw(`
		SELECT 
			schemaname as table_name,
			indexname as index_name,
			indexdef as index_def,
			pg_relation_size(indexname::regclass) as size_bytes
		FROM pg_indexes 
		WHERE schemaname = 'public' 
		AND tablename LIKE '%recipe%'
		ORDER BY pg_relation_size(indexname::regclass) DESC
	`).Scan(&indices).Error

	if err != nil {
		return nil, fmt.Errorf("failed to analyze indices: %w", err)
	}

	report.TotalIndices = len(indices)

	// Get index usage statistics
	var usageStats []IndexUsageInfo
	err = a.db.WithContext(ctx).Raw(`
		SELECT 
			schemaname as table_name,
			indexname as index_name,
			idx_scan as scans_count,
			idx_tup_read as tups_read,
			idx_tup_fetch as tups_returned
		FROM pg_stat_user_indexes 
		WHERE schemaname = 'public' 
		AND indexname LIKE '%recipe%'
		ORDER BY idx_scan DESC
	`).Scan(&usageStats).Error

	if err != nil {
		return nil, fmt.Errorf("failed to get index usage stats: %w", err)
	}

	report.IndexUsageStats = usageStats

	// Identify unused indices (0 scans)
	for _, usage := range usageStats {
		if usage.ScansCount == 0 {
			for _, idx := range indices {
				if idx.IndexName == usage.IndexName {
					report.UnusedIndices = append(report.UnusedIndices, idx)
					break
				}
			}
		}
	}

	// Get table size information
	var tableSizes []TableSizeInfo
	err = a.db.WithContext(ctx).Raw(`
		SELECT 
			tablename as table_name,
			n_tup_ins + n_tup_upd + n_tup_del as row_count,
			pg_total_relation_size(schemaname||'.'||tablename) as table_size_bytes,
			pg_indexes_size(schemaname||'.'||tablename) as index_size_bytes
		FROM pg_stat_user_tables 
		WHERE schemaname = 'public' 
		AND tablename LIKE '%recipe%'
	`).Scan(&tableSizes).Error

	if err != nil {
		return nil, fmt.Errorf("failed to get table sizes: %w", err)
	}

	report.TableSizeInfo = tableSizes

	// Generate missing index suggestions
	report.MissingIndices = a.generateIndexSuggestions()

	return report, nil
}

func (a *queryPerformanceAnalyzer) GenerateSlowQueryReport(ctx context.Context, minDuration time.Duration) (*SlowQueryReport, error) {
	report := &SlowQueryReport{
		Timestamp: time.Now(),
	}

	// Query pg_stat_statements for slow queries
	var slowQueries []SlowQueryInfo
	err := a.db.WithContext(ctx).Raw(`
		SELECT 
			query,
			mean_exec_time * interval '1 millisecond' as duration,
			calls,
			mean_exec_time * interval '1 millisecond' as mean_time
		FROM pg_stat_statements 
		WHERE query LIKE '%recipe%' 
		AND mean_exec_time > ?
		ORDER BY mean_exec_time DESC
		LIMIT 50
	`, minDuration.Milliseconds()).Scan(&slowQueries).Error

	if err != nil {
		log.Printf("Warning: Could not query pg_stat_statements (extension may not be installed): %v", err)
		// Return empty report instead of error
		return report, nil
	}

	report.SlowQueries = slowQueries
	report.TotalQueries = int64(len(slowQueries))

	// Calculate average duration
	if len(slowQueries) > 0 {
		var totalDuration time.Duration
		for _, sq := range slowQueries {
			totalDuration += sq.Duration
		}
		report.AvgDuration = totalDuration / time.Duration(len(slowQueries))
	}

	return report, nil
}

func (a *queryPerformanceAnalyzer) BenchmarkSearchQueries(ctx context.Context, iterations int) (*BenchmarkReport, error) {
	report := &BenchmarkReport{
		Iterations: iterations,
		Timestamp:  time.Now(),
	}

	// Define test cases
	testCases := []struct {
		name   string
		params *models.RecipeSearchParams
	}{
		{
			name: "simple_text_search",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					Search: stringPtr("chicken"),
				},
			},
		},
		{
			name: "cuisine_filter",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					CuisineType: stringPtr("italian"),
				},
			},
		},
		{
			name: "time_constraint",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					MaxPrepTime: intPtr(30),
				},
			},
		},
		{
			name: "dietary_filter",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					DietaryLabels: []string{"vegetarian", "gluten-free"},
				},
			},
		},
		{
			name: "complex_combined",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					Search:        stringPtr("pasta"),
					CuisineType:   stringPtr("italian"),
					MaxPrepTime:   intPtr(45),
					DietaryLabels: []string{"vegetarian"},
				},
			},
		},
	}

	startTime := time.Now()

	for _, testCase := range testCases {
		benchmarkTest := BenchmarkTestCase{
			TestName:   testCase.name,
			QueryType:  "recipe_search",
			Parameters: make(map[string]interface{}),
			Success:    true,
		}

		// Record parameters
		if testCase.params.Search != nil {
			benchmarkTest.Parameters["query"] = *testCase.params.Search
		}
		if testCase.params.CuisineType != nil {
			benchmarkTest.Parameters["cuisine_type"] = *testCase.params.CuisineType
		}

		// Run multiple iterations
		var totalTime time.Duration
		var rowsReturned int64

		for i := 0; i < iterations; i++ {
			perfReport, err := a.AnalyzeRecipeSearchPerformance(ctx, testCase.params)
			if err != nil {
				benchmarkTest.Success = false
				benchmarkTest.Error = err.Error()
				break
			}
			totalTime += perfReport.ExecutionTime
			rowsReturned = perfReport.RowsReturned
		}

		benchmarkTest.ExecutionTime = totalTime / time.Duration(iterations)
		benchmarkTest.RowsReturned = rowsReturned

		report.TestCases = append(report.TestCases, benchmarkTest)
	}

	report.TotalTime = time.Since(startTime)

	// Calculate average query time across all test cases
	var totalAvgTime time.Duration
	successfulTests := 0
	for _, test := range report.TestCases {
		if test.Success {
			totalAvgTime += test.ExecutionTime
			successfulTests++
		}
	}
	if successfulTests > 0 {
		report.AvgQueryTime = totalAvgTime / time.Duration(successfulTests)
	}

	return report, nil
}

func (a *queryPerformanceAnalyzer) generateRecommendations(report *QueryPerformanceReport) []string {
	var recommendations []string

	// Performance-based recommendations
	if report.ExecutionTime > 200*time.Millisecond {
		recommendations = append(recommendations, "Query exceeds 200ms target - consider adding specific indices")
	}

	if report.RowsScanned > 0 && report.RowsReturned > 0 {
		scanRatio := float64(report.RowsReturned) / float64(report.RowsScanned)
		if scanRatio < 0.1 {
			recommendations = append(recommendations, "Low selectivity detected - query scans too many rows for results returned")
		}
	}

	// Query-specific recommendations
	if query, ok := report.QueryParameters["search_query"]; ok && query != "" {
		recommendations = append(recommendations, "Text search detected - ensure GIN index on tsvector is being used")
	}

	if _, hasCuisine := report.QueryParameters["cuisine_type"]; hasCuisine {
		if _, hasTime := report.QueryParameters["max_prep_time"]; hasTime {
			recommendations = append(recommendations, "Multi-column filter detected - consider composite index on (cuisine_type, prep_time)")
		}
	}

	return recommendations
}

func (a *queryPerformanceAnalyzer) generateIndexSuggestions() []string {
	return []string{
		"CREATE INDEX CONCURRENTLY idx_recipes_cuisine_prep_time ON recipes(cuisine_type, prep_time) WHERE deleted_at IS NULL",
		"CREATE INDEX CONCURRENTLY idx_recipes_dietary_time ON recipes USING GIN(dietary_labels) INCLUDE (prep_time) WHERE deleted_at IS NULL",
		"CREATE INDEX CONCURRENTLY idx_recipes_rating_created ON recipes(average_rating DESC, created_at DESC) WHERE deleted_at IS NULL AND average_rating > 0",
	}
}

// Helper functions
func stringPtr(s string) *string {
	return &s
}

func intPtr(i int) *int {
	return &i
}
