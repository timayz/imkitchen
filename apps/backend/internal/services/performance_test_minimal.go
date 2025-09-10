// +build test

package services

// Simple test to verify query execution monitor functionality without import cycles
import (
	"context"
	"time"
	"gorm.io/gorm"
)

// TestableQueryExecutionMonitor provides a simple test interface
type TestableQueryExecutionMonitor struct {
	monitor QueryExecutionMonitor
}

func NewTestableQueryExecutionMonitor(db *gorm.DB) *TestableQueryExecutionMonitor {
	return &TestableQueryExecutionMonitor{
		monitor: NewQueryExecutionMonitor(db),
	}
}

func (t *TestableQueryExecutionMonitor) TestEnableLogging(ctx context.Context) error {
	return t.monitor.EnableQueryPlanLogging(ctx)
}

func (t *TestableQueryExecutionMonitor) TestDisableLogging(ctx context.Context) error {
	return t.monitor.DisableQueryPlanLogging(ctx)
}

func (t *TestableQueryExecutionMonitor) TestGetExecutionPlan(ctx context.Context, query string) (*ExecutionPlan, error) {
	return t.monitor.GetQueryExecutionPlan(ctx, query)
}

func (t *TestableQueryExecutionMonitor) TestAnalyzePlan(ctx context.Context, plan *ExecutionPlan) (*PlanAnalysis, error) {
	return t.monitor.AnalyzeQueryPlan(ctx, plan)
}

// Simplified test execution plan for validation
func CreateTestExecutionPlan() *ExecutionPlan {
	return &ExecutionPlan{
		Query:         "SELECT * FROM recipes WHERE deleted_at IS NULL LIMIT 50",
		Plan:          `{"Plan": [{"Node Type": "Index Scan", "Startup Cost": 0.43, "Total Cost": 45.67}]}`,
		ExecutionTime: 85 * time.Millisecond,
		StartupCost:   0.43,
		TotalCost:     45.67,
		PlanRows:      25,
		ActualRows:    23,
		Timestamp:     time.Now(),
	}
}

// Test performance targets
func TestPerformanceTargets(plan *ExecutionPlan) bool {
	return plan.ExecutionTime <= 200*time.Millisecond && 
		   plan.TotalCost <= 1000.0
}