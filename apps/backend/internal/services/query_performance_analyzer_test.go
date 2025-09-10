package services

import (
	"context"
	"database/sql"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
)

func setupTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock, func()) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)

	gormDB, err := gorm.Open(postgres.New(postgres.Config{
		Conn: db,
	}), &gorm.Config{})
	require.NoError(t, err)

	cleanup := func() {
		db.Close()
	}

	return gormDB, mock, cleanup
}

func TestQueryPerformanceAnalyzer_AnalyzeRecipeSearchPerformance(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	analyzer := NewQueryPerformanceAnalyzer(db)

	tests := []struct {
		name     string
		params   *models.RecipeSearchParams
		mockRows func(sqlmock.Sqlmock)
		want     func(*testing.T, *QueryPerformanceReport)
	}{
		{
			name: "simple_text_search",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					Search: stringPtr("chicken pasta"),
				},
			},
			mockRows: func(mock sqlmock.Sqlmock) {
				// Mock auto_explain enable
				mock.ExpectExec("SET auto_explain.log_analyze = ON").WillReturnResult(sqlmock.NewResult(0, 0))
				
				// Mock count query
				mock.ExpectQuery(`SELECT count\(\*\) FROM "recipes"`).
					WithArgs("chicken pasta").
					WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(150))

				// Mock main search query
				recipeRows := sqlmock.NewRows([]string{"id", "title", "description", "cuisine_type", "prep_time", "cook_time"}).
					AddRow("1", "Chicken Pasta", "Delicious pasta", "italian", 20, 25).
					AddRow("2", "Creamy Chicken Pasta", "Rich and creamy", "italian", 15, 30)
				
				mock.ExpectQuery(`SELECT \* FROM "recipes"`).
					WithArgs("chicken pasta").
					WillReturnRows(recipeRows)

				// Mock plan query
				mock.ExpectQuery("SELECT query, plan FROM pg_stat_statements").
					WillReturnRows(sqlmock.NewRows([]string{"plan"}).AddRow(""))
			},
			want: func(t *testing.T, report *QueryPerformanceReport) {
				assert.Equal(t, "recipe_search", report.QueryType)
				assert.Equal(t, int64(2), report.RowsReturned)
				assert.Equal(t, int64(150), report.RowsScanned)
				assert.Contains(t, report.QueryParameters, "search_query")
				assert.Equal(t, "chicken pasta", report.QueryParameters["search_query"])
				assert.True(t, report.ExecutionTime > 0)
			},
		},
		{
			name: "cuisine_and_time_filter",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					CuisineType: stringPtr("italian"),
					MaxPrepTime: intPtr(30),
				},
			},
			mockRows: func(mock sqlmock.Sqlmock) {
				mock.ExpectExec("SET auto_explain.log_analyze = ON").WillReturnResult(sqlmock.NewResult(0, 0))
				
				mock.ExpectQuery(`SELECT count\(\*\) FROM "recipes"`).
					WithArgs("italian", 30).
					WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(75))

				recipeRows := sqlmock.NewRows([]string{"id", "title", "cuisine_type", "prep_time"}).
					AddRow("1", "Quick Italian Pasta", "italian", 25)
				
				mock.ExpectQuery(`SELECT \* FROM "recipes"`).
					WithArgs("italian", 30).
					WillReturnRows(recipeRows)

				mock.ExpectQuery("SELECT query, plan FROM pg_stat_statements").
					WillReturnRows(sqlmock.NewRows([]string{"plan"}).AddRow(""))
			},
			want: func(t *testing.T, report *QueryPerformanceReport) {
				assert.Equal(t, "recipe_search", report.QueryType)
				assert.Equal(t, int64(1), report.RowsReturned)
				assert.Equal(t, int64(75), report.RowsScanned)
				assert.Contains(t, report.QueryParameters, "cuisine_type")
				assert.Contains(t, report.QueryParameters, "max_prep_time")
				assert.Contains(t, report.Recommendations, "Multi-column filter detected - consider composite index on (cuisine_type, prep_time)")
			},
		},
		{
			name: "dietary_labels_filter",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					DietaryLabels: []string{"vegetarian", "gluten-free"},
				},
			},
			mockRows: func(mock sqlmock.Sqlmock) {
				mock.ExpectExec("SET auto_explain.log_analyze = ON").WillReturnResult(sqlmock.NewResult(0, 0))
				
				mock.ExpectQuery(`SELECT count\(\*\) FROM "recipes"`).
					WithArgs("{vegetarian,gluten-free}").
					WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(25))

				recipeRows := sqlmock.NewRows([]string{"id", "title"}).
					AddRow("1", "Veggie Gluten-Free Bowl")
				
				mock.ExpectQuery(`SELECT \* FROM "recipes"`).
					WithArgs("{vegetarian,gluten-free}").
					WillReturnRows(recipeRows)

				mock.ExpectQuery("SELECT query, plan FROM pg_stat_statements").
					WillReturnRows(sqlmock.NewRows([]string{"plan"}).AddRow(""))
			},
			want: func(t *testing.T, report *QueryPerformanceReport) {
				assert.Equal(t, int64(1), report.RowsReturned)
				assert.Equal(t, int64(25), report.RowsScanned)
				assert.Contains(t, report.QueryParameters, "dietary_labels")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.mockRows(mock)

			report, err := analyzer.AnalyzeRecipeSearchPerformance(context.Background(), tt.params)
			require.NoError(t, err)
			require.NotNil(t, report)

			tt.want(t, report)

			// Verify all expectations were met
			assert.NoError(t, mock.ExpectationsWereMet())
		})
	}
}

func TestQueryPerformanceAnalyzer_AnalyzeDatabaseIndices(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	analyzer := NewQueryPerformanceAnalyzer(db)

	// Mock index information query
	indexRows := sqlmock.NewRows([]string{"table_name", "index_name", "index_def", "size_bytes"}).
		AddRow("recipes", "idx_recipes_cuisine_diet_preptime", "CREATE INDEX ...", 1024000).
		AddRow("recipes", "idx_recipes_fulltext_ranked", "CREATE INDEX ...", 512000).
		AddRow("recipes", "idx_unused_index", "CREATE INDEX ...", 256000)

	mock.ExpectQuery("SELECT.*FROM pg_indexes").
		WillReturnRows(indexRows)

	// Mock index usage statistics
	usageRows := sqlmock.NewRows([]string{"table_name", "index_name", "scans_count", "tups_read", "tups_returned"}).
		AddRow("recipes", "idx_recipes_cuisine_diet_preptime", 1500, 15000, 12000).
		AddRow("recipes", "idx_recipes_fulltext_ranked", 800, 8000, 6000).
		AddRow("recipes", "idx_unused_index", 0, 0, 0)

	mock.ExpectQuery("SELECT.*FROM pg_stat_user_indexes").
		WillReturnRows(usageRows)

	// Mock table size information
	tableSizeRows := sqlmock.NewRows([]string{"table_name", "row_count", "table_size_bytes", "index_size_bytes"}).
		AddRow("recipes", 10000, 52428800, 20971520)

	mock.ExpectQuery("SELECT.*FROM pg_stat_user_tables").
		WillReturnRows(tableSizeRows)

	report, err := analyzer.AnalyzeDatabaseIndices(context.Background())
	require.NoError(t, err)
	require.NotNil(t, report)

	// Verify report structure
	assert.Equal(t, 3, report.TotalIndices)
	assert.Len(t, report.IndexUsageStats, 3)
	assert.Len(t, report.UnusedIndices, 1)
	assert.Equal(t, "idx_unused_index", report.UnusedIndices[0].IndexName)
	assert.Len(t, report.TableSizeInfo, 1)
	assert.Equal(t, int64(10000), report.TableSizeInfo[0].RowCount)
	assert.Greater(t, len(report.MissingIndices), 0)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryPerformanceAnalyzer_GenerateSlowQueryReport(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	analyzer := NewQueryPerformanceAnalyzer(db)

	minDuration := 100 * time.Millisecond

	// Mock pg_stat_statements query
	slowQueryRows := sqlmock.NewRows([]string{"query", "duration", "calls", "mean_time"}).
		AddRow("SELECT * FROM recipes WHERE cuisine_type = 'italian'", "250ms", 45, "250ms").
		AddRow("SELECT * FROM recipes WHERE dietary_labels @> '[\"vegetarian\"]'", "180ms", 23, "180ms")

	mock.ExpectQuery("SELECT.*FROM pg_stat_statements").
		WithArgs(int64(100)).
		WillReturnRows(slowQueryRows)

	report, err := analyzer.GenerateSlowQueryReport(context.Background(), minDuration)
	require.NoError(t, err)
	require.NotNil(t, report)

	assert.Equal(t, int64(2), report.TotalQueries)
	assert.Len(t, report.SlowQueries, 2)
	assert.Contains(t, report.SlowQueries[0].Query, "italian")
	assert.Contains(t, report.SlowQueries[1].Query, "vegetarian")
	assert.Greater(t, report.AvgDuration, time.Duration(0))

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryPerformanceAnalyzer_GenerateSlowQueryReport_ExtensionNotInstalled(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	analyzer := NewQueryPerformanceAnalyzer(db)

	minDuration := 100 * time.Millisecond

	// Mock pg_stat_statements query failure (extension not installed)
	mock.ExpectQuery("SELECT.*FROM pg_stat_statements").
		WithArgs(int64(100)).
		WillReturnError(sql.ErrNoRows)

	report, err := analyzer.GenerateSlowQueryReport(context.Background(), minDuration)
	require.NoError(t, err) // Should not error, just return empty report
	require.NotNil(t, report)

	assert.Equal(t, int64(0), report.TotalQueries)
	assert.Len(t, report.SlowQueries, 0)
	assert.Equal(t, time.Duration(0), report.AvgDuration)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryPerformanceAnalyzer_BenchmarkSearchQueries(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	analyzer := NewQueryPerformanceAnalyzer(db)

	iterations := 3

	// Mock multiple executions for benchmarking
	for i := 0; i < 5*iterations; i++ { // 5 test cases * 3 iterations
		mock.ExpectExec("SET auto_explain.log_analyze = ON").WillReturnResult(sqlmock.NewResult(0, 0))
		
		mock.ExpectQuery(`SELECT count\(\*\) FROM "recipes"`).
			WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(100))

		recipeRows := sqlmock.NewRows([]string{"id", "title"}).
			AddRow("1", "Test Recipe")
		
		mock.ExpectQuery(`SELECT \* FROM "recipes"`).
			WillReturnRows(recipeRows)

		mock.ExpectQuery("SELECT query, plan FROM pg_stat_statements").
			WillReturnRows(sqlmock.NewRows([]string{"plan"}).AddRow(""))
	}

	report, err := analyzer.BenchmarkSearchQueries(context.Background(), iterations)
	require.NoError(t, err)
	require.NotNil(t, report)

	assert.Equal(t, iterations, report.Iterations)
	assert.Len(t, report.TestCases, 5) // 5 predefined test cases
	assert.Greater(t, report.TotalTime, time.Duration(0))
	assert.Greater(t, report.AvgQueryTime, time.Duration(0))

	// Verify all test cases succeeded
	for _, testCase := range report.TestCases {
		assert.True(t, testCase.Success, "Test case %s should succeed", testCase.TestName)
		assert.Greater(t, testCase.ExecutionTime, time.Duration(0))
		assert.Equal(t, int64(1), testCase.RowsReturned)
	}

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryPerformanceAnalyzer_generateRecommendations(t *testing.T) {
	analyzer := &queryPerformanceAnalyzer{}

	tests := []struct {
		name   string
		report *QueryPerformanceReport
		want   []string
	}{
		{
			name: "slow_query_recommendation",
			report: &QueryPerformanceReport{
				ExecutionTime:    250 * time.Millisecond,
				RowsScanned:      1000,
				RowsReturned:     10,
				QueryParameters:  map[string]interface{}{},
			},
			want: []string{
				"Query exceeds 200ms target - consider adding specific indices",
				"Low selectivity detected - query scans too many rows for results returned",
			},
		},
		{
			name: "text_search_recommendation",
			report: &QueryPerformanceReport{
				ExecutionTime:    50 * time.Millisecond,
				QueryParameters:  map[string]interface{}{"search_query": "chicken"},
			},
			want: []string{
				"Text search detected - ensure GIN index on tsvector is being used",
			},
		},
		{
			name: "multi_column_filter_recommendation",
			report: &QueryPerformanceReport{
				ExecutionTime: 100 * time.Millisecond,
				QueryParameters: map[string]interface{}{
					"cuisine_type":  "italian",
					"max_prep_time": 30,
				},
			},
			want: []string{
				"Multi-column filter detected - consider composite index on (cuisine_type, prep_time)",
			},
		},
		{
			name: "good_performance_no_recommendations",
			report: &QueryPerformanceReport{
				ExecutionTime:   50 * time.Millisecond,
				RowsScanned:     100,
				RowsReturned:    90,
				QueryParameters: map[string]interface{}{},
			},
			want: []string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := analyzer.generateRecommendations(tt.report)
			assert.ElementsMatch(t, tt.want, got)
		})
	}
}

func TestQueryPerformanceAnalyzer_generateIndexSuggestions(t *testing.T) {
	analyzer := &queryPerformanceAnalyzer{}

	suggestions := analyzer.generateIndexSuggestions()

	assert.Greater(t, len(suggestions), 0)
	assert.Contains(t, suggestions[0], "CREATE INDEX CONCURRENTLY")
	assert.Contains(t, suggestions[0], "recipes")

	// Verify each suggestion is a valid SQL statement
	for _, suggestion := range suggestions {
		assert.True(t, strings.HasPrefix(suggestion, "CREATE INDEX CONCURRENTLY"))
		assert.Contains(t, suggestion, "recipes")
		assert.Contains(t, suggestion, "WHERE deleted_at IS NULL")
	}
}

// Performance tests to verify sub-200ms target
func TestQueryPerformanceAnalyzer_Performance_Sub200ms(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping performance test in short mode")
	}

	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	analyzer := NewQueryPerformanceAnalyzer(db)

	// Setup fast mock responses
	mock.ExpectExec("SET auto_explain.log_analyze = ON").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectQuery(`SELECT count\(\*\) FROM "recipes"`).
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(5000))

	recipeRows := sqlmock.NewRows([]string{"id", "title"})
	for i := 0; i < 50; i++ {
		recipeRows.AddRow(i, fmt.Sprintf("Recipe %d", i))
	}
	
	mock.ExpectQuery(`SELECT \* FROM "recipes"`).WillReturnRows(recipeRows)
	mock.ExpectQuery("SELECT query, plan FROM pg_stat_statements").
		WillReturnRows(sqlmock.NewRows([]string{"plan"}).AddRow(""))

	// Measure actual execution time
	start := time.Now()
	report, err := analyzer.AnalyzeRecipeSearchPerformance(context.Background(), &models.RecipeSearchParams{
		RecipeFilters: models.RecipeFilters{
			Search: stringPtr("test"),
		},
	})
	actualDuration := time.Since(start)

	require.NoError(t, err)
	require.NotNil(t, report)

	// The analyzer itself should complete well under 200ms
	assert.Less(t, actualDuration, 200*time.Millisecond, 
		"Query performance analyzer should complete analysis in under 200ms")

	assert.NoError(t, mock.ExpectationsWereMet())
}