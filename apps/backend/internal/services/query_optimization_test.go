package services

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

// Test Query Performance Service
func TestQueryPerformanceService(t *testing.T) {
	db, mock := setupTestDB(t)
	service := NewQueryPerformanceService(db)

	t.Run("StartMonitoring", func(t *testing.T) {
		monitoredDB := service.StartMonitoring()
		assert.NotNil(t, monitoredDB)
		assert.NotEqual(t, db, monitoredDB) // Should return a different instance
	})

	t.Run("GetSlowQueries", func(t *testing.T) {
		ctx := context.Background()
		since := time.Hour

		queries, err := service.GetSlowQueries(ctx, since)
		assert.NoError(t, err)
		assert.NotNil(t, queries)
		assert.Equal(t, 0, len(queries)) // No queries initially
	})

	t.Run("AnalyzeQuery", func(t *testing.T) {
		ctx := context.Background()
		testQuery := "SELECT * FROM recipes WHERE deleted_at IS NULL"

		// Mock the EXPLAIN query
		mock.ExpectQuery("EXPLAIN \\(FORMAT JSON, ANALYZE false, BUFFERS false\\) SELECT").
			WillReturnRows(sqlmock.NewRows([]string{"QUERY PLAN"}).
				AddRow(`[{"Plan": {"Node Type": "Seq Scan", "Total Cost": 100.0}}]`))

		analysis, err := service.AnalyzeQuery(ctx, testQuery)
		assert.NoError(t, err)
		assert.NotNil(t, analysis)
		assert.Equal(t, testQuery, analysis.Query)
		assert.Greater(t, analysis.EstimatedCost, 0.0)
		assert.NotEmpty(t, analysis.OptimizationTips)
	})

	t.Run("GetPerformanceReport", func(t *testing.T) {
		ctx := context.Background()
		since := time.Hour

		// Mock index usage stats query
		mock.ExpectQuery("SELECT").
			WithArgs("public").
			WillReturnRows(sqlmock.NewRows([]string{
				"schema_name", "table_name", "index_name", "scan_count", "tuple_reads", "tuple_fetches",
			}).AddRow("public", "recipes", "idx_recipes_id", 100, 1000, 800))

		report, err := service.GetPerformanceReport(ctx, since)
		assert.NoError(t, err)
		assert.NotNil(t, report)
		assert.Equal(t, 0, report.TotalQueries) // No queries executed yet
		assert.Equal(t, 0, report.SlowQueries)
		assert.NotEmpty(t, report.Recommendations)
	})

	t.Run("OptimizeCommonQueries", func(t *testing.T) {
		ctx := context.Background()

		// Mock index creation queries
		mock.ExpectExec("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_search_composite").
			WillReturnResult(sqlmock.NewResult(0, 0))
		mock.ExpectExec("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_active_user").
			WillReturnResult(sqlmock.NewResult(0, 0))
		mock.ExpectExec("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_ratings_recipe_user").
			WillReturnResult(sqlmock.NewResult(0, 0))
		mock.ExpectExec("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_fulltext_gin").
			WillReturnResult(sqlmock.NewResult(0, 0))
		mock.ExpectExec("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_ingredients_gin").
			WillReturnResult(sqlmock.NewResult(0, 0))
		mock.ExpectExec("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_meals_gin").
			WillReturnResult(sqlmock.NewResult(0, 0))

		err := service.OptimizeCommonQueries(ctx)
		assert.NoError(t, err)
	})

	assert.NoError(t, mock.ExpectationsWereMet())
}

// Test Pagination Service
func TestPaginationService(t *testing.T) {
	db, _ := setupTestDB(t)
	service := NewPaginationService()

	t.Run("ApplyCursorPagination_Forward", func(t *testing.T) {
		query := db.Table("recipes")
		first := 10
		params := CursorPaginationParams{
			First: &first,
		}

		result, err := service.ApplyCursorPagination(query, params, "created_at")
		assert.NoError(t, err)
		assert.NotNil(t, result)
	})

	t.Run("ApplyCursorPagination_Backward", func(t *testing.T) {
		query := db.Table("recipes")
		last := 5
		params := CursorPaginationParams{
			Last: &last,
		}

		result, err := service.ApplyCursorPagination(query, params, "created_at")
		assert.NoError(t, err)
		assert.NotNil(t, result)
	})

	t.Run("ApplyCursorPagination_InvalidParams", func(t *testing.T) {
		query := db.Table("recipes")
		first := 10
		last := 5
		params := CursorPaginationParams{
			First: &first,
			Last:  &last,
		}

		_, err := service.ApplyCursorPagination(query, params, "created_at")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "cannot specify both 'first' and 'last'")
	})

	t.Run("ApplyOffsetPagination", func(t *testing.T) {
		query := db.Table("recipes")
		params := OffsetPaginationParams{
			Page:  2,
			Limit: 20,
		}

		result := service.ApplyOffsetPagination(query, params)
		assert.NotNil(t, result)
	})

	t.Run("CalculatePaginationInfo", func(t *testing.T) {
		totalCount := int64(100)
		params := OffsetPaginationParams{
			Page:  3,
			Limit: 20,
		}

		info := service.CalculatePaginationInfo(totalCount, params)
		assert.NotNil(t, info)
		assert.Equal(t, true, info.HasNextPage)
		assert.Equal(t, true, info.HasPreviousPage)
		assert.Equal(t, 3, *info.CurrentPage)
		assert.Equal(t, 5, *info.TotalPages)
	})

	t.Run("GenerateCursor", func(t *testing.T) {
		item := map[string]interface{}{
			"id":         "test-id",
			"created_at": time.Now(),
		}

		cursor, err := service.GenerateCursor(item, "created_at")
		assert.NoError(t, err)
		assert.NotEmpty(t, cursor)

		// Verify cursor can be parsed
		parsed, err := service.ParseCursor(cursor)
		assert.NoError(t, err)
		assert.NotNil(t, parsed)
		assert.Equal(t, "created_at", parsed.SortField)
	})

	t.Run("CreatePaginatedResult_CursorPagination", func(t *testing.T) {
		items := []interface{}{
			map[string]interface{}{"id": 1, "name": "Recipe 1"},
			map[string]interface{}{"id": 2, "name": "Recipe 2"},
		}

		first := 10
		params := CursorPaginationParams{First: &first}

		result, err := service.CreatePaginatedResult(items, params, nil, "created_at")
		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Equal(t, 2, len(result.Edges))
		assert.NotNil(t, result.PageInfo)
	})

	t.Run("CreatePaginatedResult_OffsetPagination", func(t *testing.T) {
		items := []interface{}{
			map[string]interface{}{"id": 1, "name": "Recipe 1"},
			map[string]interface{}{"id": 2, "name": "Recipe 2"},
		}

		params := OffsetPaginationParams{Page: 1, Limit: 20}
		totalCount := int64(50)

		result, err := service.CreatePaginatedResult(items, params, &totalCount, "")
		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Equal(t, 2, len(result.Edges))
		assert.NotNil(t, result.PageInfo)
		assert.Equal(t, int64(50), *result.PageInfo.TotalCount)
	})
}

// Test Query Cache Service  
func TestQueryCacheService(t *testing.T) {
	// Mock Redis client for testing
	service := NewQueryCacheService(nil) // In real tests, would use miniredis

	t.Run("GenerateCacheKey", func(t *testing.T) {
		query := "SELECT * FROM recipes WHERE id = $1"
		args := []interface{}{123}

		key := service.(*queryCacheService).generateCacheKey(query, args)
		assert.NotEmpty(t, key)
		assert.Contains(t, key, "query:")

		// Same query and args should produce same key
		key2 := service.(*queryCacheService).generateCacheKey(query, args)
		assert.Equal(t, key, key2)
	})

	t.Run("DetermineTTL", func(t *testing.T) {
		testCases := []struct {
			query       string
			expectedMin time.Duration
		}{
			{"SELECT COUNT(*) FROM recipes", time.Minute},           // Aggregation
			{"SELECT * FROM users WHERE id = $1", 5 * time.Minute}, // User lookup
			{"SELECT * FROM recipes ORDER BY created_at DESC", time.Minute}, // List query
		}

		for _, tc := range testCases {
			ttl := service.(*queryCacheService).determineTTL(tc.query)
			assert.GreaterOrEqual(t, ttl, tc.expectedMin)
		}
	})

	t.Run("GetCacheMetrics", func(t *testing.T) {
		metrics := service.GetCacheMetrics()
		assert.NotNil(t, metrics)
		assert.GreaterOrEqual(t, metrics.HitRate, 0.0)
		assert.LessOrEqual(t, metrics.HitRate, 1.0)
	})

	t.Run("InvalidatePattern", func(t *testing.T) {
		err := service.InvalidatePattern("recipes:*")
		// Will fail without real Redis, but tests the interface
		assert.Error(t, err) // Expected since no real Redis connection
	})
}

// Test Query Monitoring Integration
func TestQueryMonitoringIntegration(t *testing.T) {
	db, mock := setupTestDB(t)
	
	performanceService := NewQueryPerformanceService(db)
	cacheService := NewQueryCacheService(nil)
	integration := NewQueryMonitoringIntegration(performanceService, cacheService)

	t.Run("Initialize", func(t *testing.T) {
		monitoredDB := integration.Initialize(db)
		assert.NotNil(t, monitoredDB)
	})

	t.Run("GetRealTimeMetrics", func(t *testing.T) {
		metrics := integration.GetRealTimeMetrics()
		assert.NotNil(t, metrics)
		assert.Equal(t, 0, metrics.ActiveQueries)
		assert.GreaterOrEqual(t, metrics.QueriesPerSecond, 0.0)
		assert.GreaterOrEqual(t, metrics.SlowQueryRate, 0.0)
	})

	t.Run("SetSlowQueryAlert", func(t *testing.T) {
		alertCalled := false
		callback := func(metric QueryPerformanceMetrics) {
			alertCalled = true
		}

		integration.SetSlowQueryAlert(callback)
		// Alert callback is set but won't be triggered without actual query execution
		assert.False(t, alertCalled)
	})

	t.Run("GetQueryTrends", func(t *testing.T) {
		ctx := context.Background()
		
		trends, err := integration.GetQueryTrends(ctx, 24)
		assert.NoError(t, err)
		assert.NotNil(t, trends)
		assert.NotNil(t, trends.HourlyMetrics)
		assert.NotNil(t, trends.TopSlowQueries)
		assert.Contains(t, []string{"improving", "stable", "degrading"}, trends.PerformanceTrend)
	})

	t.Run("ExportPerformanceReport", func(t *testing.T) {
		ctx := context.Background()
		
		data, err := integration.ExportPerformanceReport(ctx, "json")
		assert.NoError(t, err)
		assert.NotEmpty(t, data)

		// Verify JSON structure
		var report map[string]interface{}
		err = json.Unmarshal(data, &report)
		assert.NoError(t, err)
		assert.Contains(t, report, "timestamp")
		assert.Contains(t, report, "realTimeMetrics")
		assert.Contains(t, report, "queryTrends")
	})

	t.Run("StartAndStopPerformanceReporting", func(t *testing.T) {
		// Start reporting
		integration.StartPerformanceReporting(time.Millisecond * 100)

		// Let it run briefly
		time.Sleep(time.Millisecond * 150)

		// Stop reporting
		integration.StopPerformanceReporting()

		// Should not panic or error
	})

	assert.NoError(t, mock.ExpectationsWereMet())
}

// Test Advanced Recipe Search Integration
func TestAdvancedRecipeSearchIntegration(t *testing.T) {
	db, mock := setupTestDB(t)
	
	cacheService := NewQueryCacheService(nil)
	paginationService := NewPaginationService()
	service := NewAdvancedRecipeSearchService(db, cacheService, paginationService)

	t.Run("SearchRecipes_BasicQuery", func(t *testing.T) {
		ctx := context.Background()
		request := SearchRequest{
			Query:     "chicken",
			MealTypes: []string{"dinner"},
			Limit:     20,
		}

		// Mock the search query
		mock.ExpectQuery("SELECT").
			WillReturnRows(sqlmock.NewRows([]string{"id", "title"}).
				AddRow(1, "Chicken Recipe"))

		// Mock count query
		mock.ExpectQuery("SELECT count").
			WillReturnRows(sqlmock.NewRows([]string{"count"}).
				AddRow(1))

		results, err := service.SearchRecipes(ctx, request)
		assert.NoError(t, err)
		assert.NotNil(t, results)
		assert.Equal(t, 1, len(results.Recipes))
	})

	t.Run("GetCachedFacets", func(t *testing.T) {
		ctx := context.Background()

		// Mock facets queries
		mock.ExpectQuery("SELECT DISTINCT cuisine_type").
			WillReturnRows(sqlmock.NewRows([]string{"cuisine_type"}).
				AddRow("Italian").AddRow("Mexican"))

		mock.ExpectQuery("SELECT DISTINCT complexity").
			WillReturnRows(sqlmock.NewRows([]string{"complexity"}).
				AddRow("simple").AddRow("moderate"))

		mock.ExpectQuery("SELECT DISTINCT UNNEST").
			WillReturnRows(sqlmock.NewRows([]string{"dietary_label"}).
				AddRow("vegetarian").AddRow("vegan"))

		facets, err := service.GetCachedFacets(ctx)
		assert.NoError(t, err)
		assert.NotNil(t, facets)
		assert.Equal(t, 2, len(facets.CuisineTypes))
		assert.Equal(t, 2, len(facets.Complexities))
	})

	t.Run("GetSearchSuggestions", func(t *testing.T) {
		ctx := context.Background()
		query := "chick"

		// Mock suggestions query
		mock.ExpectQuery("SELECT title").
			WithArgs("%chick%").
			WillReturnRows(sqlmock.NewRows([]string{"title"}).
				AddRow("Chicken Curry").AddRow("Chicken Salad"))

		suggestions, err := service.GetSearchSuggestions(ctx, query, 5)
		assert.NoError(t, err)
		assert.Equal(t, 2, len(suggestions))
		assert.Contains(t, suggestions[0], "Chicken")
	})

	assert.NoError(t, mock.ExpectationsWereMet())
}

// Helper function to setup test database with mock
func setupTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock) {
	mockDB, mock, err := sqlmock.New(sqlmock.QueryMatcherOption(sqlmock.QueryMatcherRegexp))
	require.NoError(t, err)

	db, err := gorm.Open(postgres.New(postgres.Config{
		Conn: mockDB,
	}), &gorm.Config{
		SkipDefaultTransaction: true,
	})
	require.NoError(t, err)

	return db, mock
}

// Benchmark tests for performance validation
func BenchmarkQueryPerformanceService(b *testing.B) {
	db, _ := setupTestDB(&testing.T{})
	service := NewQueryPerformanceService(db)

	b.Run("GenerateCursor", func(b *testing.B) {
		paginationService := NewPaginationService()
		item := map[string]interface{}{
			"id":         "test-id",
			"created_at": time.Now(),
		}

		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			_, _ = paginationService.GenerateCursor(item, "created_at")
		}
	})

	b.Run("GetRealTimeMetrics", func(b *testing.B) {
		cacheService := NewQueryCacheService(nil)
		integration := NewQueryMonitoringIntegration(service, cacheService)

		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			_ = integration.GetRealTimeMetrics()
		}
	})
}

// Integration test with actual query execution simulation
func TestQueryExecutionMonitoring(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	db, mock := setupTestDB(t)
	performanceService := NewQueryPerformanceService(db)
	
	// Initialize monitoring
	monitoredDB := performanceService.StartMonitoring()

	// Simulate a fast query
	mock.ExpectQuery("SELECT \\* FROM recipes").
		WillReturnRows(sqlmock.NewRows([]string{"id"}).AddRow(1))

	var result struct{ ID int }
	err := monitoredDB.Raw("SELECT * FROM recipes WHERE id = 1").Scan(&result).Error
	assert.NoError(t, err)

	// Get metrics
	ctx := context.Background()
	slowQueries, err := performanceService.GetSlowQueries(ctx, time.Hour)
	assert.NoError(t, err)
	assert.Equal(t, 0, len(slowQueries)) // Should be fast query

	assert.NoError(t, mock.ExpectationsWereMet())
}