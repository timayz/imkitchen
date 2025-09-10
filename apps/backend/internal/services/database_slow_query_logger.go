package services

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"gorm.io/gorm"
)

type DatabaseSlowQueryLogger interface {
	// Configuration and lifecycle
	Start(ctx context.Context) error
	Stop(ctx context.Context) error
	IsRunning() bool

	// Logging configuration
	SetSlowQueryThreshold(threshold time.Duration) error
	GetSlowQueryThreshold() time.Duration
	EnableDetailedLogging(enabled bool) error
	
	// Query analysis and reporting
	GetSlowQueries(ctx context.Context, since time.Time, limit int) ([]DetectedSlowQuery, error)
	AnalyzeSlowQuery(ctx context.Context, query *DetectedSlowQuery) (*SlowQueryAnalysis, error)
	GetSlowQueryStatistics(ctx context.Context, since time.Time) (*SlowQueryStatistics, error)
	
	// Real-time monitoring
	RegisterSlowQueryCallback(callback SlowQueryCallback)
	UnregisterSlowQueryCallback()
	
	// PostgreSQL-specific functionality
	EnablePgStatStatements(ctx context.Context) error
	GetTopSlowQueries(ctx context.Context, limit int) ([]PgStatQuery, error)
	ResetQueryStatistics(ctx context.Context) error
}

type DetectedSlowQuery struct {
	QueryID           string                 `json:"query_id"`
	QueryText         string                 `json:"query_text"`
	QueryNormalized   string                 `json:"query_normalized"`
	ExecutionTime     time.Duration          `json:"execution_time"`
	DetectedAt        time.Time              `json:"detected_at"`
	PlanCost          float64                `json:"plan_cost"`
	RowsExamined      int64                  `json:"rows_examined"`
	RowsReturned      int64                  `json:"rows_returned"`
	Database          string                 `json:"database"`
	User              string                 `json:"user"`
	ApplicationName   string                 `json:"application_name"`
	QueryParameters   map[string]interface{} `json:"query_parameters,omitempty"`
	ExecutionContext  *QueryExecutionContext `json:"execution_context,omitempty"`
	Severity          string                 `json:"severity"` // "warning", "critical", "severe"
	CategoryTags      []string               `json:"category_tags"`
}

type QueryExecutionContext struct {
	ConnectionID     string            `json:"connection_id"`
	SessionID        string            `json:"session_id"`
	RequestID        string            `json:"request_id"`
	UserAgent        string            `json:"user_agent"`
	ClientIP         string            `json:"client_ip"`
	QueryStartTime   time.Time         `json:"query_start_time"`
	LockWaitTime     time.Duration     `json:"lock_wait_time"`
	IOWaitTime       time.Duration     `json:"io_wait_time"`
	CPUTime          time.Duration     `json:"cpu_time"`
	TempFiles        int               `json:"temp_files"`
	TempBytesRead    int64             `json:"temp_bytes_read"`
	TempBytesWritten int64             `json:"temp_bytes_written"`
	SharedBlksHit    int64             `json:"shared_blks_hit"`
	SharedBlksRead   int64             `json:"shared_blks_read"`
	LocalBlksRead    int64             `json:"local_blks_read"`
	LocalBlksWritten int64             `json:"local_blks_written"`
	AdditionalMetrics map[string]interface{} `json:"additional_metrics,omitempty"`
}

type DetailedSlowQueryAnalysis struct {
	Query               *DetectedSlowQuery      `json:"query"`
	PerformanceImpact   *PerformanceImpact      `json:"performance_impact"`
	OptimizationSuggestions []OptimizationSuggestion `json:"optimization_suggestions"`
	IndexRecommendations    []DetailedIndexRecommendation     `json:"index_recommendations"`
	QueryPattern            *DetailedQueryPattern            `json:"query_pattern"`
	ResourceUsage           *ResourceUsageAnalysis   `json:"resource_usage"`
	ComparisonMetrics       *QueryComparisonMetrics  `json:"comparison_metrics"`
	RiskAssessment          *QueryRiskAssessment     `json:"risk_assessment"`
}

type PerformanceImpact struct {
	ImpactScore        float64 `json:"impact_score"` // 0-100
	ExpectedTime       time.Duration `json:"expected_time"`
	ActualTime         time.Duration `json:"actual_time"`
	PerformanceDelta   time.Duration `json:"performance_delta"`
	SlownessMultiplier float64       `json:"slowness_multiplier"`
	FrequencyRank      int           `json:"frequency_rank"`
	CriticalityLevel   string        `json:"criticality_level"`
}

type OptimizationSuggestion struct {
	Priority            string `json:"priority"` // "high", "medium", "low"
	Type               string `json:"type"`     // "index", "query_rewrite", "schema", "config"
	Description        string `json:"description"`
	Implementation     string `json:"implementation"`
	EstimatedImprovement string `json:"estimated_improvement"`
	ComplexityRating   string `json:"complexity_rating"`
	Prerequisites      []string `json:"prerequisites,omitempty"`
}

type DetailedIndexRecommendation struct {
	TableName         string   `json:"table_name"`
	RecommendedIndex  string   `json:"recommended_index"`
	Columns           []string `json:"columns"`
	IndexType         string   `json:"index_type"` // "btree", "gin", "gist", "hash"
	Rationale         string   `json:"rationale"`
	EstimatedSize     string   `json:"estimated_size"`
	MaintenanceCost   string   `json:"maintenance_cost"`
	ConflictingIndexes []string `json:"conflicting_indexes,omitempty"`
}

type DetailedQueryPattern struct {
	PatternType       string                 `json:"pattern_type"` // "search", "aggregation", "join", "update"
	Complexity        string                 `json:"complexity"`   // "simple", "moderate", "complex"
	TablesInvolved    []string              `json:"tables_involved"`
	JoinType          string                `json:"join_type"`
	FilterPatterns    []string              `json:"filter_patterns"`
	SortingPatterns   []string              `json:"sorting_patterns"`
	GroupingPatterns  []string              `json:"grouping_patterns"`
	SubqueryCount     int                   `json:"subquery_count"`
	AggregationCount  int                   `json:"aggregation_count"`
	UnionUsage        bool                  `json:"union_usage"`
	WindowFunctions   bool                  `json:"window_functions"`
	RecursiveQuery    bool                  `json:"recursive_query"`
	CustomAttributes  map[string]interface{} `json:"custom_attributes,omitempty"`
}

type ResourceUsageAnalysis struct {
	CPUUsage          float64 `json:"cpu_usage_percent"`
	MemoryUsage       int64   `json:"memory_usage_bytes"`
	IOOperations      int64   `json:"io_operations"`
	NetworkBandwidth  int64   `json:"network_bandwidth_bytes"`
	DiskReads         int64   `json:"disk_reads"`
	DiskWrites        int64   `json:"disk_writes"`
	BufferHitRatio    float64 `json:"buffer_hit_ratio"`
	LockContention    bool    `json:"lock_contention"`
	DeadlockRisk      string  `json:"deadlock_risk"`
	ResourceEfficiency string `json:"resource_efficiency"`
}

type QueryComparisonMetrics struct {
	SimilarQueriesFound   int           `json:"similar_queries_found"`
	AverageExecutionTime  time.Duration `json:"average_execution_time"`
	MedianExecutionTime   time.Duration `json:"median_execution_time"`
	P95ExecutionTime      time.Duration `json:"p95_execution_time"`
	BestExecutionTime     time.Duration `json:"best_execution_time"`
	WorstExecutionTime    time.Duration `json:"worst_execution_time"`
	TrendDirection        string        `json:"trend_direction"` // "improving", "degrading", "stable"
	FrequencyLastHour     int           `json:"frequency_last_hour"`
	FrequencyLastDay      int           `json:"frequency_last_day"`
	FrequencyLastWeek     int           `json:"frequency_last_week"`
}

type QueryRiskAssessment struct {
	RiskLevel            string   `json:"risk_level"` // "low", "medium", "high", "critical"
	RiskFactors          []string `json:"risk_factors"`
	BusinessImpact       string   `json:"business_impact"`
	UserExperienceImpact string   `json:"user_experience_impact"`
	RecommendedActions   []string `json:"recommended_actions"`
	MonitoringPriority   string   `json:"monitoring_priority"`
	EscalationRequired   bool     `json:"escalation_required"`
}

type SlowQueryStatistics struct {
	TotalSlowQueries        int           `json:"total_slow_queries"`
	UniqueSlowQueries       int           `json:"unique_slow_queries"`
	AverageExecutionTime    time.Duration `json:"average_execution_time"`
	MedianExecutionTime     time.Duration `json:"median_execution_time"`
	P95ExecutionTime        time.Duration `json:"p95_execution_time"`
	P99ExecutionTime        time.Duration `json:"p99_execution_time"`
	SlowestQuery            *DetectedSlowQuery `json:"slowest_query,omitempty"`
	MostFrequentSlowQuery   *DetectedSlowQuery `json:"most_frequent_slow_query,omitempty"`
	QueryCategoryCounts     map[string]int     `json:"query_category_counts"`
	HourlyDistribution      map[int]int        `json:"hourly_distribution"`
	DatabaseDistribution    map[string]int     `json:"database_distribution"`
	TableAccessPatterns     map[string]int     `json:"table_access_patterns"`
	PerformanceTrend        string             `json:"performance_trend"`
	ThresholdBreach         bool               `json:"threshold_breach"`
}

type PgStatQuery struct {
	QueryID          string        `json:"query_id"`
	Query            string        `json:"query"`
	Calls            int64         `json:"calls"`
	TotalExecTime    time.Duration `json:"total_exec_time"`
	MeanExecTime     time.Duration `json:"mean_exec_time"`
	MinExecTime      time.Duration `json:"min_exec_time"`
	MaxExecTime      time.Duration `json:"max_exec_time"`
	StddevExecTime   time.Duration `json:"stddev_exec_time"`
	Rows             int64         `json:"rows"`
	SharedBlksHit    int64         `json:"shared_blks_hit"`
	SharedBlksRead   int64         `json:"shared_blks_read"`
	SharedBlksDirtied int64        `json:"shared_blks_dirtied"`
	SharedBlksWritten int64        `json:"shared_blks_written"`
	LocalBlksHit     int64         `json:"local_blks_hit"`
	LocalBlksRead    int64         `json:"local_blks_read"`
	LocalBlksWritten int64         `json:"local_blks_written"`
	TempBlksRead     int64         `json:"temp_blks_read"`
	TempBlksWritten  int64         `json:"temp_blks_written"`
	BlkReadTime      time.Duration `json:"blk_read_time"`
	BlkWriteTime     time.Duration `json:"blk_write_time"`
	LastCall         time.Time     `json:"last_call"`
}

type SlowQueryCallback func(ctx context.Context, query *DetectedSlowQuery) error

type databaseSlowQueryLogger struct {
	db                    *gorm.DB
	queryExecutionMonitor QueryExecutionMonitor
	
	// Configuration
	slowThreshold      time.Duration
	detailedLogging    bool
	isRunning          bool
	
	// Real-time monitoring
	callback           SlowQueryCallback
	
	// Internal state
	mutex              sync.RWMutex
	stopChan          chan struct{}
	wg                sync.WaitGroup
	
	// Statistics tracking
	statsCache        map[string]*SlowQueryStatistics
	lastStatsUpdate   time.Time
	statsCacheExpiry  time.Duration
}

func NewDatabaseSlowQueryLogger(db *gorm.DB, queryMonitor QueryExecutionMonitor) DatabaseSlowQueryLogger {
	return &databaseSlowQueryLogger{
		db:                    db,
		queryExecutionMonitor: queryMonitor,
		slowThreshold:         200 * time.Millisecond, // Default from AC
		detailedLogging:       true,
		statsCache:           make(map[string]*SlowQueryStatistics),
		statsCacheExpiry:     5 * time.Minute,
		isRunning:            false,
	}
}

func (l *databaseSlowQueryLogger) Start(ctx context.Context) error {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	
	if l.isRunning {
		return fmt.Errorf("slow query logger is already running")
	}
	
	// Enable PostgreSQL extensions for monitoring
	if err := l.enablePostgreSQLLogging(ctx); err != nil {
		return fmt.Errorf("failed to enable PostgreSQL logging: %w", err)
	}
	
	// Enable pg_stat_statements
	if err := l.EnablePgStatStatements(ctx); err != nil {
		log.Printf("Warning: failed to enable pg_stat_statements: %v", err)
		// Not critical - continue without it
	}
	
	l.stopChan = make(chan struct{})
	l.isRunning = true
	
	// Start background monitoring
	l.wg.Add(1)
	go l.backgroundMonitor(ctx)
	
	log.Printf("Database slow query logger started with threshold: %v", l.slowThreshold)
	return nil
}

func (l *databaseSlowQueryLogger) Stop(ctx context.Context) error {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	
	if !l.isRunning {
		return nil
	}
	
	close(l.stopChan)
	l.isRunning = false
	
	// Wait for background processes to complete
	l.wg.Wait()
	
	// Disable logging
	if err := l.disablePostgreSQLLogging(ctx); err != nil {
		log.Printf("Warning: failed to disable PostgreSQL logging: %v", err)
	}
	
	log.Printf("Database slow query logger stopped")
	return nil
}

func (l *databaseSlowQueryLogger) IsRunning() bool {
	l.mutex.RLock()
	defer l.mutex.RUnlock()
	return l.isRunning
}

func (l *databaseSlowQueryLogger) SetSlowQueryThreshold(threshold time.Duration) error {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	
	if threshold <= 0 {
		return fmt.Errorf("threshold must be positive")
	}
	
	l.slowThreshold = threshold
	
	// Update PostgreSQL configuration if running
	if l.isRunning {
		query := fmt.Sprintf("SET log_min_duration_statement = '%d'", int(threshold.Milliseconds()))
		if err := l.db.Exec(query).Error; err != nil {
			return fmt.Errorf("failed to update PostgreSQL threshold: %w", err)
		}
	}
	
	log.Printf("Slow query threshold updated to: %v", threshold)
	return nil
}

func (l *databaseSlowQueryLogger) GetSlowQueryThreshold() time.Duration {
	l.mutex.RLock()
	defer l.mutex.RUnlock()
	return l.slowThreshold
}

func (l *databaseSlowQueryLogger) EnableDetailedLogging(enabled bool) error {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	
	l.detailedLogging = enabled
	
	if l.isRunning {
		// Update PostgreSQL configuration
		logLevel := "off"
		if enabled {
			logLevel = "all"
		}
		
		query := fmt.Sprintf("SET log_statement = '%s'", logLevel)
		if err := l.db.Exec(query).Error; err != nil {
			return fmt.Errorf("failed to update PostgreSQL detailed logging: %w", err)
		}
	}
	
	log.Printf("Detailed logging %s", map[bool]string{true: "enabled", false: "disabled"}[enabled])
	return nil
}

func (l *databaseSlowQueryLogger) GetSlowQueries(ctx context.Context, since time.Time, limit int) ([]DetectedSlowQuery, error) {
	// Query from pg_stat_statements and combine with our analysis
	pgQueries, err := l.GetTopSlowQueries(ctx, limit*2) // Get more to filter
	if err != nil {
		return nil, fmt.Errorf("failed to get queries from pg_stat_statements: %w", err)
	}
	
	var slowQueries []DetectedSlowQuery
	
	for _, pgQuery := range pgQueries {
		if pgQuery.MeanExecTime < l.slowThreshold {
			continue
		}
		
		// Convert to our format
		slowQuery := DetectedSlowQuery{
			QueryID:         pgQuery.QueryID,
			QueryText:       pgQuery.Query,
			QueryNormalized: l.normalizeQuery(pgQuery.Query),
			ExecutionTime:   pgQuery.MeanExecTime,
			DetectedAt:      pgQuery.LastCall,
			RowsExamined:    pgQuery.Rows,
			Database:        "imkitchen", // Default database
			Severity:        l.calculateSeverity(pgQuery.MeanExecTime),
			CategoryTags:    l.categorizeQuery(pgQuery.Query),
		}
		
		// Add execution context if available
		slowQuery.ExecutionContext = l.buildExecutionContext(pgQuery)
		
		slowQueries = append(slowQueries, slowQuery)
		
		if len(slowQueries) >= limit {
			break
		}
	}
	
	return slowQueries, nil
}

func (l *databaseSlowQueryLogger) AnalyzeSlowQuery(ctx context.Context, query *DetectedSlowQuery) (*SlowQueryAnalysis, error) {
	analysis := &SlowQueryAnalysis{
		Query: query,
	}
	
	// Get execution plan for analysis
	executionPlan, err := l.queryExecutionMonitor.GetQueryExecutionPlan(ctx, query.QueryText)
	if err != nil {
		log.Printf("Warning: failed to get execution plan for slow query analysis: %v", err)
		// Continue with limited analysis
	}
	
	// Analyze performance impact
	analysis.PerformanceImpact = l.analyzePerformanceImpact(query, executionPlan)
	
	// Generate optimization suggestions
	analysis.OptimizationSuggestions = l.generateOptimizationSuggestions(query, executionPlan)
	
	// Generate index recommendations
	analysis.IndexRecommendations = l.generateIndexRecommendations(query, executionPlan)
	
	// Analyze query pattern
	analysis.QueryPattern = l.analyzeQueryPattern(query)
	
	// Analyze resource usage
	analysis.ResourceUsage = l.analyzeResourceUsage(query)
	
	// Get comparison metrics
	analysis.ComparisonMetrics = l.getComparisonMetrics(ctx, query)
	
	// Assess risk
	analysis.RiskAssessment = l.assessQueryRisk(query, analysis)
	
	return analysis, nil
}

func (l *databaseSlowQueryLogger) GetSlowQueryStatistics(ctx context.Context, since time.Time) (*SlowQueryStatistics, error) {
	// Check cache first
	cacheKey := fmt.Sprintf("stats_%d", since.Unix())
	
	l.mutex.RLock()
	if cachedStats, exists := l.statsCache[cacheKey]; exists && 
		time.Since(l.lastStatsUpdate) < l.statsCacheExpiry {
		l.mutex.RUnlock()
		return cachedStats, nil
	}
	l.mutex.RUnlock()
	
	// Gather fresh statistics
	slowQueries, err := l.GetSlowQueries(ctx, since, 1000) // Get more for stats
	if err != nil {
		return nil, fmt.Errorf("failed to get slow queries for statistics: %w", err)
	}
	
	stats := l.computeStatistics(slowQueries)
	
	// Cache the results
	l.mutex.Lock()
	l.statsCache[cacheKey] = stats
	l.lastStatsUpdate = time.Now()
	l.mutex.Unlock()
	
	return stats, nil
}

func (l *databaseSlowQueryLogger) RegisterSlowQueryCallback(callback SlowQueryCallback) {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	l.callback = callback
}

func (l *databaseSlowQueryLogger) UnregisterSlowQueryCallback() {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	l.callback = nil
}

func (l *databaseSlowQueryLogger) EnablePgStatStatements(ctx context.Context) error {
	// Check if extension is available
	var extensionExists bool
	err := l.db.WithContext(ctx).Raw(
		"SELECT EXISTS(SELECT 1 FROM pg_available_extensions WHERE name = 'pg_stat_statements')",
	).Scan(&extensionExists).Error
	
	if err != nil {
		return fmt.Errorf("failed to check pg_stat_statements availability: %w", err)
	}
	
	if !extensionExists {
		return fmt.Errorf("pg_stat_statements extension is not available")
	}
	
	// Create extension if not exists
	err = l.db.WithContext(ctx).Exec("CREATE EXTENSION IF NOT EXISTS pg_stat_statements").Error
	if err != nil {
		return fmt.Errorf("failed to create pg_stat_statements extension: %w", err)
	}
	
	// Configure pg_stat_statements
	queries := []string{
		"SELECT pg_stat_statements_reset()", // Reset statistics
	}
	
	for _, query := range queries {
		if err := l.db.WithContext(ctx).Exec(query).Error; err != nil {
			log.Printf("Warning: failed to execute pg_stat_statements configuration: %v", err)
		}
	}
	
	log.Printf("pg_stat_statements extension enabled")
	return nil
}

func (l *databaseSlowQueryLogger) GetTopSlowQueries(ctx context.Context, limit int) ([]PgStatQuery, error) {
	query := `
		SELECT 
			queryid::text as query_id,
			query,
			calls,
			total_exec_time as total_exec_time_ms,
			mean_exec_time as mean_exec_time_ms,
			min_exec_time as min_exec_time_ms,
			max_exec_time as max_exec_time_ms,
			stddev_exec_time as stddev_exec_time_ms,
			rows,
			shared_blks_hit,
			shared_blks_read,
			shared_blks_dirtied,
			shared_blks_written,
			local_blks_hit,
			local_blks_read,
			local_blks_written,
			temp_blks_read,
			temp_blks_written,
			blk_read_time as blk_read_time_ms,
			blk_write_time as blk_write_time_ms,
			NOW() as last_call -- Approximation
		FROM pg_stat_statements 
		WHERE query NOT LIKE '%pg_stat_statements%'
		  AND query NOT LIKE '%EXPLAIN%'
		  AND mean_exec_time >= ?
		ORDER BY mean_exec_time DESC
		LIMIT ?
	`
	
	type pgStatResult struct {
		QueryID           string  `json:"query_id"`
		Query             string  `json:"query"`
		Calls             int64   `json:"calls"`
		TotalExecTimeMs   float64 `json:"total_exec_time_ms"`
		MeanExecTimeMs    float64 `json:"mean_exec_time_ms"`
		MinExecTimeMs     float64 `json:"min_exec_time_ms"`
		MaxExecTimeMs     float64 `json:"max_exec_time_ms"`
		StddevExecTimeMs  float64 `json:"stddev_exec_time_ms"`
		Rows              int64   `json:"rows"`
		SharedBlksHit     int64   `json:"shared_blks_hit"`
		SharedBlksRead    int64   `json:"shared_blks_read"`
		SharedBlksDirtied int64   `json:"shared_blks_dirtied"`
		SharedBlksWritten int64   `json:"shared_blks_written"`
		LocalBlksHit      int64   `json:"local_blks_hit"`
		LocalBlksRead     int64   `json:"local_blks_read"`
		LocalBlksWritten  int64   `json:"local_blks_written"`
		TempBlksRead      int64   `json:"temp_blks_read"`
		TempBlksWritten   int64   `json:"temp_blks_written"`
		BlkReadTimeMs     float64 `json:"blk_read_time_ms"`
		BlkWriteTimeMs    float64 `json:"blk_write_time_ms"`
		LastCall          time.Time `json:"last_call"`
	}
	
	var results []pgStatResult
	err := l.db.WithContext(ctx).Raw(query, l.slowThreshold.Milliseconds(), limit).Scan(&results).Error
	if err != nil {
		return nil, fmt.Errorf("failed to query pg_stat_statements: %w", err)
	}
	
	var pgQueries []PgStatQuery
	for _, result := range results {
		pgQueries = append(pgQueries, PgStatQuery{
			QueryID:          result.QueryID,
			Query:            result.Query,
			Calls:            result.Calls,
			TotalExecTime:    time.Duration(result.TotalExecTimeMs) * time.Millisecond,
			MeanExecTime:     time.Duration(result.MeanExecTimeMs) * time.Millisecond,
			MinExecTime:      time.Duration(result.MinExecTimeMs) * time.Millisecond,
			MaxExecTime:      time.Duration(result.MaxExecTimeMs) * time.Millisecond,
			StddevExecTime:   time.Duration(result.StddevExecTimeMs) * time.Millisecond,
			Rows:             result.Rows,
			SharedBlksHit:    result.SharedBlksHit,
			SharedBlksRead:   result.SharedBlksRead,
			SharedBlksDirtied: result.SharedBlksDirtied,
			SharedBlksWritten: result.SharedBlksWritten,
			LocalBlksHit:     result.LocalBlksHit,
			LocalBlksRead:    result.LocalBlksRead,
			LocalBlksWritten: result.LocalBlksWritten,
			TempBlksRead:     result.TempBlksRead,
			TempBlksWritten:  result.TempBlksWritten,
			BlkReadTime:      time.Duration(result.BlkReadTimeMs) * time.Millisecond,
			BlkWriteTime:     time.Duration(result.BlkWriteTimeMs) * time.Millisecond,
			LastCall:         result.LastCall,
		})
	}
	
	return pgQueries, nil
}

func (l *databaseSlowQueryLogger) ResetQueryStatistics(ctx context.Context) error {
	err := l.db.WithContext(ctx).Exec("SELECT pg_stat_statements_reset()").Error
	if err != nil {
		return fmt.Errorf("failed to reset pg_stat_statements: %w", err)
	}
	
	// Clear internal cache
	l.mutex.Lock()
	l.statsCache = make(map[string]*SlowQueryStatistics)
	l.mutex.Unlock()
	
	log.Printf("Query statistics reset")
	return nil
}

// Internal helper methods

func (l *databaseSlowQueryLogger) enablePostgreSQLLogging(ctx context.Context) error {
	queries := []string{
		fmt.Sprintf("SET log_min_duration_statement = '%d'", int(l.slowThreshold.Milliseconds())),
		"SET log_checkpoints = on",
		"SET log_connections = on",
		"SET log_disconnections = on",
		"SET log_lock_waits = on",
		"SET log_statement = 'all'",
		"SET log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '",
	}
	
	for _, query := range queries {
		if err := l.db.WithContext(ctx).Exec(query).Error; err != nil {
			return fmt.Errorf("failed to execute logging configuration: %w", err)
		}
	}
	
	return nil
}

func (l *databaseSlowQueryLogger) disablePostgreSQLLogging(ctx context.Context) error {
	queries := []string{
		"SET log_min_duration_statement = -1",
		"SET log_statement = 'none'",
	}
	
	for _, query := range queries {
		if err := l.db.WithContext(ctx).Exec(query).Error; err != nil {
			return fmt.Errorf("failed to disable logging configuration: %w", err)
		}
	}
	
	return nil
}

func (l *databaseSlowQueryLogger) backgroundMonitor(ctx context.Context) {
	defer l.wg.Done()
	
	ticker := time.NewTicker(30 * time.Second) // Check every 30 seconds
	defer ticker.Stop()
	
	for {
		select {
		case <-l.stopChan:
			return
		case <-ticker.C:
			l.performBackgroundCheck(ctx)
		}
	}
}

func (l *databaseSlowQueryLogger) performBackgroundCheck(ctx context.Context) {
	if l.callback == nil {
		return
	}
	
	// Get recent slow queries
	since := time.Now().Add(-5 * time.Minute)
	slowQueries, err := l.GetSlowQueries(ctx, since, 10)
	if err != nil {
		log.Printf("Error getting slow queries in background check: %v", err)
		return
	}
	
	// Process each slow query through callback
	for _, slowQuery := range slowQueries {
		if err := l.callback(ctx, &slowQuery); err != nil {
			log.Printf("Error in slow query callback: %v", err)
		}
	}
}

// Analysis helper methods

func (l *databaseSlowQueryLogger) normalizeQuery(query string) string {
	// Simple normalization - replace parameters with placeholders
	normalized := strings.ReplaceAll(query, "'", "?")
	normalized = strings.ReplaceAll(normalized, "$1", "?")
	normalized = strings.ReplaceAll(normalized, "$2", "?")
	normalized = strings.ReplaceAll(normalized, "$3", "?")
	return normalized
}

func (l *databaseSlowQueryLogger) calculateSeverity(executionTime time.Duration) string {
	switch {
	case executionTime > 5*time.Second:
		return "critical"
	case executionTime > 2*time.Second:
		return "severe"
	case executionTime > 1*time.Second:
		return "warning"
	default:
		return "warning"
	}
}

func (l *databaseSlowQueryLogger) categorizeQuery(query string) []string {
	tags := []string{}
	queryLower := strings.ToLower(query)
	
	if strings.Contains(queryLower, "select") {
		tags = append(tags, "select")
	}
	if strings.Contains(queryLower, "insert") {
		tags = append(tags, "insert")
	}
	if strings.Contains(queryLower, "update") {
		tags = append(tags, "update")
	}
	if strings.Contains(queryLower, "delete") {
		tags = append(tags, "delete")
	}
	if strings.Contains(queryLower, "recipe") {
		tags = append(tags, "recipe_related")
	}
	if strings.Contains(queryLower, "join") {
		tags = append(tags, "join")
	}
	if strings.Contains(queryLower, "order by") {
		tags = append(tags, "sorting")
	}
	if strings.Contains(queryLower, "group by") {
		tags = append(tags, "aggregation")
	}
	
	return tags
}

func (l *databaseSlowQueryLogger) buildExecutionContext(pgQuery PgStatQuery) *QueryExecutionContext {
	return &QueryExecutionContext{
		QueryStartTime:       pgQuery.LastCall,
		SharedBlksHit:        pgQuery.SharedBlksHit,
		SharedBlksRead:       pgQuery.SharedBlksRead,
		LocalBlksRead:        pgQuery.LocalBlksRead,
		LocalBlksWritten:     pgQuery.LocalBlksWritten,
		TempBytesRead:        pgQuery.TempBlksRead * 8192,  // Assume 8KB blocks
		TempBytesWritten:     pgQuery.TempBlksWritten * 8192,
		IOWaitTime:          pgQuery.BlkReadTime + pgQuery.BlkWriteTime,
		AdditionalMetrics: map[string]interface{}{
			"calls": pgQuery.Calls,
			"total_exec_time": pgQuery.TotalExecTime.String(),
			"rows": pgQuery.Rows,
		},
	}
}

func (l *databaseSlowQueryLogger) analyzePerformanceImpact(query *DetectedSlowQuery, plan *ExecutionPlan) *PerformanceImpact {
	expectedTime := 100 * time.Millisecond // Expected baseline
	multiplier := float64(query.ExecutionTime) / float64(expectedTime)
	
	var criticalityLevel string
	var impactScore float64
	
	switch {
	case multiplier > 50:
		criticalityLevel = "critical"
		impactScore = 100
	case multiplier > 20:
		criticalityLevel = "high"
		impactScore = 85
	case multiplier > 10:
		criticalityLevel = "medium"
		impactScore = 65
	case multiplier > 5:
		criticalityLevel = "low"
		impactScore = 35
	default:
		criticalityLevel = "minimal"
		impactScore = 15
	}
	
	return &PerformanceImpact{
		ImpactScore:        impactScore,
		ExpectedTime:       expectedTime,
		ActualTime:         query.ExecutionTime,
		PerformanceDelta:   query.ExecutionTime - expectedTime,
		SlownessMultiplier: multiplier,
		CriticalityLevel:   criticalityLevel,
	}
}

func (l *databaseSlowQueryLogger) generateOptimizationSuggestions(query *DetectedSlowQuery, plan *ExecutionPlan) []OptimizationSuggestion {
	suggestions := []OptimizationSuggestion{}
	
	queryLower := strings.ToLower(query.QueryText)
	
	// Check for missing indices
	if strings.Contains(queryLower, "where") && !strings.Contains(queryLower, "index") {
		suggestions = append(suggestions, OptimizationSuggestion{
			Priority:     "high",
			Type:         "index",
			Description:  "Add database index on frequently queried columns",
			Implementation: "CREATE INDEX idx_<table>_<column> ON <table> (<column>)",
			EstimatedImprovement: "70-90% performance improvement",
			ComplexityRating: "low",
		})
	}
	
	// Check for N+1 queries
	if strings.Contains(queryLower, "select") && len(l.categorizeQuery(query.QueryText)) == 1 {
		suggestions = append(suggestions, OptimizationSuggestion{
			Priority:     "medium",
			Type:         "query_rewrite",
			Description:  "Consider using JOIN instead of multiple SELECT queries",
			Implementation: "Rewrite as single query with appropriate JOINs",
			EstimatedImprovement: "50-80% performance improvement",
			ComplexityRating: "medium",
		})
	}
	
	// Check for inefficient ORDER BY
	if strings.Contains(queryLower, "order by") && query.ExecutionTime > 500*time.Millisecond {
		suggestions = append(suggestions, OptimizationSuggestion{
			Priority:     "medium",
			Type:         "index",
			Description:  "Add index on ORDER BY columns to avoid sorting",
			Implementation: "CREATE INDEX ON <table> (<order_by_columns>)",
			EstimatedImprovement: "60-85% performance improvement",
			ComplexityRating: "low",
		})
	}
	
	return suggestions
}

func (l *databaseSlowQueryLogger) generateIndexRecommendations(query *DetectedSlowQuery, plan *ExecutionPlan) []DetailedIndexRecommendation {
	recommendations := []DetailedIndexRecommendation{}
	queryLower := strings.ToLower(query.QueryText)
	
	// Simple pattern matching for common index needs
	if strings.Contains(queryLower, "recipes") && strings.Contains(queryLower, "cuisine") {
		recommendations = append(recommendations, DetailedIndexRecommendation{
			TableName:        "recipes",
			RecommendedIndex: "idx_recipes_cuisine",
			Columns:          []string{"cuisine"},
			IndexType:        "btree",
			Rationale:        "Frequently queried in recipe search operations",
			EstimatedSize:    "~2MB",
			MaintenanceCost:  "Low",
		})
	}
	
	if strings.Contains(queryLower, "recipes") && strings.Contains(queryLower, "preparation_time") {
		recommendations = append(recommendations, DetailedIndexRecommendation{
			TableName:        "recipes",
			RecommendedIndex: "idx_recipes_prep_time",
			Columns:          []string{"preparation_time"},
			IndexType:        "btree",
			Rationale:        "Support for preparation time filtering",
			EstimatedSize:    "~1MB",
			MaintenanceCost:  "Low",
		})
	}
	
	return recommendations
}

func (l *databaseSlowQueryLogger) analyzeQueryPattern(query *DetectedSlowQuery) *DetailedQueryPattern {
	queryLower := strings.ToLower(query.QueryText)
	
	pattern := &DetailedQueryPattern{
		TablesInvolved:   l.extractTables(queryLower),
		FilterPatterns:   l.extractFilters(queryLower),
		SortingPatterns:  l.extractSorting(queryLower),
		GroupingPatterns: l.extractGrouping(queryLower),
	}
	
	// Determine pattern type
	if strings.Contains(queryLower, "select") {
		if strings.Contains(queryLower, "count") || strings.Contains(queryLower, "sum") || strings.Contains(queryLower, "avg") {
			pattern.PatternType = "aggregation"
		} else if strings.Contains(queryLower, "join") {
			pattern.PatternType = "join"
		} else {
			pattern.PatternType = "search"
		}
	} else if strings.Contains(queryLower, "update") {
		pattern.PatternType = "update"
	}
	
	// Determine complexity
	complexityScore := 0
	if len(pattern.TablesInvolved) > 2 {
		complexityScore += 2
	}
	if strings.Contains(queryLower, "subquery") || strings.Contains(queryLower, "(select") {
		complexityScore += 3
		pattern.SubqueryCount = strings.Count(queryLower, "(select")
	}
	if strings.Contains(queryLower, "union") {
		complexityScore += 2
		pattern.UnionUsage = true
	}
	if strings.Contains(queryLower, "over(") {
		complexityScore += 2
		pattern.WindowFunctions = true
	}
	
	switch {
	case complexityScore > 5:
		pattern.Complexity = "complex"
	case complexityScore > 2:
		pattern.Complexity = "moderate"
	default:
		pattern.Complexity = "simple"
	}
	
	return pattern
}

func (l *databaseSlowQueryLogger) analyzeResourceUsage(query *DetectedSlowQuery) *ResourceUsageAnalysis {
	if query.ExecutionContext == nil {
		return &ResourceUsageAnalysis{
			ResourceEfficiency: "unknown",
		}
	}
	
	ctx := query.ExecutionContext
	
	// Calculate buffer hit ratio
	var hitRatio float64
	if ctx.SharedBlksHit+ctx.SharedBlksRead > 0 {
		hitRatio = float64(ctx.SharedBlksHit) / float64(ctx.SharedBlksHit+ctx.SharedBlksRead)
	}
	
	efficiency := "good"
	if hitRatio < 0.95 {
		efficiency = "poor"
	} else if hitRatio < 0.98 {
		efficiency = "fair"
	}
	
	return &ResourceUsageAnalysis{
		MemoryUsage:       ctx.TempBytesRead + ctx.TempBytesWritten,
		IOOperations:      ctx.SharedBlksRead + ctx.SharedBlksHit + ctx.LocalBlksRead + ctx.LocalBlksWritten,
		DiskReads:         ctx.SharedBlksRead,
		DiskWrites:        ctx.LocalBlksWritten,
		BufferHitRatio:    hitRatio,
		LockContention:    ctx.LockWaitTime > 100*time.Millisecond,
		DeadlockRisk:      "low", // Default assessment
		ResourceEfficiency: efficiency,
	}
}

func (l *databaseSlowQueryLogger) getComparisonMetrics(ctx context.Context, query *DetectedSlowQuery) *QueryComparisonMetrics {
	// This would ideally query historical data
	// For now, return basic comparison
	return &QueryComparisonMetrics{
		SimilarQueriesFound:  1,
		AverageExecutionTime: query.ExecutionTime,
		MedianExecutionTime:  query.ExecutionTime,
		P95ExecutionTime:     query.ExecutionTime,
		BestExecutionTime:    query.ExecutionTime,
		WorstExecutionTime:   query.ExecutionTime,
		TrendDirection:       "stable",
		FrequencyLastHour:    1,
		FrequencyLastDay:     1,
		FrequencyLastWeek:    1,
	}
}

func (l *databaseSlowQueryLogger) assessQueryRisk(query *DetectedSlowQuery, analysis *SlowQueryAnalysis) *QueryRiskAssessment {
	riskFactors := []string{}
	riskLevel := "low"
	
	if query.ExecutionTime > 2*time.Second {
		riskFactors = append(riskFactors, "High execution time")
		riskLevel = "high"
	}
	
	if analysis.PerformanceImpact.SlownessMultiplier > 20 {
		riskFactors = append(riskFactors, "Extreme performance degradation")
		riskLevel = "critical"
	}
	
	if len(analysis.OptimizationSuggestions) > 3 {
		riskFactors = append(riskFactors, "Multiple optimization opportunities")
		if riskLevel == "low" {
			riskLevel = "medium"
		}
	}
	
	businessImpact := "minimal"
	userImpact := "minimal"
	
	if riskLevel == "high" || riskLevel == "critical" {
		businessImpact = "moderate to high"
		userImpact = "noticeable delays"
	}
	
	return &QueryRiskAssessment{
		RiskLevel:            riskLevel,
		RiskFactors:          riskFactors,
		BusinessImpact:       businessImpact,
		UserExperienceImpact: userImpact,
		RecommendedActions: []string{
			"Monitor query performance",
			"Consider implementing optimization suggestions",
		},
		MonitoringPriority: riskLevel,
		EscalationRequired: riskLevel == "critical",
	}
}

func (l *databaseSlowQueryLogger) computeStatistics(queries []DetectedSlowQuery) *SlowQueryStatistics {
	if len(queries) == 0 {
		return &SlowQueryStatistics{
			QueryCategoryCounts:  make(map[string]int),
			HourlyDistribution:   make(map[int]int),
			DatabaseDistribution: make(map[string]int),
			TableAccessPatterns:  make(map[string]int),
		}
	}
	
	stats := &SlowQueryStatistics{
		TotalSlowQueries:     len(queries),
		QueryCategoryCounts:  make(map[string]int),
		HourlyDistribution:   make(map[int]int),
		DatabaseDistribution: make(map[string]int),
		TableAccessPatterns:  make(map[string]int),
	}
	
	var totalTime time.Duration
	executionTimes := make([]time.Duration, len(queries))
	uniqueQueries := make(map[string]bool)
	
	var slowest *DetectedSlowQuery
	var mostFrequent *DetectedSlowQuery
	frequencyCount := make(map[string]int)
	
	for i, query := range queries {
		totalTime += query.ExecutionTime
		executionTimes[i] = query.ExecutionTime
		uniqueQueries[query.QueryNormalized] = true
		
		// Track slowest
		if slowest == nil || query.ExecutionTime > slowest.ExecutionTime {
			slowest = &query
		}
		
		// Track frequency
		frequencyCount[query.QueryNormalized]++
		if mostFrequent == nil || frequencyCount[query.QueryNormalized] > frequencyCount[mostFrequent.QueryNormalized] {
			mostFrequent = &query
		}
		
		// Category counts
		for _, tag := range query.CategoryTags {
			stats.QueryCategoryCounts[tag]++
		}
		
		// Hourly distribution
		hour := query.DetectedAt.Hour()
		stats.HourlyDistribution[hour]++
		
		// Database distribution
		stats.DatabaseDistribution[query.Database]++
		
		// Table access patterns (simplified)
		queryLower := strings.ToLower(query.QueryText)
		if strings.Contains(queryLower, "recipes") {
			stats.TableAccessPatterns["recipes"]++
		}
		if strings.Contains(queryLower, "users") {
			stats.TableAccessPatterns["users"]++
		}
	}
	
	stats.UniqueSlowQueries = len(uniqueQueries)
	stats.AverageExecutionTime = totalTime / time.Duration(len(queries))
	stats.SlowestQuery = slowest
	stats.MostFrequentSlowQuery = mostFrequent
	
	// Calculate percentiles (simplified)
	if len(executionTimes) > 0 {
		// Sort execution times
		for i := 0; i < len(executionTimes); i++ {
			for j := i + 1; j < len(executionTimes); j++ {
				if executionTimes[i] > executionTimes[j] {
					executionTimes[i], executionTimes[j] = executionTimes[j], executionTimes[i]
				}
			}
		}
		
		stats.MedianExecutionTime = executionTimes[len(executionTimes)/2]
		stats.P95ExecutionTime = executionTimes[int(float64(len(executionTimes))*0.95)]
		stats.P99ExecutionTime = executionTimes[int(float64(len(executionTimes))*0.99)]
	}
	
	// Simple trend analysis
	stats.PerformanceTrend = "stable" // Would require historical data for actual trending
	stats.ThresholdBreach = len(queries) > 100 // Arbitrary threshold
	
	return stats
}

// Helper methods for query pattern analysis

func (l *databaseSlowQueryLogger) extractTables(query string) []string {
	tables := []string{}
	
	// Simple table extraction - look for FROM and JOIN clauses
	words := strings.Fields(query)
	for i, word := range words {
		if (strings.ToLower(word) == "from" || strings.ToLower(word) == "join") && i+1 < len(words) {
			tables = append(tables, words[i+1])
		}
	}
	
	return tables
}

func (l *databaseSlowQueryLogger) extractFilters(query string) []string {
	filters := []string{}
	
	if strings.Contains(query, "where") {
		filters = append(filters, "where_clause")
	}
	if strings.Contains(query, "=") {
		filters = append(filters, "equality")
	}
	if strings.Contains(query, "like") {
		filters = append(filters, "pattern_matching")
	}
	if strings.Contains(query, "in(") {
		filters = append(filters, "in_clause")
	}
	
	return filters
}

func (l *databaseSlowQueryLogger) extractSorting(query string) []string {
	sorting := []string{}
	
	if strings.Contains(query, "order by") {
		sorting = append(sorting, "order_by")
	}
	if strings.Contains(query, "desc") {
		sorting = append(sorting, "descending")
	} else if strings.Contains(query, "asc") {
		sorting = append(sorting, "ascending")
	}
	
	return sorting
}

func (l *databaseSlowQueryLogger) extractGrouping(query string) []string {
	grouping := []string{}
	
	if strings.Contains(query, "group by") {
		grouping = append(grouping, "group_by")
	}
	if strings.Contains(query, "having") {
		grouping = append(grouping, "having")
	}
	
	return grouping
}