package services

import (
	"context"
	"database/sql"
	"encoding/json"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func TestQueryExecutionMonitor_EnableQueryPlanLogging(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	// Mock the auto_explain setup queries
	mock.ExpectExec("LOAD 'auto_explain'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_min_duration = '100ms'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_analyze = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_verbose = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_buffers = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_timing = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_nested_statements = true").WillReturnResult(sqlmock.NewResult(0, 0))

	err := monitor.EnableQueryPlanLogging(context.Background())
	require.NoError(t, err)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_EnableQueryPlanLogging_Error(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	// Mock first query success, second query failure
	mock.ExpectExec("LOAD 'auto_explain'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_min_duration = '100ms'").
		WillReturnError(sql.ErrConnDone)

	err := monitor.EnableQueryPlanLogging(context.Background())
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to enable query plan logging")

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_DisableQueryPlanLogging(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	// Mock the disable queries
	mock.ExpectExec("SET auto_explain.log_min_duration = -1").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_analyze = false").WillReturnResult(sqlmock.NewResult(0, 0))

	err := monitor.DisableQueryPlanLogging(context.Background())
	require.NoError(t, err)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_GetQueryExecutionPlan(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	testQuery := "SELECT * FROM recipes WHERE cuisine_type = $1"
	testArgs := []interface{}{"italian"}

	// Create mock execution plan JSON
	mockPlan := map[string]interface{}{
		"Plan": []interface{}{
			map[string]interface{}{
				"Node Type":     "Index Scan",
				"Startup Cost":  0.43,
				"Total Cost":    45.67,
				"Plan Rows":     float64(25),
				"Actual Rows":   float64(23),
				"Index Name":    "idx_recipes_cuisine_type",
			},
		},
	}

	explainResult := []map[string]interface{}{
		{
			"QUERY PLAN": mockPlan,
		},
	}

	explainQuery := "EXPLAIN (ANALYZE true, BUFFERS true, VERBOSE true, FORMAT JSON) SELECT * FROM recipes WHERE cuisine_type = $1"
	
	mock.ExpectQuery("EXPLAIN \\(ANALYZE true, BUFFERS true, VERBOSE true, FORMAT JSON\\)").
		WithArgs("italian").
		WillReturnRows(sqlmock.NewRows([]string{"QUERY PLAN"}).
			AddRow(`{"Plan": [{"Node Type": "Index Scan", "Startup Cost": 0.43, "Total Cost": 45.67, "Plan Rows": 25, "Actual Rows": 23, "Index Name": "idx_recipes_cuisine_type"}]}`))

	plan, err := monitor.GetQueryExecutionPlan(context.Background(), testQuery, testArgs...)
	require.NoError(t, err)
	require.NotNil(t, plan)

	assert.Equal(t, testQuery, plan.Query)
	assert.Equal(t, 0.43, plan.StartupCost)
	assert.Equal(t, 45.67, plan.TotalCost)
	assert.Equal(t, int64(25), plan.PlanRows)
	assert.Equal(t, int64(23), plan.ActualRows)
	assert.Greater(t, plan.ExecutionTime, time.Duration(0))
	assert.NotEmpty(t, plan.Plan)
	assert.NotNil(t, plan.PlanJSON)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_GetQueryExecutionPlan_NoResults(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	testQuery := "SELECT * FROM recipes"

	mock.ExpectQuery("EXPLAIN").
		WillReturnRows(sqlmock.NewRows([]string{"QUERY PLAN"}))

	plan, err := monitor.GetQueryExecutionPlan(context.Background(), testQuery)
	assert.Error(t, err)
	assert.Nil(t, plan)
	assert.Contains(t, err.Error(), "no execution plan returned")

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_GetQueryExecutionPlan_ExecutionError(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	testQuery := "SELECT * FROM recipes"

	mock.ExpectQuery("EXPLAIN").
		WillReturnError(sql.ErrConnDone)

	plan, err := monitor.GetQueryExecutionPlan(context.Background(), testQuery)
	assert.Error(t, err)
	assert.Nil(t, plan)
	assert.Contains(t, err.Error(), "failed to get execution plan")

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_AnalyzeQueryPlan(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	tests := []struct {
		name string
		plan *ExecutionPlan
		want func(*testing.T, *PlanAnalysis)
	}{
		{
			name: "fast_index_scan",
			plan: &ExecutionPlan{
				Query:         "SELECT * FROM recipes WHERE cuisine_type = 'italian'",
				Plan:          `{"Plan": [{"Node Type": "Index Scan"}]}`,
				ExecutionTime: 45 * time.Millisecond,
				StartupCost:   0.5,
				TotalCost:     12.3,
				PlanRows:      20,
				ActualRows:    18,
				Timestamp:     time.Now(),
			},
			want: func(t *testing.T, analysis *PlanAnalysis) {
				assert.Equal(t, "low", analysis.CostAnalysis.CostRating)
				assert.False(t, analysis.CostAnalysis.IsExpensive)
				assert.True(t, analysis.Performance.MeetsTarget)
				assert.Equal(t, 0, len(analysis.Issues), "Should have no issues for fast, efficient query")
				assert.Greater(t, analysis.Performance.EstimateAccuracy, 0.8)
			},
		},
		{
			name: "slow_sequential_scan",
			plan: &ExecutionPlan{
				Query:         "SELECT * FROM recipes WHERE description LIKE '%pasta%'",
				Plan:          `{"Plan": [{"Node Type": "Seq Scan"}]}`,
				ExecutionTime: 350 * time.Millisecond,
				StartupCost:   0.0,
				TotalCost:     1500.0,
				PlanRows:      100,
				ActualRows:    5,
				Timestamp:     time.Now(),
			},
			want: func(t *testing.T, analysis *PlanAnalysis) {
				assert.Equal(t, "very_high", analysis.CostAnalysis.CostRating)
				assert.True(t, analysis.CostAnalysis.IsExpensive)
				assert.False(t, analysis.Performance.MeetsTarget)
				
				// Should detect multiple issues
				issueTypes := make(map[string]bool)
				for _, issue := range analysis.Issues {
					issueTypes[issue.Type] = true
				}
				assert.True(t, issueTypes["seq_scan"], "Should detect sequential scan")
				assert.True(t, issueTypes["high_cost"], "Should detect high cost")
				assert.True(t, issueTypes["slow_execution"], "Should detect slow execution")
				assert.True(t, issueTypes["estimate_accuracy"], "Should detect poor estimate accuracy")
				
				assert.Less(t, analysis.Performance.EstimateAccuracy, 0.5)
				assert.Contains(t, analysis.Recommendations, "Add index on frequently queried columns to eliminate sequential scans")
			},
		},
		{
			name: "medium_cost_acceptable_performance",
			plan: &ExecutionPlan{
				Query:         "SELECT * FROM recipes ORDER BY created_at DESC LIMIT 50",
				Plan:          `{"Plan": [{"Node Type": "Limit"}, {"Node Type": "Index Scan"}]}`,
				ExecutionTime: 120 * time.Millisecond,
				StartupCost:   1.2,
				TotalCost:     350.0,
				PlanRows:      50,
				ActualRows:    50,
				Timestamp:     time.Now(),
			},
			want: func(t *testing.T, analysis *PlanAnalysis) {
				assert.Equal(t, "medium", analysis.CostAnalysis.CostRating)
				assert.False(t, analysis.CostAnalysis.IsExpensive)
				assert.True(t, analysis.Performance.MeetsTarget)
				assert.Equal(t, 0, len(analysis.Issues), "Should have no critical issues")
				assert.Equal(t, 1.0, analysis.Performance.EstimateAccuracy)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			analysis, err := monitor.AnalyzeQueryPlan(context.Background(), tt.plan)
			require.NoError(t, err)
			require.NotNil(t, analysis)

			assert.Equal(t, tt.plan, analysis.Plan)
			assert.NotNil(t, analysis.CostAnalysis)
			assert.NotNil(t, analysis.Performance)

			tt.want(t, analysis)
		})
	}
}

func TestQueryExecutionMonitor_GetSlowQueriesFromLog(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	minDuration := 100 * time.Millisecond
	limit := 10

	// Mock pg_stat_statements query
	slowQueryRows := sqlmock.NewRows([]string{"query", "duration", "log_time"}).
		AddRow("SELECT * FROM recipes WHERE cuisine_type = 'italian'", 150.5, time.Now()).
		AddRow("SELECT COUNT(*) FROM recipes WHERE dietary_labels @> '[\"vegan\"]'", 220.3, time.Now().Add(-1*time.Hour))

	mock.ExpectQuery("SELECT.*FROM pg_stat_statements").
		WithArgs(int64(100), 10).
		WillReturnRows(slowQueryRows)

	slowQueries, err := monitor.GetSlowQueriesFromLog(context.Background(), minDuration, limit)
	require.NoError(t, err)
	assert.Len(t, slowQueries, 2)

	assert.Contains(t, slowQueries[0].Query, "italian")
	assert.Equal(t, time.Duration(150.5)*time.Millisecond, slowQueries[0].Duration)
	assert.Equal(t, "Available via EXPLAIN analysis", slowQueries[0].PlanSummary)

	assert.Contains(t, slowQueries[1].Query, "vegan")
	assert.Equal(t, time.Duration(220.3)*time.Millisecond, slowQueries[1].Duration)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_GetSlowQueriesFromLog_ExtensionNotAvailable(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)

	minDuration := 100 * time.Millisecond
	limit := 10

	// Mock pg_stat_statements failure (extension not installed)
	mock.ExpectQuery("SELECT.*FROM pg_stat_statements").
		WithArgs(int64(100), 10).
		WillReturnError(sql.ErrNoRows)

	slowQueries, err := monitor.GetSlowQueriesFromLog(context.Background(), minDuration, limit)
	require.NoError(t, err) // Should not error, just return empty
	assert.Len(t, slowQueries, 0)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryExecutionMonitor_analyzeCosts(t *testing.T) {
	monitor := &queryExecutionMonitor{}

	tests := []struct {
		name string
		plan *ExecutionPlan
		want *CostAnalysis
	}{
		{
			name: "low_cost",
			plan: &ExecutionPlan{
				StartupCost: 0.5,
				TotalCost:   45.0,
				PlanRows:    10,
			},
			want: &CostAnalysis{
				StartupCost:  0.5,
				TotalCost:    45.0,
				CostPerRow:   4.5,
				CostRating:   "low",
				IsExpensive:  false,
			},
		},
		{
			name: "medium_cost",
			plan: &ExecutionPlan{
				StartupCost: 2.1,
				TotalCost:   250.0,
				PlanRows:    50,
			},
			want: &CostAnalysis{
				StartupCost:  2.1,
				TotalCost:    250.0,
				CostPerRow:   5.0,
				CostRating:   "medium",
				IsExpensive:  false,
			},
		},
		{
			name: "high_cost",
			plan: &ExecutionPlan{
				StartupCost: 10.5,
				TotalCost:   800.0,
				PlanRows:    100,
			},
			want: &CostAnalysis{
				StartupCost:  10.5,
				TotalCost:    800.0,
				CostPerRow:   8.0,
				CostRating:   "high",
				IsExpensive:  false,
			},
		},
		{
			name: "very_high_cost",
			plan: &ExecutionPlan{
				StartupCost: 50.0,
				TotalCost:   5000.0,
				PlanRows:    200,
			},
			want: &CostAnalysis{
				StartupCost:  50.0,
				TotalCost:    5000.0,
				CostPerRow:   25.0,
				CostRating:   "very_high",
				IsExpensive:  true,
			},
		},
		{
			name: "expensive_threshold",
			plan: &ExecutionPlan{
				StartupCost: 5.0,
				TotalCost:   1200.0,
				PlanRows:    100,
			},
			want: &CostAnalysis{
				StartupCost:  5.0,
				TotalCost:    1200.0,
				CostPerRow:   12.0,
				CostRating:   "high",
				IsExpensive:  true,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := monitor.analyzeCosts(tt.plan)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestQueryExecutionMonitor_analyzePerformance(t *testing.T) {
	monitor := &queryExecutionMonitor{}

	tests := []struct {
		name string
		plan *ExecutionPlan
		want *PerformanceMetrics
	}{
		{
			name: "meets_target",
			plan: &ExecutionPlan{
				ExecutionTime: 150 * time.Millisecond,
				PlanRows:      25,
				ActualRows:    20,
			},
			want: &PerformanceMetrics{
				ExecutionTime:    150 * time.Millisecond,
				RowsEstimated:    25,
				RowsActual:       20,
				EstimateAccuracy: 0.8, // 20/25
				MeetsTarget:      true,
			},
		},
		{
			name: "exceeds_target",
			plan: &ExecutionPlan{
				ExecutionTime: 350 * time.Millisecond,
				PlanRows:      100,
				ActualRows:    5,
			},
			want: &PerformanceMetrics{
				ExecutionTime:    350 * time.Millisecond,
				RowsEstimated:    100,
				RowsActual:       5,
				EstimateAccuracy: 0.05, // 5/100
				MeetsTarget:      false,
			},
		},
		{
			name: "perfect_estimate",
			plan: &ExecutionPlan{
				ExecutionTime: 75 * time.Millisecond,
				PlanRows:      50,
				ActualRows:    50,
			},
			want: &PerformanceMetrics{
				ExecutionTime:    75 * time.Millisecond,
				RowsEstimated:    50,
				RowsActual:       50,
				EstimateAccuracy: 1.0,
				MeetsTarget:      true,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := monitor.analyzePerformance(tt.plan)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestQueryExecutionMonitor_analyzeIndexUsage(t *testing.T) {
	monitor := &queryExecutionMonitor{}

	tests := []struct {
		name string
		plan *ExecutionPlan
		want []IndexUsageDetail
	}{
		{
			name: "index_scan_detected",
			plan: &ExecutionPlan{
				Plan: `{"Plan": [{"Node Type": "Index Scan", "Index Name": "idx_recipes_cuisine"}]}`,
			},
			want: []IndexUsageDetail{
				{
					IndexName: "detected_from_plan",
					Used:      true,
					ScanType:  "index_scan",
				},
			},
		},
		{
			name: "sequential_scan_no_index",
			plan: &ExecutionPlan{
				Plan: `{"Plan": [{"Node Type": "Seq Scan"}]}`,
			},
			want: []IndexUsageDetail{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := monitor.analyzeIndexUsage(tt.plan)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestQueryExecutionMonitor_MinMaxHelpers(t *testing.T) {
	assert.Equal(t, int64(5), min(5, 10))
	assert.Equal(t, int64(5), min(10, 5))
	assert.Equal(t, int64(5), min(5, 5))

	assert.Equal(t, int64(10), max(5, 10))
	assert.Equal(t, int64(10), max(10, 5))
	assert.Equal(t, int64(5), max(5, 5))
}

// Integration test for complete monitoring workflow
func TestQueryExecutionMonitor_CompleteWorkflow(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryExecutionMonitor(db)
	ctx := context.Background()

	// 1. Enable logging
	mock.ExpectExec("LOAD 'auto_explain'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_min_duration = '100ms'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_analyze = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_verbose = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_buffers = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_timing = true").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_nested_statements = true").WillReturnResult(sqlmock.NewResult(0, 0))

	err := monitor.EnableQueryPlanLogging(ctx)
	require.NoError(t, err)

	// 2. Get execution plan
	testQuery := "SELECT * FROM recipes WHERE cuisine_type = $1 AND prep_time <= $2"
	mock.ExpectQuery("EXPLAIN").
		WithArgs("italian", 30).
		WillReturnRows(sqlmock.NewRows([]string{"QUERY PLAN"}).
			AddRow(`{"Plan": [{"Node Type": "Index Scan", "Startup Cost": 0.43, "Total Cost": 45.67, "Plan Rows": 25, "Actual Rows": 23}]}`))

	plan, err := monitor.GetQueryExecutionPlan(ctx, testQuery, "italian", 30)
	require.NoError(t, err)
	require.NotNil(t, plan)

	// 3. Analyze the plan
	analysis, err := monitor.AnalyzeQueryPlan(ctx, plan)
	require.NoError(t, err)
	require.NotNil(t, analysis)

	// 4. Disable logging
	mock.ExpectExec("SET auto_explain.log_min_duration = -1").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_analyze = false").WillReturnResult(sqlmock.NewResult(0, 0))

	err = monitor.DisableQueryPlanLogging(ctx)
	require.NoError(t, err)

	// Verify workflow results
	assert.Equal(t, testQuery, plan.Query)
	assert.Equal(t, plan, analysis.Plan)
	assert.NotNil(t, analysis.CostAnalysis)
	assert.NotNil(t, analysis.Performance)
	assert.True(t, analysis.Performance.MeetsTarget) // Should be under 200ms

	assert.NoError(t, mock.ExpectationsWereMet())
}