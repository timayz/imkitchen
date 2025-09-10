package models

import "time"

// SlowQueryAnalysis represents analysis of slow database queries
type SlowQueryAnalysis struct {
	Query           string        `json:"query"`
	QueryPattern    string        `json:"query_pattern"`
	Count           int64         `json:"count"`
	TotalTime       time.Duration `json:"total_time"`
	AverageTime     time.Duration `json:"average_time"`
	MinTime         time.Duration `json:"min_time"`
	MaxTime         time.Duration `json:"max_time"`
	LastSeen        time.Time     `json:"last_seen"`
	ExecutionPlan   *ExecutionPlan `json:"execution_plan,omitempty"`
	Recommendations []string      `json:"recommendations"`
	Severity        string        `json:"severity"` // "critical", "high", "medium", "low"
}

// IndexRecommendation represents a database index recommendation
type IndexRecommendation struct {
	TableName   string   `json:"table_name"`
	Columns     []string `json:"columns"`
	IndexType   string   `json:"index_type"`
	Reason      string   `json:"reason"`
	Impact      string   `json:"impact"`
	Priority    int      `json:"priority"`
}

// QueryPattern represents a pattern in query execution
type QueryPattern struct {
	Pattern         string        `json:"pattern"`
	Count           int64         `json:"count"`
	AverageTime     time.Duration `json:"average_time"`
	TotalTime       time.Duration `json:"total_time"`
	Queries         []string      `json:"example_queries"`
	Severity        string        `json:"severity"`
}

// ImplementationStep represents a step in optimization implementation
type ImplementationStep struct {
	StepNumber  int    `json:"step_number"`
	Description string `json:"description"`
	Command     string `json:"command,omitempty"`
	Expected    string `json:"expected"`
}

// QueryRewriteRecommendation represents a query rewrite suggestion
type QueryRewriteRecommendation struct {
	OriginalQuery   string `json:"original_query"`
	RewrittenQuery  string `json:"rewritten_query"`
	Reason          string `json:"reason"`
	ExpectedGain    string `json:"expected_gain"`
}

// IndexUsageDetail represents detailed index usage information
type IndexUsageDetail struct {
	IndexName     string  `json:"index_name"`
	TableName     string  `json:"table_name"`
	Columns       []string `json:"columns"`
	UsageCount    int64   `json:"usage_count"`
	LastUsed      time.Time `json:"last_used"`
	SelectCount   int64   `json:"select_count"`
	InsertCount   int64   `json:"insert_count"`
	UpdateCount   int64   `json:"update_count"`
	DeleteCount   int64   `json:"delete_count"`
	IndexSize     int64   `json:"index_size"`
	Efficiency    float64 `json:"efficiency"`
}

// PerformanceMetrics represents query performance metrics
type PerformanceMetrics struct {
	QueryTime        time.Duration `json:"query_time"`
	PlanTime         time.Duration `json:"plan_time"`
	ExecuteTime      time.Duration `json:"execute_time"`
	BuffersHit       int64        `json:"buffers_hit"`
	BuffersRead      int64        `json:"buffers_read"`
	BuffersDirtied   int64        `json:"buffers_dirtied"`
	EstimateAccuracy float64      `json:"estimate_accuracy"`
	RowsReturned     int64        `json:"rows_returned"`
	RowsExamined     int64        `json:"rows_examined"`
}

// SlowQueryReport represents a report of slow queries
type SlowQueryReport struct {
	ReportTime        time.Time           `json:"report_time"`
	TotalQueries      int64              `json:"total_queries"`
	SlowQueries       int64              `json:"slow_queries"`
	AverageQueryTime  time.Duration      `json:"average_query_time"`
	TopSlowQueries    []SlowQueryAnalysis `json:"top_slow_queries"`
	QueryPatterns     []QueryPattern     `json:"query_patterns"`
	Recommendations   []string           `json:"recommendations"`
	PerformanceGains  map[string]string  `json:"performance_gains"`
	DatabaseHealth    string             `json:"database_health"`
	Severity          string             `json:"severity"`
}

// ExecutionPlan represents a database query execution plan
type ExecutionPlan struct {
	NodeType         string          `json:"node_type"`
	RelationName     string          `json:"relation_name,omitempty"`
	Alias            string          `json:"alias,omitempty"`
	StartupCost      float64         `json:"startup_cost"`
	TotalCost        float64         `json:"total_cost"`
	PlanRows         int64           `json:"plan_rows"`
	PlanWidth        int64           `json:"plan_width"`
	ActualRows       int64           `json:"actual_rows,omitempty"`
	ActualLoops      int64           `json:"actual_loops,omitempty"`
	ActualTotalTime  float64         `json:"actual_total_time,omitempty"`
	Plans            []ExecutionPlan `json:"plans,omitempty"`
	IndexName        string          `json:"index_name,omitempty"`
	JoinType         string          `json:"join_type,omitempty"`
	SortKey          []string        `json:"sort_key,omitempty"`
	Filter           string          `json:"filter,omitempty"`
	IndexCondition   string          `json:"index_condition,omitempty"`
	WorkersPlanned   int64           `json:"workers_planned,omitempty"`
	WorkersLaunched  int64           `json:"workers_launched,omitempty"`
}