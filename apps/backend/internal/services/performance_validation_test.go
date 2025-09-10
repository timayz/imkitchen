package services

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestQueryExecutionMonitor_Core tests core functionality without external dependencies
func TestQueryExecutionMonitor_Core(t *testing.T) {
	t.Run("test_execution_plan_creation", func(t *testing.T) {
		plan := CreateTestExecutionPlan()
		require.NotNil(t, plan)
		
		assert.Equal(t, "SELECT * FROM recipes WHERE deleted_at IS NULL LIMIT 50", plan.Query)
		assert.Equal(t, 85*time.Millisecond, plan.ExecutionTime)
		assert.Equal(t, float64(0.43), plan.StartupCost)
		assert.Equal(t, float64(45.67), plan.TotalCost)
		assert.Equal(t, int64(25), plan.PlanRows)
		assert.Equal(t, int64(23), plan.ActualRows)
		assert.Contains(t, plan.Plan, "Index Scan")
	})

	t.Run("test_performance_targets", func(t *testing.T) {
		// Test plan that meets performance targets
		goodPlan := &ExecutionPlan{
			ExecutionTime: 150 * time.Millisecond,
			TotalCost:     500.0,
		}
		assert.True(t, TestPerformanceTargets(goodPlan), "Good plan should meet performance targets")

		// Test plan that fails execution time target
		slowPlan := &ExecutionPlan{
			ExecutionTime: 300 * time.Millisecond,
			TotalCost:     500.0,
		}
		assert.False(t, TestPerformanceTargets(slowPlan), "Slow plan should fail performance targets")

		// Test plan that fails cost target
		expensivePlan := &ExecutionPlan{
			ExecutionTime: 150 * time.Millisecond,
			TotalCost:     1500.0,
		}
		assert.False(t, TestPerformanceTargets(expensivePlan), "Expensive plan should fail performance targets")
	})

	t.Run("test_cost_analysis", func(t *testing.T) {
		monitor := &queryExecutionMonitor{}
		
		tests := []struct {
			name      string
			plan      *ExecutionPlan
			wantRating string
			wantExpensive bool
		}{
			{
				name: "low_cost",
				plan: &ExecutionPlan{StartupCost: 0.5, TotalCost: 45.0, PlanRows: 10},
				wantRating: "low",
				wantExpensive: false,
			},
			{
				name: "medium_cost", 
				plan: &ExecutionPlan{StartupCost: 2.0, TotalCost: 250.0, PlanRows: 50},
				wantRating: "medium",
				wantExpensive: false,
			},
			{
				name: "high_cost",
				plan: &ExecutionPlan{StartupCost: 5.0, TotalCost: 800.0, PlanRows: 100},
				wantRating: "high",
				wantExpensive: false,
			},
			{
				name: "very_high_cost",
				plan: &ExecutionPlan{StartupCost: 10.0, TotalCost: 3000.0, PlanRows: 200},
				wantRating: "very_high",
				wantExpensive: true,
			},
		}

		for _, tt := range tests {
			t.Run(tt.name, func(t *testing.T) {
				analysis := monitor.analyzeCosts(tt.plan)
				assert.Equal(t, tt.wantRating, analysis.CostRating)
				assert.Equal(t, tt.wantExpensive, analysis.IsExpensive)
			})
		}
	})

	t.Run("test_performance_metrics", func(t *testing.T) {
		monitor := &queryExecutionMonitor{}
		
		tests := []struct {
			name           string
			plan           *ExecutionPlan
			wantMeetsTarget bool
			wantAccuracy   float64
		}{
			{
				name: "meets_target_perfect_accuracy",
				plan: &ExecutionPlan{
					ExecutionTime: 150 * time.Millisecond,
					PlanRows:      50,
					ActualRows:    50,
				},
				wantMeetsTarget: true,
				wantAccuracy:   1.0,
			},
			{
				name: "exceeds_target_poor_accuracy",
				plan: &ExecutionPlan{
					ExecutionTime: 300 * time.Millisecond,
					PlanRows:      100,
					ActualRows:    10,
				},
				wantMeetsTarget: false,
				wantAccuracy:   0.1,
			},
		}

		for _, tt := range tests {
			t.Run(tt.name, func(t *testing.T) {
				metrics := monitor.analyzePerformance(tt.plan)
				assert.Equal(t, tt.wantMeetsTarget, metrics.MeetsTarget)
				assert.Equal(t, tt.wantAccuracy, metrics.EstimateAccuracy)
			})
		}
	})
}

// TestQueryAnalysisWorkflow tests the complete analysis workflow
func TestQueryAnalysisWorkflow(t *testing.T) {
	monitor := &queryExecutionMonitor{}
	ctx := context.Background()

	// Create test execution plan
	plan := &ExecutionPlan{
		Query:         "SELECT * FROM recipes WHERE cuisine_type = 'italian' AND prep_time <= 30",
		Plan:          `{"Plan": [{"Node Type": "Index Scan", "Index Name": "idx_recipes_cuisine_prep"}]}`,
		ExecutionTime: 120 * time.Millisecond,
		StartupCost:   0.8,
		TotalCost:     65.4,
		PlanRows:      20,
		ActualRows:    18,
		Timestamp:     time.Now(),
	}

	// Analyze the plan
	analysis, err := monitor.AnalyzeQueryPlan(ctx, plan)
	require.NoError(t, err)
	require.NotNil(t, analysis)

	// Verify analysis results
	assert.Equal(t, plan, analysis.Plan)
	assert.NotNil(t, analysis.CostAnalysis)
	assert.NotNil(t, analysis.Performance)

	// Should meet performance targets
	assert.True(t, analysis.Performance.MeetsTarget)
	assert.Equal(t, "low", analysis.CostAnalysis.CostRating)
	assert.False(t, analysis.CostAnalysis.IsExpensive)

	// Should have good estimate accuracy
	assert.Greater(t, analysis.Performance.EstimateAccuracy, 0.8)

	// Should have no critical issues
	criticalIssues := 0
	for _, issue := range analysis.Issues {
		if issue.Severity == "high" {
			criticalIssues++
		}
	}
	assert.Equal(t, 0, criticalIssues, "Should have no critical performance issues")
}

// TestIndexUsageAnalysis tests index usage detection
func TestIndexUsageAnalysis(t *testing.T) {
	monitor := &queryExecutionMonitor{}

	tests := []struct {
		name     string
		plan     *ExecutionPlan
		wantUsed bool
	}{
		{
			name: "index_scan_detected",
			plan: &ExecutionPlan{
				Plan: `{"Plan": [{"Node Type": "Index Scan", "Index Name": "idx_recipes_cuisine"}]}`,
			},
			wantUsed: true,
		},
		{
			name: "sequential_scan",
			plan: &ExecutionPlan{
				Plan: `{"Plan": [{"Node Type": "Seq Scan"}]}`,
			},
			wantUsed: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			indexUsage := monitor.analyzeIndexUsage(tt.plan)
			
			if tt.wantUsed {
				assert.Len(t, indexUsage, 1)
				assert.True(t, indexUsage[0].Used)
				assert.Equal(t, "index_scan", indexUsage[0].ScanType)
			} else {
				assert.Len(t, indexUsage, 0)
			}
		})
	}
}