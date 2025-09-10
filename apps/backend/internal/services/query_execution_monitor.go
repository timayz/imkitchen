package services

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/utils"
)

type QueryExecutionMonitor interface {
	EnableQueryPlanLogging(ctx context.Context) error
	DisableQueryPlanLogging(ctx context.Context) error
	GetQueryExecutionPlan(ctx context.Context, query string, args ...interface{}) (*ExecutionPlan, error)
	AnalyzeQueryPlan(ctx context.Context, plan *ExecutionPlan) (*PlanAnalysis, error)
	GetSlowQueriesFromLog(ctx context.Context, minDuration time.Duration, limit int) ([]SlowQuery, error)
}

type ExecutionPlan struct {
	Query         string                 `json:"query"`
	Plan          string                 `json:"plan"`
	PlanJSON      map[string]interface{} `json:"plan_json,omitempty"`
	ExecutionTime time.Duration          `json:"execution_time"`
	StartupCost   float64                `json:"startup_cost"`
	TotalCost     float64                `json:"total_cost"`
	PlanRows      int64                  `json:"plan_rows"`
	ActualRows    int64                  `json:"actual_rows"`
	Timestamp     time.Time              `json:"timestamp"`
}

type PlanAnalysis struct {
	Plan            *ExecutionPlan      `json:"plan"`
	Issues          []PlanIssue         `json:"issues"`
	Recommendations []string            `json:"recommendations"`
	IndexUsage      []IndexUsageDetail  `json:"index_usage"`
	CostAnalysis    *CostAnalysis       `json:"cost_analysis"`
	Performance     *PerformanceMetrics `json:"performance"`
}

type PlanIssue struct {
	Severity    string `json:"severity"` // "high", "medium", "low"
	Type        string `json:"type"`     // "seq_scan", "high_cost", "index_miss"
	Description string `json:"description"`
	Suggestion  string `json:"suggestion"`
}

type IndexUsageDetail struct {
	IndexName   string  `json:"index_name"`
	TableName   string  `json:"table_name"`
	Used        bool    `json:"used"`
	ScanType    string  `json:"scan_type"`
	Selectivity float64 `json:"selectivity"`
}

type CostAnalysis struct {
	StartupCost float64 `json:"startup_cost"`
	TotalCost   float64 `json:"total_cost"`
	CostPerRow  float64 `json:"cost_per_row"`
	IsExpensive bool    `json:"is_expensive"`
	CostRating  string  `json:"cost_rating"` // "low", "medium", "high", "very_high"
}

type PerformanceMetrics struct {
	ExecutionTime    time.Duration `json:"execution_time"`
	PlanningTime     time.Duration `json:"planning_time"`
	RowsEstimated    int64         `json:"rows_estimated"`
	RowsActual       int64         `json:"rows_actual"`
	EstimateAccuracy float64       `json:"estimate_accuracy"`
	MeetsTarget      bool          `json:"meets_target"` // <200ms target
}

type SlowQuery struct {
	Query       string        `json:"query"`
	Duration    time.Duration `json:"duration"`
	LogTime     time.Time     `json:"log_time"`
	PlanSummary string        `json:"plan_summary"`
}

type queryExecutionMonitor struct {
	db *gorm.DB
}

func NewQueryExecutionMonitor(db *gorm.DB) QueryExecutionMonitor {
	return &queryExecutionMonitor{
		db: db,
	}
}

func (m *queryExecutionMonitor) EnableQueryPlanLogging(ctx context.Context) error {
	// Enable auto_explain extension for automatic query plan logging
	queries := []string{
		"LOAD 'auto_explain'",
		"SET auto_explain.log_min_duration = '100ms'",
		"SET auto_explain.log_analyze = true",
		"SET auto_explain.log_verbose = true",
		"SET auto_explain.log_buffers = true",
		"SET auto_explain.log_timing = true",
		"SET auto_explain.log_nested_statements = true",
	}

	for _, query := range queries {
		if err := m.db.WithContext(ctx).Exec(query).Error; err != nil {
			return fmt.Errorf("failed to enable query plan logging: %w", err)
		}
	}

	return nil
}

func (m *queryExecutionMonitor) DisableQueryPlanLogging(ctx context.Context) error {
	queries := []string{
		"SET auto_explain.log_min_duration = -1",
		"SET auto_explain.log_analyze = false",
	}

	for _, query := range queries {
		if err := m.db.WithContext(ctx).Exec(query).Error; err != nil {
			return fmt.Errorf("failed to disable query plan logging: %w", err)
		}
	}

	return nil
}

func (m *queryExecutionMonitor) GetQueryExecutionPlan(ctx context.Context, query string, args ...interface{}) (*ExecutionPlan, error) {
	plan := &ExecutionPlan{
		Query:     query,
		Timestamp: time.Now(),
	}

	// Get EXPLAIN (ANALYZE, BUFFERS, VERBOSE, FORMAT JSON) for the query
	explainQuery := fmt.Sprintf("EXPLAIN (ANALYZE true, BUFFERS true, VERBOSE true, FORMAT JSON) %s", query)

	startTime := time.Now()

	var result []map[string]interface{}
	err := m.db.WithContext(ctx).Raw(explainQuery, args...).Scan(&result).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get execution plan: %w", err)
	}

	plan.ExecutionTime = time.Since(startTime)

	if len(result) == 0 {
		return nil, fmt.Errorf("no execution plan returned")
	}

	// Parse the JSON plan
	if planData, ok := result[0]["QUERY PLAN"]; ok {
		planBytes, err := json.Marshal(planData)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal plan data: %w", err)
		}

		plan.Plan = string(planBytes)

		// Parse JSON plan for detailed analysis
		var planJSON map[string]interface{}
		if err := json.Unmarshal(planBytes, &planJSON); err == nil {
			plan.PlanJSON = planJSON

			// Extract costs and rows from plan
			if planArray, ok := planJSON["Plan"].([]interface{}); ok && len(planArray) > 0 {
				if rootPlan, ok := planArray[0].(map[string]interface{}); ok {
					if startupCost, ok := rootPlan["Startup Cost"].(float64); ok {
						plan.StartupCost = startupCost
					}
					if totalCost, ok := rootPlan["Total Cost"].(float64); ok {
						plan.TotalCost = totalCost
					}
					if planRows, ok := rootPlan["Plan Rows"].(float64); ok {
						plan.PlanRows = int64(planRows)
					}
					if actualRows, ok := rootPlan["Actual Rows"].(float64); ok {
						plan.ActualRows = int64(actualRows)
					}
				}
			}
		}
	}

	return plan, nil
}

func (m *queryExecutionMonitor) AnalyzeQueryPlan(ctx context.Context, plan *ExecutionPlan) (*PlanAnalysis, error) {
	analysis := &PlanAnalysis{
		Plan:            plan,
		Issues:          []PlanIssue{},
		Recommendations: []string{},
		IndexUsage:      []IndexUsageDetail{},
	}

	// Analyze costs
	analysis.CostAnalysis = m.analyzeCosts(plan)

	// Analyze performance metrics
	analysis.Performance = m.analyzePerformance(plan)

	// Parse plan text for common issues
	planText := strings.ToLower(plan.Plan)

	// Check for sequential scans
	if strings.Contains(planText, "seq scan") {
		analysis.Issues = append(analysis.Issues, PlanIssue{
			Severity:    "medium",
			Type:        "seq_scan",
			Description: "Query uses sequential scan which may be inefficient for large tables",
			Suggestion:  "Consider adding appropriate indices to avoid full table scans",
		})
		analysis.Recommendations = append(analysis.Recommendations,
			"Add index on frequently queried columns to eliminate sequential scans")
	}

	// Check for high cost queries
	if plan.TotalCost > 1000 {
		analysis.Issues = append(analysis.Issues, PlanIssue{
			Severity:    "high",
			Type:        "high_cost",
			Description: fmt.Sprintf("Query has high total cost: %.2f", plan.TotalCost),
			Suggestion:  "Review query structure and consider index optimization",
		})
	}

	// Check execution time against target
	if plan.ExecutionTime > 200*time.Millisecond {
		analysis.Issues = append(analysis.Issues, PlanIssue{
			Severity:    "high",
			Type:        "slow_execution",
			Description: fmt.Sprintf("Query execution time %v exceeds 200ms target", plan.ExecutionTime),
			Suggestion:  "Optimize query or add performance indices",
		})
	}

	// Check for inaccurate row estimates
	if plan.PlanRows > 0 && plan.ActualRows > 0 {
		estimateAccuracy := float64(utils.MinInt64(plan.PlanRows, plan.ActualRows)) / float64(utils.MaxInt64(plan.PlanRows, plan.ActualRows))
		if estimateAccuracy < 0.5 {
			analysis.Issues = append(analysis.Issues, PlanIssue{
				Severity:    "medium",
				Type:        "estimate_accuracy",
				Description: fmt.Sprintf("Poor row estimate accuracy: %.2f", estimateAccuracy),
				Suggestion:  "Consider updating table statistics with ANALYZE",
			})
			analysis.Recommendations = append(analysis.Recommendations,
				"Run ANALYZE on relevant tables to improve query planning")
		}
	}

	// Analyze index usage from plan
	analysis.IndexUsage = m.analyzeIndexUsage(plan)

	return analysis, nil
}

func (m *queryExecutionMonitor) GetSlowQueriesFromLog(ctx context.Context, minDuration time.Duration, limit int) ([]SlowQuery, error) {
	var slowQueries []SlowQuery

	// Query PostgreSQL logs for slow queries
	// Note: This requires log_statement and log_min_duration_statement to be configured
	query := `
		SELECT 
			query_text as query,
			execution_time as duration,
			log_time
		FROM pg_stat_statements 
		WHERE mean_exec_time >= ? 
		AND query_text LIKE '%recipe%'
		ORDER BY mean_exec_time DESC
		LIMIT ?
	`

	type logEntry struct {
		Query    string    `json:"query"`
		Duration float64   `json:"duration"` // milliseconds
		LogTime  time.Time `json:"log_time"`
	}

	var entries []logEntry
	err := m.db.WithContext(ctx).Raw(query, minDuration.Milliseconds(), limit).Scan(&entries).Error
	if err != nil {
		// If pg_stat_statements is not available, return empty result
		return slowQueries, nil
	}

	for _, entry := range entries {
		slowQueries = append(slowQueries, SlowQuery{
			Query:       entry.Query,
			Duration:    time.Duration(entry.Duration) * time.Millisecond,
			LogTime:     entry.LogTime,
			PlanSummary: "Available via EXPLAIN analysis",
		})
	}

	return slowQueries, nil
}

func (m *queryExecutionMonitor) analyzeCosts(plan *ExecutionPlan) *CostAnalysis {
	analysis := &CostAnalysis{
		StartupCost: plan.StartupCost,
		TotalCost:   plan.TotalCost,
	}

	if plan.PlanRows > 0 {
		analysis.CostPerRow = plan.TotalCost / float64(plan.PlanRows)
	}

	// Determine cost rating
	switch {
	case plan.TotalCost < 100:
		analysis.CostRating = "low"
	case plan.TotalCost < 500:
		analysis.CostRating = "medium"
	case plan.TotalCost < 2000:
		analysis.CostRating = "high"
	default:
		analysis.CostRating = "very_high"
		analysis.IsExpensive = true
	}

	if plan.TotalCost > 1000 {
		analysis.IsExpensive = true
	}

	return analysis
}

func (m *queryExecutionMonitor) analyzePerformance(plan *ExecutionPlan) *PerformanceMetrics {
	metrics := &PerformanceMetrics{
		ExecutionTime: plan.ExecutionTime,
		RowsEstimated: plan.PlanRows,
		RowsActual:    plan.ActualRows,
		MeetsTarget:   plan.ExecutionTime <= 200*time.Millisecond,
	}

	// Calculate estimate accuracy
	if plan.PlanRows > 0 && plan.ActualRows > 0 {
		metrics.EstimateAccuracy = float64(utils.MinInt64(plan.PlanRows, plan.ActualRows)) / float64(utils.MaxInt64(plan.PlanRows, plan.ActualRows))
	}

	return metrics
}

func (m *queryExecutionMonitor) analyzeIndexUsage(plan *ExecutionPlan) []IndexUsageDetail {
	var indexUsage []IndexUsageDetail

	// Parse plan text for index usage information
	planText := strings.ToLower(plan.Plan)

	// Look for index scan patterns
	if strings.Contains(planText, "index scan") {
		// Extract index names from plan (simplified)
		// In a full implementation, you'd parse the JSON plan structure
		indexUsage = append(indexUsage, IndexUsageDetail{
			IndexName: "detected_from_plan",
			Used:      true,
			ScanType:  "index_scan",
		})
	}

	return indexUsage
}

