package services

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"sort"
	"strings"
	"sync"
	"time"

	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/utils"
)

type DatabasePerformanceMonitor interface {
	StartMonitoring(ctx context.Context) error
	StopMonitoring() error
	GetSlowQueryAnalysis(ctx context.Context, threshold time.Duration) (*SlowQueryReport, error)
	GetQueryPerformanceMetrics(ctx context.Context) (*QueryPerformanceMetrics, error)
	GetDatabaseHealthReport(ctx context.Context) (*DatabaseHealthReport, error)
	SetAlertThresholds(thresholds *DatabaseAlertThresholds)
	GetOptimizationRecommendations(ctx context.Context) (*QueryOptimizationReport, error)
	ExportPerformanceData(ctx context.Context, format string, timeRange time.Duration) ([]byte, error)
}

type SlowQueryReport struct {
	ReportTime      time.Time     `json:"report_time"`
	TimeRange       time.Duration `json:"time_range"`
	TotalSlowQueries int64        `json:"total_slow_queries"`
	AverageSlowTime time.Duration `json:"average_slow_time"`
	TopSlowQueries  []SlowQueryAnalysis `json:"top_slow_queries"`
	QueryPatterns   []QueryPattern      `json:"query_patterns"`
	PerformanceGains []OptimizationGain `json:"performance_gains"`
}

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

type QueryPattern struct {
	Pattern         string        `json:"pattern"`
	Count           int64         `json:"count"`
	AverageTime     time.Duration `json:"average_time"`
	TotalTime       time.Duration `json:"total_time"`
	Queries         []string      `json:"example_queries"`
	Recommendations []string      `json:"recommendations"`
}

type QueryPerformanceMetrics struct {
	CollectedAt              time.Time             `json:"collected_at"`
	TimeRange               time.Duration         `json:"time_range"`
	TotalQueries            int64                `json:"total_queries"`
	SlowQueriesCount        int64                `json:"slow_queries_count"`
	SlowQueryPercentage     float64              `json:"slow_query_percentage"`
	AverageQueryTime        time.Duration        `json:"average_query_time"`
	P50QueryTime           time.Duration        `json:"p50_query_time"`
	P95QueryTime           time.Duration        `json:"p95_query_time"`
	P99QueryTime           time.Duration        `json:"p99_query_time"`
	QueryThroughputPerSec   float64              `json:"query_throughput_per_sec"`
	DatabaseConnections     DatabaseConnStats    `json:"database_connections"`
	IndexPerformance        IndexPerformanceData `json:"index_performance"`
	TablePerformance        []TablePerformanceData `json:"table_performance"`
	ConnectionPoolMetrics   ConnectionPoolMetrics `json:"connection_pool_metrics"`
}

type DatabaseConnStats struct {
	Active      int `json:"active"`
	Idle        int `json:"idle"`
	Total       int `json:"total"`
	MaxAllowed  int `json:"max_allowed"`
	Utilization float64 `json:"utilization_percentage"`
}

type IndexPerformanceData struct {
	TotalIndexes     int             `json:"total_indexes"`
	UnusedIndexes    []UnusedIndex   `json:"unused_indexes"`
	MostUsedIndexes  []IndexUsage    `json:"most_used_indexes"`
	IndexHitRatio    float64         `json:"index_hit_ratio"`
	IndexEfficiency  float64         `json:"index_efficiency"`
}

type UnusedIndex struct {
	IndexName  string    `json:"index_name"`
	TableName  string    `json:"table_name"`
	Size       int64     `json:"size_bytes"`
	LastUsed   *time.Time `json:"last_used,omitempty"`
	CreatedAt  time.Time `json:"created_at"`
}

type IndexUsage struct {
	IndexName     string  `json:"index_name"`
	TableName     string  `json:"table_name"`
	UsageCount    int64   `json:"usage_count"`
	HitRatio      float64 `json:"hit_ratio"`
	Efficiency    float64 `json:"efficiency"`
	LastUsed      time.Time `json:"last_used"`
}

type TablePerformanceData struct {
	TableName       string        `json:"table_name"`
	RowCount        int64         `json:"row_count"`
	TableSize       int64         `json:"table_size_bytes"`
	IndexSize       int64         `json:"index_size_bytes"`
	SequentialScans int64         `json:"sequential_scans"`
	IndexScans      int64         `json:"index_scans"`
	ScanRatio       float64       `json:"scan_ratio"`
	AvgQueryTime    time.Duration `json:"avg_query_time"`
	HotSpot         bool          `json:"hot_spot"`
}

type ConnectionPoolMetrics struct {
	ActiveConnections   int           `json:"active_connections"`
	IdleConnections     int           `json:"idle_connections"`
	WaitingQueries      int           `json:"waiting_queries"`
	AverageWaitTime     time.Duration `json:"average_wait_time"`
	ConnectionErrors    int64         `json:"connection_errors"`
	PoolUtilization     float64       `json:"pool_utilization"`
}

type DatabaseHealthReport struct {
	OverallHealth       string                `json:"overall_health"` // "excellent", "good", "warning", "critical"
	HealthScore         float64               `json:"health_score"`   // 0-100
	Issues              []DatabaseHealthIssue `json:"issues"`
	Recommendations     []DatabaseRecommendation `json:"recommendations"`
	PerformanceTrends   []PerformanceTrend    `json:"performance_trends"`
	ResourceUtilization DatabaseResourceUsage `json:"resource_utilization"`
	QueryAnalysisSummary QueryAnalysisSummary `json:"query_analysis_summary"`
	GeneratedAt         time.Time             `json:"generated_at"`
}

type DatabaseHealthIssue struct {
	ID          string    `json:"id"`
	Severity    string    `json:"severity"` // "critical", "high", "medium", "low"
	Category    string    `json:"category"` // "performance", "resource", "connection", "query"
	Title       string    `json:"title"`
	Description string    `json:"description"`
	Impact      string    `json:"impact"`
	DetectedAt  time.Time `json:"detected_at"`
	Resolved    bool      `json:"resolved"`
}

type DatabaseRecommendation struct {
	Priority        string  `json:"priority"` // "critical", "high", "medium", "low"
	Category        string  `json:"category"` // "index", "query", "connection", "configuration"
	Title           string  `json:"title"`
	Description     string  `json:"description"`
	ExpectedGain    string  `json:"expected_gain"`
	ImplementationEffort string `json:"implementation_effort"`
	SQLStatement    string  `json:"sql_statement,omitempty"`
}

type PerformanceTrend struct {
	Metric      string    `json:"metric"`
	Direction   string    `json:"direction"` // "improving", "degrading", "stable"
	ChangeRate  float64   `json:"change_rate"`
	CurrentValue float64   `json:"current_value"`
	PreviousValue float64  `json:"previous_value"`
	Timestamp   time.Time `json:"timestamp"`
}

type DatabaseResourceUsage struct {
	CPUUsage        float64 `json:"cpu_usage_percent"`
	MemoryUsage     float64 `json:"memory_usage_percent"`
	DiskUsage       float64 `json:"disk_usage_percent"`
	IOWaitTime      float64 `json:"io_wait_time_percent"`
	CacheHitRatio   float64 `json:"cache_hit_ratio"`
	BufferHitRatio  float64 `json:"buffer_hit_ratio"`
}

type QueryAnalysisSummary struct {
	TotalQueriesAnalyzed    int64   `json:"total_queries_analyzed"`
	SlowQueriesCount        int64   `json:"slow_queries_count"`
	OptimizableQueries      int64   `json:"optimizable_queries"`
	IndexMissingQueries     int64   `json:"index_missing_queries"`
	SequentialScansCount    int64   `json:"sequential_scans_count"`
	AvgOptimizationPotential float64 `json:"avg_optimization_potential"`
}

type DatabaseAlertThresholds struct {
	SlowQueryThreshold      time.Duration `json:"slow_query_threshold"`
	MaxSlowQueryPercentage  float64       `json:"max_slow_query_percentage"`
	MaxConnectionUtilization float64      `json:"max_connection_utilization"`
	MinCacheHitRatio        float64       `json:"min_cache_hit_ratio"`
	MaxCPUUsage             float64       `json:"max_cpu_usage"`
	MaxMemoryUsage          float64       `json:"max_memory_usage"`
	MinHealthScore          float64       `json:"min_health_score"`
}

type QueryOptimizationReport struct {
	GeneratedAt         time.Time                    `json:"generated_at"`
	AnalyzedQueries     int64                       `json:"analyzed_queries"`
	OptimizableQueries  []OptimizableQuery          `json:"optimizable_queries"`
	IndexRecommendations []IndexRecommendation       `json:"index_recommendations"`
	QueryRewrites       []QueryRewriteRecommendation `json:"query_rewrites"`
	PerformanceGains    []OptimizationGain          `json:"estimated_performance_gains"`
	ImplementationPlan  []ImplementationStep        `json:"implementation_plan"`
}

type OptimizableQuery struct {
	Query              string        `json:"query"`
	QueryPattern       string        `json:"query_pattern"`
	CurrentTime        time.Duration `json:"current_time"`
	EstimatedTime      time.Duration `json:"estimated_time_after_optimization"`
	ImprovementPotential float64     `json:"improvement_potential_percent"`
	Issues             []string      `json:"issues"`
	Recommendations    []string      `json:"recommendations"`
	Priority           string        `json:"priority"`
}

type IndexRecommendation struct {
	TableName       string   `json:"table_name"`
	Columns         []string `json:"columns"`
	IndexType       string   `json:"index_type"` // "btree", "hash", "gin", "gist"
	EstimatedGain   string   `json:"estimated_gain"`
	CreateStatement string   `json:"create_statement"`
	ImpactQueries   []string `json:"impact_queries"`
	Priority        string   `json:"priority"`
}

type QueryRewriteRecommendation struct {
	OriginalQuery   string `json:"original_query"`
	RewrittenQuery  string `json:"rewritten_query"`
	Reason          string `json:"reason"`
	EstimatedGain   string `json:"estimated_gain"`
	Confidence      string `json:"confidence"` // "high", "medium", "low"
}

type OptimizationGain struct {
	Category        string  `json:"category"`
	Description     string  `json:"description"`
	EstimatedGain   float64 `json:"estimated_gain_percent"`
	AffectedQueries int64   `json:"affected_queries"`
	Impact          string  `json:"impact"` // "high", "medium", "low"
}

type ImplementationStep struct {
	Step            int    `json:"step"`
	Action          string `json:"action"`
	Description     string `json:"description"`
	EstimatedTime   string `json:"estimated_time"`
	Risk            string `json:"risk"` // "low", "medium", "high"
	Prerequisites   []string `json:"prerequisites"`
	SQLStatements   []string `json:"sql_statements,omitempty"`
}

type databasePerformanceMonitor struct {
	db                *gorm.DB
	queryMonitor      QueryExecutionMonitor
	cacheMonitor      RecipeCacheMonitor
	alertThresholds   *DatabaseAlertThresholds
	isMonitoring      bool
	stopChan          chan struct{}
	metrics           *QueryPerformanceMetrics
	slowQueries       []SlowQueryAnalysis
	healthHistory     []DatabaseHealthReport
	mu                sync.RWMutex
	alerts            []DatabaseHealthIssue
}

func NewDatabasePerformanceMonitor(
	db *gorm.DB,
	queryMonitor QueryExecutionMonitor,
	cacheMonitor RecipeCacheMonitor,
) DatabasePerformanceMonitor {
	return &databasePerformanceMonitor{
		db:           db,
		queryMonitor: queryMonitor,
		cacheMonitor: cacheMonitor,
		alertThresholds: &DatabaseAlertThresholds{
			SlowQueryThreshold:       time.Millisecond * 500,
			MaxSlowQueryPercentage:   5.0,
			MaxConnectionUtilization: 80.0,
			MinCacheHitRatio:         0.90,
			MaxCPUUsage:              80.0,
			MaxMemoryUsage:           85.0,
			MinHealthScore:           75.0,
		},
		stopChan:      make(chan struct{}),
		slowQueries:   make([]SlowQueryAnalysis, 0),
		healthHistory: make([]DatabaseHealthReport, 0, 144), // 24 hours of 10-minute intervals
		alerts:        make([]DatabaseHealthIssue, 0),
	}
}

func (d *databasePerformanceMonitor) StartMonitoring(ctx context.Context) error {
	d.mu.Lock()
	defer d.mu.Unlock()

	if d.isMonitoring {
		return fmt.Errorf("database monitoring already started")
	}

	d.isMonitoring = true
	log.Printf("Starting database performance monitoring...")

	// Start performance metrics collection
	go d.performanceMetricsLoop(ctx)

	// Start slow query analysis
	go d.slowQueryAnalysisLoop(ctx)

	// Start health monitoring
	go d.healthMonitoringLoop(ctx)

	// Start alerting
	go d.alertingLoop(ctx)

	log.Printf("Database performance monitoring started successfully")
	return nil
}

func (d *databasePerformanceMonitor) StopMonitoring() error {
	d.mu.Lock()
	defer d.mu.Unlock()

	if !d.isMonitoring {
		return fmt.Errorf("database monitoring not started")
	}

	close(d.stopChan)
	d.isMonitoring = false

	log.Printf("Database performance monitoring stopped")
	return nil
}

func (d *databasePerformanceMonitor) GetSlowQueryAnalysis(ctx context.Context, threshold time.Duration) (*SlowQueryReport, error) {
	d.mu.RLock()
	defer d.mu.RUnlock()

	// Get slow queries from the execution monitor
	slowQueries, err := d.queryMonitor.GetSlowQueriesFromLog(ctx, threshold, 100)
	if err != nil {
		return nil, fmt.Errorf("failed to get slow queries: %w", err)
	}

	// Analyze and aggregate slow queries
	report := &SlowQueryReport{
		ReportTime:       time.Now(),
		TimeRange:        time.Hour * 24, // Last 24 hours
		TotalSlowQueries: int64(len(slowQueries)),
		TopSlowQueries:   make([]SlowQueryAnalysis, 0),
		QueryPatterns:    make([]QueryPattern, 0),
		PerformanceGains: make([]OptimizationGain, 0),
	}

	if len(slowQueries) == 0 {
		return report, nil
	}

	// Calculate average slow time
	var totalTime time.Duration
	for _, sq := range slowQueries {
		totalTime += sq.Duration
	}
	report.AverageSlowTime = totalTime / time.Duration(len(slowQueries))

	// Analyze query patterns and group similar queries
	patternMap := make(map[string]*QueryPattern)
	for _, sq := range slowQueries {
		pattern := d.extractQueryPattern(sq.Query)
		
		if existing, exists := patternMap[pattern]; exists {
			existing.Count++
			existing.TotalTime += sq.Duration
			existing.AverageTime = existing.TotalTime / time.Duration(existing.Count)
			if len(existing.Queries) < 3 {
				existing.Queries = append(existing.Queries, sq.Query)
			}
		} else {
			patternMap[pattern] = &QueryPattern{
				Pattern:     pattern,
				Count:       1,
				AverageTime: sq.Duration,
				TotalTime:   sq.Duration,
				Queries:     []string{sq.Query},
				Recommendations: d.generateQueryPatternRecommendations(pattern),
			}
		}
	}

	// Convert pattern map to slice and sort by total time
	for _, pattern := range patternMap {
		report.QueryPatterns = append(report.QueryPatterns, *pattern)
	}

	sort.Slice(report.QueryPatterns, func(i, j int) bool {
		return report.QueryPatterns[i].TotalTime > report.QueryPatterns[j].TotalTime
	})

	// Generate top slow queries analysis
	report.TopSlowQueries = d.analyzeTopSlowQueries(slowQueries[:utils.MinInt(len(slowQueries), 20)])

	// Calculate potential performance gains
	report.PerformanceGains = d.calculatePerformanceGains(report.QueryPatterns)

	return report, nil
}

func (d *databasePerformanceMonitor) GetQueryPerformanceMetrics(ctx context.Context) (*QueryPerformanceMetrics, error) {
	metrics := &QueryPerformanceMetrics{
		CollectedAt: time.Now(),
		TimeRange:   time.Hour, // Last hour
	}

	// Get database statistics
	if err := d.collectDatabaseStats(ctx, metrics); err != nil {
		return nil, fmt.Errorf("failed to collect database stats: %w", err)
	}

	// Get connection pool metrics
	if err := d.collectConnectionPoolMetrics(ctx, metrics); err != nil {
		log.Printf("Failed to collect connection pool metrics: %v", err)
		// Continue with partial metrics
	}

	// Get index performance data
	if err := d.collectIndexPerformanceData(ctx, metrics); err != nil {
		log.Printf("Failed to collect index performance data: %v", err)
		// Continue with partial metrics
	}

	// Get table performance data
	if err := d.collectTablePerformanceData(ctx, metrics); err != nil {
		log.Printf("Failed to collect table performance data: %v", err)
		// Continue with partial metrics
	}

	d.mu.Lock()
	d.metrics = metrics
	d.mu.Unlock()

	return metrics, nil
}

func (d *databasePerformanceMonitor) GetDatabaseHealthReport(ctx context.Context) (*DatabaseHealthReport, error) {
	metrics, err := d.GetQueryPerformanceMetrics(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get performance metrics: %w", err)
	}

	health := &DatabaseHealthReport{
		GeneratedAt:      time.Now(),
		Issues:           make([]DatabaseHealthIssue, 0),
		Recommendations:  make([]DatabaseRecommendation, 0),
		PerformanceTrends: make([]PerformanceTrend, 0),
	}

	// Calculate health score
	healthScore := d.calculateHealthScore(metrics)
	health.HealthScore = healthScore

	// Determine overall health status
	if healthScore >= 90 {
		health.OverallHealth = "excellent"
	} else if healthScore >= 80 {
		health.OverallHealth = "good"
	} else if healthScore >= 60 {
		health.OverallHealth = "warning"
	} else {
		health.OverallHealth = "critical"
	}

	// Analyze health issues
	d.analyzeHealthIssues(metrics, health)

	// Generate recommendations
	d.generateHealthRecommendations(metrics, health)

	// Calculate performance trends (if we have historical data)
	d.calculatePerformanceTrends(health)

	// Resource utilization (simulated for demonstration)
	health.ResourceUtilization = DatabaseResourceUsage{
		CPUUsage:       65.0,
		MemoryUsage:    78.0,
		DiskUsage:      45.0,
		IOWaitTime:     12.0,
		CacheHitRatio:  metrics.IndexPerformance.IndexHitRatio,
		BufferHitRatio: 0.95,
	}

	// Query analysis summary
	health.QueryAnalysisSummary = QueryAnalysisSummary{
		TotalQueriesAnalyzed: metrics.TotalQueries,
		SlowQueriesCount:     metrics.SlowQueriesCount,
		OptimizableQueries:   int64(float64(metrics.SlowQueriesCount) * 0.8), // Estimate 80% are optimizable
		IndexMissingQueries:  int64(len(metrics.IndexPerformance.UnusedIndexes)),
		SequentialScansCount: d.calculateTotalSequentialScans(metrics.TablePerformance),
		AvgOptimizationPotential: 25.0, // Simulated average
	}

	// Store in history
	d.mu.Lock()
	d.healthHistory = append(d.healthHistory, *health)
	if len(d.healthHistory) > 144 { // Keep last 24 hours
		d.healthHistory = d.healthHistory[1:]
	}
	d.mu.Unlock()

	return health, nil
}

func (d *databasePerformanceMonitor) SetAlertThresholds(thresholds *DatabaseAlertThresholds) {
	d.mu.Lock()
	defer d.mu.Unlock()

	d.alertThresholds = thresholds
	log.Printf("Updated database alert thresholds")
}

func (d *databasePerformanceMonitor) GetOptimizationRecommendations(ctx context.Context) (*QueryOptimizationReport, error) {
	report := &QueryOptimizationReport{
		GeneratedAt:         time.Now(),
		OptimizableQueries:  make([]OptimizableQuery, 0),
		IndexRecommendations: make([]IndexRecommendation, 0),
		QueryRewrites:       make([]QueryRewriteRecommendation, 0),
		PerformanceGains:    make([]OptimizationGain, 0),
		ImplementationPlan:  make([]ImplementationStep, 0),
	}

	// Get slow query analysis
	slowQueryReport, err := d.GetSlowQueryAnalysis(ctx, d.alertThresholds.SlowQueryThreshold)
	if err != nil {
		return nil, fmt.Errorf("failed to get slow query analysis: %w", err)
	}

	report.AnalyzedQueries = slowQueryReport.TotalSlowQueries

	// Analyze queries for optimization opportunities
	for _, pattern := range slowQueryReport.QueryPatterns {
		if pattern.AverageTime > d.alertThresholds.SlowQueryThreshold {
			optimizable := OptimizableQuery{
				QueryPattern:         pattern.Pattern,
				CurrentTime:          pattern.AverageTime,
				EstimatedTime:        time.Duration(float64(pattern.AverageTime) * 0.6), // Assume 40% improvement
				ImprovementPotential: 40.0,
				Issues:              d.identifyQueryIssues(pattern.Pattern),
				Recommendations:     pattern.Recommendations,
				Priority:            d.calculateOptimizationPriority(pattern),
			}
			if len(pattern.Queries) > 0 {
				optimizable.Query = pattern.Queries[0]
			}
			report.OptimizableQueries = append(report.OptimizableQueries, optimizable)
		}
	}

	// Generate index recommendations
	report.IndexRecommendations = d.generateIndexRecommendations(ctx, slowQueryReport.QueryPatterns)

	// Generate query rewrite recommendations
	report.QueryRewrites = d.generateQueryRewrites(report.OptimizableQueries)

	// Calculate performance gains
	report.PerformanceGains = slowQueryReport.PerformanceGains

	// Generate implementation plan
	report.ImplementationPlan = d.generateImplementationPlan(report)

	return report, nil
}

func (d *databasePerformanceMonitor) ExportPerformanceData(ctx context.Context, format string, timeRange time.Duration) ([]byte, error) {
	switch format {
	case "json":
		return d.exportJSONPerformanceData(ctx, timeRange)
	case "prometheus":
		return d.exportPrometheusPerformanceData(ctx, timeRange)
	case "csv":
		return d.exportCSVPerformanceData(ctx, timeRange)
	default:
		return nil, fmt.Errorf("unsupported export format: %s", format)
	}
}

// Helper methods and monitoring loops

func (d *databasePerformanceMonitor) performanceMetricsLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Minute * 5) // Collect metrics every 5 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			if _, err := d.GetQueryPerformanceMetrics(ctx); err != nil {
				log.Printf("Failed to collect performance metrics: %v", err)
			}
		case <-d.stopChan:
			return
		case <-ctx.Done():
			return
		}
	}
}

func (d *databasePerformanceMonitor) slowQueryAnalysisLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Minute * 10) // Analyze slow queries every 10 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			if _, err := d.GetSlowQueryAnalysis(ctx, d.alertThresholds.SlowQueryThreshold); err != nil {
				log.Printf("Failed to analyze slow queries: %v", err)
			}
		case <-d.stopChan:
			return
		case <-ctx.Done():
			return
		}
	}
}

func (d *databasePerformanceMonitor) healthMonitoringLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Minute * 10) // Health check every 10 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			health, err := d.GetDatabaseHealthReport(ctx)
			if err != nil {
				log.Printf("Failed to get database health report: %v", err)
				continue
			}

			// Check for critical issues
			for _, issue := range health.Issues {
				if issue.Severity == "critical" {
					d.triggerAlert(issue)
				}
			}

			// Check overall health score
			if health.HealthScore < d.alertThresholds.MinHealthScore {
				d.triggerAlert(DatabaseHealthIssue{
					ID:          fmt.Sprintf("health_score_%d", time.Now().Unix()),
					Severity:    "high",
					Category:    "performance",
					Title:       "Database Health Score Below Threshold",
					Description: fmt.Sprintf("Database health score is %.2f, below threshold of %.2f", health.HealthScore, d.alertThresholds.MinHealthScore),
					Impact:      "Overall database performance may be degraded",
					DetectedAt:  time.Now(),
					Resolved:    false,
				})
			}

		case <-d.stopChan:
			return
		case <-ctx.Done():
			return
		}
	}
}

func (d *databasePerformanceMonitor) alertingLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Minute * 5) // Process alerts every 5 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			d.processAlerts()
		case <-d.stopChan:
			return
		case <-ctx.Done():
			return
		}
	}
}

// Implementation continues with helper methods...
// (The following methods would be implemented with specific database query logic)

func (d *databasePerformanceMonitor) collectDatabaseStats(ctx context.Context, metrics *QueryPerformanceMetrics) error {
	// This would query PostgreSQL system tables for actual statistics
	// For demonstration, using simulated values
	
	metrics.TotalQueries = 10000
	metrics.SlowQueriesCount = 250
	metrics.SlowQueryPercentage = 2.5
	metrics.AverageQueryTime = time.Millisecond * 45
	metrics.P50QueryTime = time.Millisecond * 25
	metrics.P95QueryTime = time.Millisecond * 150
	metrics.P99QueryTime = time.Millisecond * 300
	metrics.QueryThroughputPerSec = 166.67 // 10000 queries per minute

	metrics.DatabaseConnections = DatabaseConnStats{
		Active:      15,
		Idle:        5,
		Total:       20,
		MaxAllowed:  100,
		Utilization: 20.0,
	}

	return nil
}

func (d *databasePerformanceMonitor) collectConnectionPoolMetrics(ctx context.Context, metrics *QueryPerformanceMetrics) error {
	// This would collect actual connection pool statistics
	metrics.ConnectionPoolMetrics = ConnectionPoolMetrics{
		ActiveConnections:   15,
		IdleConnections:     5,
		WaitingQueries:      2,
		AverageWaitTime:     time.Millisecond * 5,
		ConnectionErrors:    3,
		PoolUtilization:     75.0,
	}
	return nil
}

func (d *databasePerformanceMonitor) collectIndexPerformanceData(ctx context.Context, metrics *QueryPerformanceMetrics) error {
	// This would query pg_stat_user_indexes and related tables
	metrics.IndexPerformance = IndexPerformanceData{
		TotalIndexes:     25,
		IndexHitRatio:    0.95,
		IndexEfficiency:  0.88,
		UnusedIndexes:    []UnusedIndex{},
		MostUsedIndexes:  []IndexUsage{},
	}
	return nil
}

func (d *databasePerformanceMonitor) collectTablePerformanceData(ctx context.Context, metrics *QueryPerformanceMetrics) error {
	// This would query pg_stat_user_tables and related tables
	metrics.TablePerformance = []TablePerformanceData{
		{
			TableName:       "recipes",
			RowCount:        10000,
			TableSize:       1024 * 1024 * 50, // 50MB
			IndexSize:       1024 * 1024 * 10, // 10MB
			SequentialScans: 150,
			IndexScans:      8500,
			ScanRatio:       0.982,
			AvgQueryTime:    time.Millisecond * 25,
			HotSpot:         true,
		},
	}
	return nil
}

// Additional helper methods would be implemented here...
// (calculateHealthScore, analyzeHealthIssues, generateHealthRecommendations, etc.)

func (d *databasePerformanceMonitor) calculateHealthScore(metrics *QueryPerformanceMetrics) float64 {
	score := 100.0

	// Slow query penalty (30% of score)
	if metrics.SlowQueryPercentage > 5.0 {
		score -= (metrics.SlowQueryPercentage - 5.0) * 3
	}

	// Connection utilization penalty (20% of score)
	if metrics.DatabaseConnections.Utilization > 80.0 {
		score -= (metrics.DatabaseConnections.Utilization - 80.0) * 0.5
	}

	// Index efficiency penalty (25% of score)
	if metrics.IndexPerformance.IndexHitRatio < 0.90 {
		score -= (0.90 - metrics.IndexPerformance.IndexHitRatio) * 100
	}

	// Query latency penalty (25% of score)
	if metrics.P95QueryTime > time.Millisecond*200 {
		excess := float64(metrics.P95QueryTime-time.Millisecond*200) / float64(time.Millisecond)
		score -= excess * 0.1
	}

	if score < 0 {
		score = 0
	}

	return score
}

func (d *databasePerformanceMonitor) analyzeHealthIssues(metrics *QueryPerformanceMetrics, health *DatabaseHealthReport) {
	// Analyze various health aspects and add issues
	if metrics.SlowQueryPercentage > d.alertThresholds.MaxSlowQueryPercentage {
		health.Issues = append(health.Issues, DatabaseHealthIssue{
			ID:          "slow_query_high",
			Severity:    "high",
			Category:    "performance",
			Title:       "High Slow Query Percentage",
			Description: fmt.Sprintf("Slow queries represent %.2f%% of total queries, above threshold of %.2f%%", metrics.SlowQueryPercentage, d.alertThresholds.MaxSlowQueryPercentage),
			Impact:      "Degraded application response times and user experience",
			DetectedAt:  time.Now(),
			Resolved:    false,
		})
	}

	if metrics.DatabaseConnections.Utilization > d.alertThresholds.MaxConnectionUtilization {
		health.Issues = append(health.Issues, DatabaseHealthIssue{
			ID:          "connection_high",
			Severity:    "medium",
			Category:    "connection",
			Title:       "High Connection Pool Utilization",
			Description: fmt.Sprintf("Connection pool utilization is %.2f%%, above threshold of %.2f%%", metrics.DatabaseConnections.Utilization, d.alertThresholds.MaxConnectionUtilization),
			Impact:      "Potential connection pool exhaustion and query queuing",
			DetectedAt:  time.Now(),
			Resolved:    false,
		})
	}
}

func (d *databasePerformanceMonitor) generateHealthRecommendations(metrics *QueryPerformanceMetrics, health *DatabaseHealthReport) {
	// Generate recommendations based on detected issues
	if metrics.SlowQueryPercentage > 3.0 {
		health.Recommendations = append(health.Recommendations, DatabaseRecommendation{
			Priority:    "high",
			Category:    "query",
			Title:       "Optimize Slow Queries",
			Description: "Analyze and optimize the slowest queries to improve overall performance",
			ExpectedGain: "15-30% reduction in query response time",
			ImplementationEffort: "Medium",
		})
	}

	if len(metrics.IndexPerformance.UnusedIndexes) > 5 {
		health.Recommendations = append(health.Recommendations, DatabaseRecommendation{
			Priority:    "medium",
			Category:    "index",
			Title:       "Remove Unused Indexes",
			Description: "Drop unused indexes to reduce storage overhead and improve write performance",
			ExpectedGain: "10-15% improvement in write operations",
			ImplementationEffort: "Low",
		})
	}
}

func (d *databasePerformanceMonitor) calculatePerformanceTrends(health *DatabaseHealthReport) {
	// Calculate trends if we have historical data
	d.mu.RLock()
	defer d.mu.RUnlock()

	if len(d.healthHistory) < 2 {
		return
	}

	current := d.healthHistory[len(d.healthHistory)-1]
	previous := d.healthHistory[len(d.healthHistory)-2]

	// Health score trend
	healthChange := current.HealthScore - previous.HealthScore
	healthDirection := "stable"
	if healthChange > 1.0 {
		healthDirection = "improving"
	} else if healthChange < -1.0 {
		healthDirection = "degrading"
	}

	health.PerformanceTrends = append(health.PerformanceTrends, PerformanceTrend{
		Metric:        "health_score",
		Direction:     healthDirection,
		ChangeRate:    healthChange,
		CurrentValue:  current.HealthScore,
		PreviousValue: previous.HealthScore,
		Timestamp:     time.Now(),
	})
}

func (d *databasePerformanceMonitor) calculateTotalSequentialScans(tableData []TablePerformanceData) int64 {
	var total int64
	for _, table := range tableData {
		total += table.SequentialScans
	}
	return total
}

func (d *databasePerformanceMonitor) triggerAlert(issue DatabaseHealthIssue) {
	d.mu.Lock()
	d.alerts = append(d.alerts, issue)
	d.mu.Unlock()

	log.Printf("DATABASE ALERT [%s]: %s - %s", issue.Severity, issue.Title, issue.Description)
}

func (d *databasePerformanceMonitor) processAlerts() {
	// Process alerts and send notifications
	// Implementation would depend on alerting infrastructure
}

// Placeholder implementations for additional helper methods
func (d *databasePerformanceMonitor) extractQueryPattern(query string) string {
	// Extract query pattern by replacing literals with placeholders
	// This is a simplified implementation
	return strings.ToLower(strings.Fields(query)[0]) // Return first word (SELECT, INSERT, etc.)
}

func (d *databasePerformanceMonitor) generateQueryPatternRecommendations(pattern string) []string {
	return []string{"Consider adding appropriate indexes", "Analyze query execution plan"}
}

func (d *databasePerformanceMonitor) analyzeTopSlowQueries(queries []SlowQuery) []SlowQueryAnalysis {
	result := make([]SlowQueryAnalysis, 0, len(queries))
	
	for _, q := range queries {
		analysis := SlowQueryAnalysis{
			Query:           q.Query,
			QueryPattern:    d.extractQueryPattern(q.Query),
			Count:           1,
			TotalTime:       q.Duration,
			AverageTime:     q.Duration,
			MinTime:         q.Duration,
			MaxTime:         q.Duration,
			LastSeen:        q.Timestamp,
			Recommendations: d.generateQueryPatternRecommendations(q.Query),
			Severity:        d.calculateQuerySeverity(q.Duration),
		}
		result = append(result, analysis)
	}
	
	return result
}

func (d *databasePerformanceMonitor) calculateQuerySeverity(duration time.Duration) string {
	if duration > time.Second*5 {
		return "critical"
	} else if duration > time.Second*2 {
		return "high"
	} else if duration > time.Millisecond*500 {
		return "medium"
	}
	return "low"
}

func (d *databasePerformanceMonitor) calculatePerformanceGains(patterns []QueryPattern) []OptimizationGain {
	return []OptimizationGain{
		{
			Category:        "Index Optimization",
			Description:     "Adding missing indexes for frequently accessed columns",
			EstimatedGain:   35.0,
			AffectedQueries: int64(len(patterns)),
			Impact:          "high",
		},
		{
			Category:        "Query Rewriting", 
			Description:     "Optimizing subqueries and joins",
			EstimatedGain:   20.0,
			AffectedQueries: int64(len(patterns) / 2),
			Impact:          "medium",
		},
	}
}

func (d *databasePerformanceMonitor) calculateOptimizationPriority(pattern QueryPattern) string {
	if pattern.AverageTime > time.Second*2 && pattern.Count > 100 {
		return "critical"
	} else if pattern.AverageTime > time.Second || pattern.Count > 50 {
		return "high"
	} else if pattern.AverageTime > time.Millisecond*500 {
		return "medium"
	}
	return "low"
}

func (d *databasePerformanceMonitor) identifyQueryIssues(pattern string) []string {
	return []string{"Sequential scan detected", "Missing index on join condition"}
}

func (d *databasePerformanceMonitor) generateIndexRecommendations(ctx context.Context, patterns []QueryPattern) []IndexRecommendation {
	return []IndexRecommendation{
		{
			TableName:       "recipes",
			Columns:         []string{"cuisine", "dietary_restrictions"},
			IndexType:       "btree",
			EstimatedGain:   "30-50% improvement in search queries",
			CreateStatement: "CREATE INDEX CONCURRENTLY idx_recipes_cuisine_dietary ON recipes (cuisine, dietary_restrictions);",
			ImpactQueries:   []string{"SELECT * FROM recipes WHERE cuisine = ? AND dietary_restrictions @> ?"},
			Priority:        "high",
		},
	}
}

func (d *databasePerformanceMonitor) generateQueryRewrites(optimizable []OptimizableQuery) []QueryRewriteRecommendation {
	return []QueryRewriteRecommendation{
		{
			OriginalQuery:  "SELECT * FROM recipes WHERE id IN (SELECT recipe_id FROM favorites WHERE user_id = ?)",
			RewrittenQuery: "SELECT r.* FROM recipes r INNER JOIN favorites f ON r.id = f.recipe_id WHERE f.user_id = ?",
			Reason:         "Replace subquery with JOIN for better performance",
			EstimatedGain:  "25% faster execution",
			Confidence:     "high",
		},
	}
}

func (d *databasePerformanceMonitor) generateImplementationPlan(report *QueryOptimizationReport) []ImplementationStep {
	return []ImplementationStep{
		{
			Step:          1,
			Action:        "Create Missing Indexes",
			Description:   "Add high-priority indexes for frequently accessed columns",
			EstimatedTime: "30 minutes",
			Risk:          "low",
			Prerequisites: []string{"Database backup", "Maintenance window"},
			SQLStatements: []string{"CREATE INDEX CONCURRENTLY idx_recipes_cuisine ON recipes (cuisine);"},
		},
	}
}

// Export method implementations
func (d *databasePerformanceMonitor) exportJSONPerformanceData(ctx context.Context, timeRange time.Duration) ([]byte, error) {
	metrics, err := d.GetQueryPerformanceMetrics(ctx)
	if err != nil {
		return nil, err
	}
	return json.MarshalIndent(metrics, "", "  ")
}

func (d *databasePerformanceMonitor) exportPrometheusPerformanceData(ctx context.Context, timeRange time.Duration) ([]byte, error) {
	metrics, err := d.GetQueryPerformanceMetrics(ctx)
	if err != nil {
		return nil, err
	}

	prometheusFormat := fmt.Sprintf(`
# HELP db_total_queries_total Total database queries executed
# TYPE db_total_queries_total counter
db_total_queries_total %d

# HELP db_slow_queries_total Total slow queries executed
# TYPE db_slow_queries_total counter
db_slow_queries_total %d

# HELP db_query_duration_seconds Query execution time distribution
# TYPE db_query_duration_seconds histogram
db_query_duration_seconds_p50 %f
db_query_duration_seconds_p95 %f
db_query_duration_seconds_p99 %f

# HELP db_connections_active Active database connections
# TYPE db_connections_active gauge
db_connections_active %d
`, metrics.TotalQueries, metrics.SlowQueriesCount, 
   metrics.P50QueryTime.Seconds(), metrics.P95QueryTime.Seconds(), metrics.P99QueryTime.Seconds(),
   metrics.DatabaseConnections.Active)

	return []byte(prometheusFormat), nil
}

func (d *databasePerformanceMonitor) exportCSVPerformanceData(ctx context.Context, timeRange time.Duration) ([]byte, error) {
	// Simplified CSV export implementation
	csvData := "timestamp,total_queries,slow_queries,avg_time_ms,p95_time_ms\n"
	metrics, err := d.GetQueryPerformanceMetrics(ctx)
	if err != nil {
		return nil, err
	}

	csvData += fmt.Sprintf("%s,%d,%d,%.2f,%.2f\n",
		metrics.CollectedAt.Format(time.RFC3339),
		metrics.TotalQueries,
		metrics.SlowQueriesCount,
		float64(metrics.AverageQueryTime.Nanoseconds())/1000000,
		float64(metrics.P95QueryTime.Nanoseconds())/1000000)

	return []byte(csvData), nil
}

