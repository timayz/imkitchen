package utils

import "time"

// RateLimiterInterface defines the interface for rate limiting
type RateLimiterInterface interface {
	Allow() bool
	Reset()
	SetLimit(limit int)
	GetCurrentCount() int
}

// SlowQueryAnalysis represents analysis of slow database queries
type SlowQueryAnalysis struct {
	QueryHash    string    `json:"query_hash"`
	Query        string    `json:"query"`
	ExecutionTime float64  `json:"execution_time"`
	Count        int       `json:"count"`
	LastSeen     time.Time `json:"last_seen"`
	Tables       []string  `json:"tables"`
	Operations   []string  `json:"operations"`
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
	Pattern     string    `json:"pattern"`
	Count       int       `json:"count"`
	AvgDuration float64   `json:"avg_duration"`
	LastSeen    time.Time `json:"last_seen"`
}

// ImplementationStep represents a step in optimization implementation
type ImplementationStep struct {
	StepNumber  int    `json:"step_number"`
	Description string `json:"description"`
	Command     string `json:"command,omitempty"`
	Expected    string `json:"expected"`
}