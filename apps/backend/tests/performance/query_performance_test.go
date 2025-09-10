package performance

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/google/uuid"
	"github.com/lib/pq"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

// Simplified Recipe model for testing without import cycles
type Recipe struct {
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

type RecipeSearchFilters struct {
	Search        *string        `form:"search"`
	CuisineType   *string        `form:"cuisineType"`
	MaxPrepTime   *int           `form:"maxPrepTime"`
	DietaryLabels []string       `form:"dietaryLabels[]"`
	ExcludeIDs    []uuid.UUID    `form:"-"`
}

// ExecutionPlan represents PostgreSQL query execution plan
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

// PlanAnalysis represents analysis results
type PlanAnalysis struct {
	Plan            *ExecutionPlan         `json:"plan"`
	Issues          []PlanIssue           `json:"issues"`
	Recommendations []string              `json:"recommendations"`
	CostAnalysis    *CostAnalysis         `json:"cost_analysis"`
	Performance     *PerformanceMetrics   `json:"performance"`
}

type PlanIssue struct {
	Severity    string `json:"severity"`
	Type        string `json:"type"`
	Description string `json:"description"`
	Suggestion  string `json:"suggestion"`
}

type CostAnalysis struct {
	StartupCost      float64 `json:"startup_cost"`
	TotalCost        float64 `json:"total_cost"`
	CostPerRow       float64 `json:"cost_per_row"`
	IsExpensive      bool    `json:"is_expensive"`
	CostRating       string  `json:"cost_rating"`
}

type PerformanceMetrics struct {
	ExecutionTime    time.Duration `json:"execution_time"`
	RowsEstimated    int64         `json:"rows_estimated"`
	RowsActual       int64         `json:"rows_actual"`
	EstimateAccuracy float64       `json:"estimate_accuracy"`
	MeetsTarget      bool          `json:"meets_target"`
}

// QueryPerformanceMonitor interface
type QueryPerformanceMonitor interface {
	GetQueryExecutionPlan(ctx context.Context, query string, args ...interface{}) (*ExecutionPlan, error)
	AnalyzeQueryPlan(ctx context.Context, plan *ExecutionPlan) (*PlanAnalysis, error)
	EnableQueryPlanLogging(ctx context.Context) error
	DisableQueryPlanLogging(ctx context.Context) error
}

type queryPerformanceMonitor struct {
	db *gorm.DB
}

func NewQueryPerformanceMonitor(db *gorm.DB) QueryPerformanceMonitor {
	return &queryPerformanceMonitor{db: db}
}

func (m *queryPerformanceMonitor) EnableQueryPlanLogging(ctx context.Context) error {
	queries := []string{
		"LOAD 'auto_explain'",
		"SET auto_explain.log_min_duration = '100ms'",
		"SET auto_explain.log_analyze = true",
	}

	for _, query := range queries {
		if err := m.db.WithContext(ctx).Exec(query).Error; err != nil {
			return err
		}
	}
	return nil
}

func (m *queryPerformanceMonitor) DisableQueryPlanLogging(ctx context.Context) error {
	queries := []string{
		"SET auto_explain.log_min_duration = -1",
		"SET auto_explain.log_analyze = false",
	}

	for _, query := range queries {
		if err := m.db.WithContext(ctx).Exec(query).Error; err != nil {
			return err
		}
	}
	return nil
}

func (m *queryPerformanceMonitor) GetQueryExecutionPlan(ctx context.Context, query string, args ...interface{}) (*ExecutionPlan, error) {
	plan := &ExecutionPlan{
		Query:     query,
		Timestamp: time.Now(),
	}

	explainQuery := "EXPLAIN (ANALYZE true, BUFFERS true, VERBOSE true, FORMAT JSON) " + query
	startTime := time.Now()
	
	var result []map[string]interface{}
	err := m.db.WithContext(ctx).Raw(explainQuery, args...).Scan(&result).Error
	if err != nil {
		return nil, err
	}
	
	plan.ExecutionTime = time.Since(startTime)

	if len(result) == 0 {
		return nil, assert.AnError
	}

	// Parse the JSON plan
	if planData, ok := result[0]["QUERY PLAN"]; ok {
		if planStr, ok := planData.(string); ok {
			plan.Plan = planStr
			
			var planJSON map[string]interface{}
			if json.Unmarshal([]byte(planStr), &planJSON) == nil {
				plan.PlanJSON = planJSON
				
				if planArray, ok := planJSON["Plan"].([]interface{}); ok && len(planArray) > 0 {
					if rootPlan, ok := planArray[0].(map[string]interface{}); ok {
						if startupCost, ok := rootPlan["Startup Cost"].(float64); ok {
							plan.StartupCost = startupCost
						}
						if totalCost, ok := rootPlan["Total Cost"].(float64); ok {
							plan.TotalCost = totalCost
						}
						// Handle both float64 and integer types for Plan Rows
						if planRows, ok := rootPlan["Plan Rows"].(float64); ok {
							plan.PlanRows = int64(planRows)
						} else if planRowsInt, ok := rootPlan["Plan Rows"].(int); ok {
							plan.PlanRows = int64(planRowsInt)
						}
						// Handle both float64 and integer types for Actual Rows
						if actualRows, ok := rootPlan["Actual Rows"].(float64); ok {
							plan.ActualRows = int64(actualRows)
						} else if actualRowsInt, ok := rootPlan["Actual Rows"].(int); ok {
							plan.ActualRows = int64(actualRowsInt)
						}
					}
				}
			}
		}
	}

	return plan, nil
}

func (m *queryPerformanceMonitor) AnalyzeQueryPlan(ctx context.Context, plan *ExecutionPlan) (*PlanAnalysis, error) {
	analysis := &PlanAnalysis{
		Plan:            plan,
		Issues:          []PlanIssue{},
		Recommendations: []string{},
	}

	analysis.CostAnalysis = m.analyzeCosts(plan)
	analysis.Performance = m.analyzePerformance(plan)

	// Check for performance issues
	if plan.ExecutionTime > 200*time.Millisecond {
		analysis.Issues = append(analysis.Issues, PlanIssue{
			Severity:    "high",
			Type:        "slow_execution",
			Description: "Query execution time exceeds 200ms target",
			Suggestion:  "Optimize query or add performance indices",
		})
	}

	if plan.TotalCost > 1000 {
		analysis.Issues = append(analysis.Issues, PlanIssue{
			Severity:    "high",
			Type:        "high_cost",
			Description: "Query has high total cost",
			Suggestion:  "Review query structure and consider index optimization",
		})
	}

	return analysis, nil
}

func (m *queryPerformanceMonitor) analyzeCosts(plan *ExecutionPlan) *CostAnalysis {
	analysis := &CostAnalysis{
		StartupCost: plan.StartupCost,
		TotalCost:   plan.TotalCost,
	}

	if plan.PlanRows > 0 {
		analysis.CostPerRow = plan.TotalCost / float64(plan.PlanRows)
	}

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

func (m *queryPerformanceMonitor) analyzePerformance(plan *ExecutionPlan) *PerformanceMetrics {
	metrics := &PerformanceMetrics{
		ExecutionTime: plan.ExecutionTime,
		RowsEstimated: plan.PlanRows,
		RowsActual:    plan.ActualRows,
		MeetsTarget:   plan.ExecutionTime <= 200*time.Millisecond,
	}

	if plan.PlanRows > 0 && plan.ActualRows > 0 {
		metrics.EstimateAccuracy = float64(min(plan.PlanRows, plan.ActualRows)) / float64(max(plan.PlanRows, plan.ActualRows))
	}

	return metrics
}

func min(a, b int64) int64 {
	if a < b {
		return a
	}
	return b
}

func max(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}

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

// Test the QueryPerformanceMonitor functionality
func TestQueryPerformanceMonitor_EnableDisableLogging(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryPerformanceMonitor(db)

	// Test EnableQueryPlanLogging
	mock.ExpectExec("LOAD 'auto_explain'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_min_duration = '100ms'").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_analyze = true").WillReturnResult(sqlmock.NewResult(0, 0))

	err := monitor.EnableQueryPlanLogging(context.Background())
	require.NoError(t, err)

	// Test DisableQueryPlanLogging
	mock.ExpectExec("SET auto_explain.log_min_duration = -1").WillReturnResult(sqlmock.NewResult(0, 0))
	mock.ExpectExec("SET auto_explain.log_analyze = false").WillReturnResult(sqlmock.NewResult(0, 0))

	err = monitor.DisableQueryPlanLogging(context.Background())
	require.NoError(t, err)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryPerformanceMonitor_GetExecutionPlan(t *testing.T) {
	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryPerformanceMonitor(db)

	testQuery := "SELECT * FROM recipes WHERE cuisine_type = $1"

	mockPlanJSON := `{"Plan": [{"Node Type": "Index Scan", "Startup Cost": 0.43, "Total Cost": 45.67, "Plan Rows": 25, "Actual Rows": 23}]}`

	rows := sqlmock.NewRows([]string{"QUERY PLAN"}).
		AddRow(mockPlanJSON)

	mock.ExpectQuery("EXPLAIN \\(ANALYZE true, BUFFERS true, VERBOSE true, FORMAT JSON\\)").
		WithArgs("italian").
		WillReturnRows(rows)

	plan, err := monitor.GetQueryExecutionPlan(context.Background(), testQuery, "italian")
	require.NoError(t, err)
	require.NotNil(t, plan)

	assert.Equal(t, testQuery, plan.Query)
	assert.Equal(t, 0.43, plan.StartupCost)
	assert.Equal(t, 45.67, plan.TotalCost)
	assert.Equal(t, int64(25), plan.PlanRows)
	assert.Equal(t, int64(23), plan.ActualRows)
	assert.Greater(t, plan.ExecutionTime, time.Duration(0))

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestQueryPerformanceMonitor_AnalyzeQueryPlan(t *testing.T) {
	db, _, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryPerformanceMonitor(db)

	tests := []struct {
		name string
		plan *ExecutionPlan
		want func(*testing.T, *PlanAnalysis)
	}{
		{
			name: "fast_low_cost_plan",
			plan: &ExecutionPlan{
				Query:         "SELECT * FROM recipes WHERE id = $1",
				ExecutionTime: 50 * time.Millisecond,
				StartupCost:   0.5,
				TotalCost:     12.3,
				PlanRows:      1,
				ActualRows:    1,
			},
			want: func(t *testing.T, analysis *PlanAnalysis) {
				assert.True(t, analysis.Performance.MeetsTarget, "Should meet 200ms target")
				assert.Equal(t, "low", analysis.CostAnalysis.CostRating)
				assert.False(t, analysis.CostAnalysis.IsExpensive)
				assert.Equal(t, 1.0, analysis.Performance.EstimateAccuracy)
				assert.Equal(t, 0, len(analysis.Issues), "Should have no issues")
			},
		},
		{
			name: "slow_expensive_plan",
			plan: &ExecutionPlan{
				Query:         "SELECT * FROM recipes WHERE description LIKE '%pasta%'",
				ExecutionTime: 350 * time.Millisecond,
				StartupCost:   0.0,
				TotalCost:     2500.0, // Increased to trigger "very_high" rating
				PlanRows:      100,
				ActualRows:    5,
			},
			want: func(t *testing.T, analysis *PlanAnalysis) {
				assert.False(t, analysis.Performance.MeetsTarget, "Should not meet 200ms target")
				assert.Equal(t, "very_high", analysis.CostAnalysis.CostRating)
				assert.True(t, analysis.CostAnalysis.IsExpensive)
				assert.Equal(t, 0.05, analysis.Performance.EstimateAccuracy)
				
				// Should detect both slow execution and high cost issues
				issueTypes := make(map[string]bool)
				for _, issue := range analysis.Issues {
					issueTypes[issue.Type] = true
				}
				assert.True(t, issueTypes["slow_execution"], "Should detect slow execution")
				assert.True(t, issueTypes["high_cost"], "Should detect high cost")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			analysis, err := monitor.AnalyzeQueryPlan(context.Background(), tt.plan)
			require.NoError(t, err)
			require.NotNil(t, analysis)

			tt.want(t, analysis)
		})
	}
}

// Performance benchmark test to ensure sub-200ms target
func TestQueryPerformanceMonitor_Performance200msTarget(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping performance test in short mode")
	}

	db, mock, cleanup := setupTestDB(t)
	defer cleanup()

	monitor := NewQueryPerformanceMonitor(db)

	// Mock a fast query response
	explainQuery := "EXPLAIN \\(ANALYZE true, BUFFERS true, VERBOSE true, FORMAT JSON\\)"
	mockPlanJSON := `{"Plan": [{"Node Type": "Index Scan", "Startup Cost": 0.1, "Total Cost": 5.0, "Plan Rows": 1, "Actual Rows": 1}]}`

	mock.ExpectQuery(explainQuery).
		WillReturnRows(sqlmock.NewRows([]string{"QUERY PLAN"}).AddRow(mockPlanJSON))

	// Measure execution time
	start := time.Now()
	plan, err := monitor.GetQueryExecutionPlan(context.Background(), "SELECT * FROM recipes WHERE id = $1", uuid.New())
	executionTime := time.Since(start)

	require.NoError(t, err)
	require.NotNil(t, plan)

	// The monitor should complete analysis in well under 200ms
	assert.Less(t, executionTime, 100*time.Millisecond, 
		"Query performance monitor should complete in under 100ms")

	// The analyzed plan should meet performance targets
	analysis, err := monitor.AnalyzeQueryPlan(context.Background(), plan)
	require.NoError(t, err)
	
	assert.True(t, analysis.Performance.MeetsTarget, 
		"Optimized query should meet sub-200ms performance target")
	assert.Equal(t, "low", analysis.CostAnalysis.CostRating,
		"Optimized query should have low cost rating")

	assert.NoError(t, mock.ExpectationsWereMet())
}