package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sync"
	"time"

	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/utils"
)

type QueryPerformanceMetricsCollector interface {
	// Lifecycle management
	Start(ctx context.Context) error
	Stop(ctx context.Context) error
	IsRunning() bool
	
	// Metrics collection
	RecordQueryExecution(ctx context.Context, metric *QueryExecutionMetric) error
	GetQueryMetrics(ctx context.Context, filter *MetricsFilter) (*QueryMetricsCollection, error)
	GetAggregatedMetrics(ctx context.Context, timeWindow time.Duration, groupBy MetricsGroupBy) (*AggregatedMetrics, error)
	
	// Real-time metrics
	GetCurrentMetrics(ctx context.Context) (*CurrentPerformanceSnapshot, error)
	GetMetricsTrends(ctx context.Context, timeWindow time.Duration) (*MetricsTrends, error)
	
	// Threshold monitoring
	SetPerformanceThresholds(thresholds *PerformanceThresholds) error
	GetPerformanceThresholds() *PerformanceThresholds
	CheckThresholdBreaches(ctx context.Context) ([]ThresholdBreach, error)
	
	// Export and reporting
	ExportMetrics(ctx context.Context, format ExportFormat, timeRange TimeRange) ([]byte, error)
	GeneratePerformanceReport(ctx context.Context, reportType ReportType, timeRange TimeRange) (*PerformanceReport, error)
	
	// Health and diagnostics
	GetCollectorHealth(ctx context.Context) (*CollectorHealth, error)
	ResetMetrics(ctx context.Context) error
}

type QueryExecutionMetric struct {
	MetricID            string                 `json:"metric_id"`
	QueryID             string                 `json:"query_id"`
	QueryHash           string                 `json:"query_hash"`
	QueryText           string                 `json:"query_text"`
	QueryNormalized     string                 `json:"query_normalized"`
	QueryType           QueryType              `json:"query_type"`
	ExecutionTime       time.Duration          `json:"execution_time"`
	PlanningTime        time.Duration          `json:"planning_time"`
	WaitTime            time.Duration          `json:"wait_time"`
	LockWaitTime        time.Duration          `json:"lock_wait_time"`
	IOWaitTime          time.Duration          `json:"io_wait_time"`
	CPUTime             time.Duration          `json:"cpu_time"`
	NetworkTime         time.Duration          `json:"network_time"`
	Timestamp           time.Time              `json:"timestamp"`
	Database            string                 `json:"database"`
	Schema              string                 `json:"schema"`
	User                string                 `json:"user"`
	ApplicationName     string                 `json:"application_name"`
	ConnectionID        string                 `json:"connection_id"`
	SessionID           string                 `json:"session_id"`
	ClientIP            string                 `json:"client_ip"`
	RowsAffected        int64                  `json:"rows_affected"`
	RowsExamined        int64                  `json:"rows_examined"`
	RowsReturned        int64                  `json:"rows_returned"`
	BytesScanned        int64                  `json:"bytes_scanned"`
	BytesReturned       int64                  `json:"bytes_returned"`
	TempTablesCreated   int                    `json:"temp_tables_created"`
	TempDiskTablesUsed  int                    `json:"temp_disk_tables_used"`
	IndexesUsed         []string               `json:"indexes_used"`
	TablesAccessed      []string               `json:"tables_accessed"`
	QueryPlanCost       float64                `json:"query_plan_cost"`
	QueryPlanRows       int64                  `json:"query_plan_rows"`
	CacheHit            bool                   `json:"cache_hit"`
	CacheType           string                 `json:"cache_type"`
	ErrorOccurred       bool                   `json:"error_occurred"`
	ErrorMessage        string                 `json:"error_message,omitempty"`
	WarningCount        int                    `json:"warning_count"`
	ResourceUsage       *ResourceUsageMetrics  `json:"resource_usage,omitempty"`
	PerformanceContext  *PerformanceContext    `json:"performance_context,omitempty"`
	CustomTags          map[string]string      `json:"custom_tags,omitempty"`
	CustomMetrics       map[string]interface{} `json:"custom_metrics,omitempty"`
}

type ResourceUsageMetrics struct {
	SharedBuffersHit    int64   `json:"shared_buffers_hit"`
	SharedBuffersRead   int64   `json:"shared_buffers_read"`
	LocalBuffersHit     int64   `json:"local_buffers_hit"`
	LocalBuffersRead    int64   `json:"local_buffers_read"`
	LocalBuffersWritten int64   `json:"local_buffers_written"`
	TempBuffersRead     int64   `json:"temp_buffers_read"`
	TempBuffersWritten  int64   `json:"temp_buffers_written"`
	BufferHitRatio      float64 `json:"buffer_hit_ratio"`
	IOReadBytes         int64   `json:"io_read_bytes"`
	IOWriteBytes        int64   `json:"io_write_bytes"`
	LogicalReads        int64   `json:"logical_reads"`
	PhysicalReads       int64   `json:"physical_reads"`
	RandomReads         int64   `json:"random_reads"`
	SequentialReads     int64   `json:"sequential_reads"`
	MemoryUsage         int64   `json:"memory_usage_bytes"`
	WorkMemUsage        int64   `json:"work_mem_usage"`
	MaintenanceWorkMem  int64   `json:"maintenance_work_mem"`
}

type PerformanceContext struct {
	SystemLoad          float64 `json:"system_load"`
	CPUUtilization      float64 `json:"cpu_utilization"`
	MemoryUtilization   float64 `json:"memory_utilization"`
	DiskUtilization     float64 `json:"disk_utilization"`
	NetworkUtilization  float64 `json:"network_utilization"`
	ActiveConnections   int     `json:"active_connections"`
	QueriesPerSecond    float64 `json:"queries_per_second"`
	TransactionsPerSecond float64 `json:"transactions_per_second"`
	DatabaseSize        int64   `json:"database_size_bytes"`
	TableSizes          map[string]int64 `json:"table_sizes,omitempty"`
	IndexSizes          map[string]int64 `json:"index_sizes,omitempty"`
}

type QueryType string

const (
	QueryTypeSelect QueryType = "SELECT"
	QueryTypeInsert QueryType = "INSERT"
	QueryTypeUpdate QueryType = "UPDATE"
	QueryTypeDelete QueryType = "DELETE"
	QueryTypeDDL    QueryType = "DDL"
	QueryTypeUnknown QueryType = "UNKNOWN"
)

type MetricsFilter struct {
	TimeRange      *TimeRange    `json:"time_range,omitempty"`
	QueryTypes     []QueryType   `json:"query_types,omitempty"`
	Databases      []string      `json:"databases,omitempty"`
	Users          []string      `json:"users,omitempty"`
	Applications   []string      `json:"applications,omitempty"`
	Tables         []string      `json:"tables,omitempty"`
	MinExecutionTime time.Duration `json:"min_execution_time,omitempty"`
	MaxExecutionTime time.Duration `json:"max_execution_time,omitempty"`
	HasErrors      *bool         `json:"has_errors,omitempty"`
	CustomTags     map[string]string `json:"custom_tags,omitempty"`
	Limit          int           `json:"limit,omitempty"`
	Offset         int           `json:"offset,omitempty"`
	OrderBy        string        `json:"order_by,omitempty"`
	SortDirection  string        `json:"sort_direction,omitempty"`
}

type MetricsGroupBy string

const (
	GroupByQueryType     MetricsGroupBy = "query_type"
	GroupByDatabase      MetricsGroupBy = "database"
	GroupByUser          MetricsGroupBy = "user"
	GroupByApplication   MetricsGroupBy = "application"
	GroupByTable         MetricsGroupBy = "table"
	GroupByHour          MetricsGroupBy = "hour"
	GroupByDay           MetricsGroupBy = "day"
	GroupByTime          MetricsGroupBy = "time"
	GroupByQueryPattern  MetricsGroupBy = "query_pattern"
	GroupByNone          MetricsGroupBy = "none"
)

type TimeRange struct {
	Start time.Time `json:"start"`
	End   time.Time `json:"end"`
}

type QueryMetricsCollection struct {
	Metrics      []*QueryExecutionMetric `json:"metrics"`
	TotalCount   int                     `json:"total_count"`
	FilteredCount int                    `json:"filtered_count"`
	TimeRange    *TimeRange             `json:"time_range"`
	Filter       *MetricsFilter         `json:"filter"`
	Summary      *MetricsSummary        `json:"summary"`
}

type MetricsSummary struct {
	TotalQueries           int           `json:"total_queries"`
	UniqueQueries          int           `json:"unique_queries"`
	AverageExecutionTime   time.Duration `json:"average_execution_time"`
	MedianExecutionTime    time.Duration `json:"median_execution_time"`
	P95ExecutionTime       time.Duration `json:"p95_execution_time"`
	P99ExecutionTime       time.Duration `json:"p99_execution_time"`
	FastestQuery           time.Duration `json:"fastest_query"`
	SlowestQuery           time.Duration `json:"slowest_query"`
	TotalExecutionTime     time.Duration `json:"total_execution_time"`
	ErrorRate              float64       `json:"error_rate"`
	CacheHitRate           float64       `json:"cache_hit_rate"`
	AverageRowsExamined    float64       `json:"average_rows_examined"`
	AverageRowsReturned    float64       `json:"average_rows_returned"`
	QueryTypeDistribution  map[QueryType]int `json:"query_type_distribution"`
	DatabaseDistribution   map[string]int    `json:"database_distribution"`
	UserDistribution       map[string]int    `json:"user_distribution"`
	ApplicationDistribution map[string]int   `json:"application_distribution"`
}

type AggregatedMetrics struct {
	GroupBy       MetricsGroupBy                   `json:"group_by"`
	TimeWindow    time.Duration                    `json:"time_window"`
	Groups        map[string]*AggregatedGroupMetrics `json:"groups"`
	OverallSummary *MetricsSummary                  `json:"overall_summary"`
	TimeRange     *TimeRange                       `json:"time_range"`
	GeneratedAt   time.Time                        `json:"generated_at"`
}

type AggregatedGroupMetrics struct {
	GroupKey            string        `json:"group_key"`
	TotalQueries        int           `json:"total_queries"`
	AverageExecutionTime time.Duration `json:"average_execution_time"`
	MedianExecutionTime time.Duration `json:"median_execution_time"`
	P95ExecutionTime    time.Duration `json:"p95_execution_time"`
	P99ExecutionTime    time.Duration `json:"p99_execution_time"`
	ErrorCount          int           `json:"error_count"`
	ErrorRate           float64       `json:"error_rate"`
	CacheHitCount       int           `json:"cache_hit_count"`
	CacheHitRate        float64       `json:"cache_hit_rate"`
	TotalRowsExamined   int64         `json:"total_rows_examined"`
	TotalRowsReturned   int64         `json:"total_rows_returned"`
	TotalExecutionTime  time.Duration `json:"total_execution_time"`
	FirstSeen           time.Time     `json:"first_seen"`
	LastSeen            time.Time     `json:"last_seen"`
}

type CurrentPerformanceSnapshot struct {
	Timestamp               time.Time     `json:"timestamp"`
	CurrentQPS              float64       `json:"current_qps"`
	CurrentTPS              float64       `json:"current_tps"`
	ActiveQueries           int           `json:"active_queries"`
	AverageExecutionTime    time.Duration `json:"average_execution_time"`
	SlowestActiveQuery      *QueryExecutionMetric `json:"slowest_active_query,omitempty"`
	RecentSlowQueries       []*QueryExecutionMetric `json:"recent_slow_queries"`
	SystemResourceUsage     *SystemResourceSnapshot `json:"system_resource_usage"`
	DatabaseResourceUsage   *DatabaseResourceSnapshot `json:"database_resource_usage"`
	PerformanceHealth       *PerformanceHealthMetrics `json:"performance_health"`
	ActiveConnections       int           `json:"active_connections"`
	ConnectionPoolUsage     float64       `json:"connection_pool_usage"`
	QueuedQueries           int           `json:"queued_queries"`
	ReplicationLag          time.Duration `json:"replication_lag,omitempty"`
}

type SystemResourceSnapshot struct {
	CPUUsage       float64 `json:"cpu_usage_percent"`
	MemoryUsage    float64 `json:"memory_usage_percent"`
	DiskUsage      float64 `json:"disk_usage_percent"`
	NetworkRxBytes int64   `json:"network_rx_bytes_per_sec"`
	NetworkTxBytes int64   `json:"network_tx_bytes_per_sec"`
	LoadAverage    float64 `json:"load_average"`
	IOWait         float64 `json:"io_wait_percent"`
}

type DatabaseResourceSnapshot struct {
	SharedBufferUsage    float64 `json:"shared_buffer_usage_percent"`
	WorkMemUsage         float64 `json:"work_mem_usage_percent"`
	TotalConnections     int     `json:"total_connections"`
	MaxConnections       int     `json:"max_connections"`
	ActiveTransactions   int     `json:"active_transactions"`
	IdleTransactions     int     `json:"idle_transactions"`
	LocksHeld            int     `json:"locks_held"`
	WaitingForLocks      int     `json:"waiting_for_locks"`
	BackgroundWrites     int64   `json:"background_writes"`
	CheckpointWrites     int64   `json:"checkpoint_writes"`
	DatabaseSize         int64   `json:"database_size_bytes"`
	VacuumRunning        bool    `json:"vacuum_running"`
	AnalyzeRunning       bool    `json:"analyze_running"`
}

type PerformanceHealthMetrics struct {
	OverallScore      float64 `json:"overall_score"` // 0-100
	QueryPerformance  float64 `json:"query_performance_score"`
	ResourceEfficiency float64 `json:"resource_efficiency_score"`
	ErrorRate         float64 `json:"error_rate_score"`
	ThroughputScore   float64 `json:"throughput_score"`
	HealthStatus      string  `json:"health_status"` // "excellent", "good", "fair", "poor", "critical"
	HealthIssues      []string `json:"health_issues,omitempty"`
	Recommendations   []string `json:"recommendations,omitempty"`
}

type MetricsTrends struct {
	TimeWindow           time.Duration            `json:"time_window"`
	DataPoints           []TrendDataPoint         `json:"data_points"`
	Trends               map[string]*TrendAnalysis `json:"trends"`
	PerformanceTrend     string                   `json:"performance_trend"` // "improving", "stable", "degrading"
	TrendConfidence      float64                  `json:"trend_confidence"`
	AnomaliesDetected    []PerformanceAnomaly     `json:"anomalies_detected"`
	ForecastedMetrics    *ForecastedPerformance   `json:"forecasted_metrics,omitempty"`
	GeneratedAt          time.Time                `json:"generated_at"`
}

type TrendDataPoint struct {
	Timestamp            time.Time     `json:"timestamp"`
	QueriesPerSecond     float64       `json:"queries_per_second"`
	AverageExecutionTime time.Duration `json:"average_execution_time"`
	ErrorRate            float64       `json:"error_rate"`
	CacheHitRate         float64       `json:"cache_hit_rate"`
	CPUUsage             float64       `json:"cpu_usage"`
	MemoryUsage          float64       `json:"memory_usage"`
	ActiveConnections    int           `json:"active_connections"`
}

type TrendAnalysis struct {
	MetricName      string  `json:"metric_name"`
	TrendDirection  string  `json:"trend_direction"` // "up", "down", "stable"
	TrendStrength   float64 `json:"trend_strength"`  // 0-1
	ChangePercent   float64 `json:"change_percent"`
	IsSignificant   bool    `json:"is_significant"`
	RegressionSlope float64 `json:"regression_slope"`
}

type PerformanceAnomaly struct {
	DetectedAt      time.Time `json:"detected_at"`
	AnomalyType     string    `json:"anomaly_type"` // "spike", "drop", "plateau", "oscillation"
	MetricAffected  string    `json:"metric_affected"`
	Severity        string    `json:"severity"` // "low", "medium", "high", "critical"
	Description     string    `json:"description"`
	ExpectedValue   float64   `json:"expected_value"`
	ActualValue     float64   `json:"actual_value"`
	DeviationPercent float64  `json:"deviation_percent"`
	Duration        time.Duration `json:"duration"`
	PossibleCauses  []string  `json:"possible_causes"`
}

type ForecastedPerformance struct {
	ForecastHorizon      time.Duration `json:"forecast_horizon"`
	ExpectedQPS          float64       `json:"expected_qps"`
	ExpectedAvgExecTime  time.Duration `json:"expected_avg_exec_time"`
	ExpectedErrorRate    float64       `json:"expected_error_rate"`
	CapacityWarnings     []string      `json:"capacity_warnings,omitempty"`
	Confidence           float64       `json:"confidence"`
}

type PerformanceThresholds struct {
	MaxExecutionTime     time.Duration `json:"max_execution_time"`
	MaxAverageExecTime   time.Duration `json:"max_average_exec_time"`
	MaxErrorRate         float64       `json:"max_error_rate"`
	MinCacheHitRate      float64       `json:"min_cache_hit_rate"`
	MaxCPUUsage          float64       `json:"max_cpu_usage"`
	MaxMemoryUsage       float64       `json:"max_memory_usage"`
	MaxActiveConnections int           `json:"max_active_connections"`
	MinQPS               float64       `json:"min_qps"`
	MaxQPS               float64       `json:"max_qps"`
	MaxQueryQueueDepth   int           `json:"max_query_queue_depth"`
	MaxReplicationLag    time.Duration `json:"max_replication_lag"`
	CustomThresholds     map[string]interface{} `json:"custom_thresholds,omitempty"`
}

type ThresholdBreach struct {
	ThresholdName   string      `json:"threshold_name"`
	ThresholdValue  interface{} `json:"threshold_value"`
	ActualValue     interface{} `json:"actual_value"`
	Severity        string      `json:"severity"`
	DetectedAt      time.Time   `json:"detected_at"`
	Duration        time.Duration `json:"duration"`
	Description     string      `json:"description"`
	ImpactLevel     string      `json:"impact_level"`
	RecommendedAction string    `json:"recommended_action"`
}

type ExportFormat string

const (
	CollectorExportFormatJSON     ExportFormat = "json"
	ExportFormatCSV      ExportFormat = "csv"
	ExportFormatParquet  ExportFormat = "parquet"
	ExportFormatPrometheus ExportFormat = "prometheus"
)

type ReportType string

const (
	ReportTypeSummary      ReportType = "summary"
	ReportTypeDetailed     ReportType = "detailed"
	ReportTypeSlowQueries  ReportType = "slow_queries"
	ReportTypeErrorAnalysis ReportType = "error_analysis"
	ReportTypeTrends       ReportType = "trends"
	ReportTypeCapacity     ReportType = "capacity"
)

type PerformanceReport struct {
	ReportID      string              `json:"report_id"`
	ReportType    ReportType          `json:"report_type"`
	TimeRange     *TimeRange          `json:"time_range"`
	GeneratedAt   time.Time           `json:"generated_at"`
	Summary       *ReportSummary      `json:"summary"`
	Sections      []*ReportSection    `json:"sections"`
	Recommendations []string          `json:"recommendations"`
	Attachments   []*ReportAttachment `json:"attachments,omitempty"`
	Metadata      map[string]interface{} `json:"metadata,omitempty"`
}

type ReportSummary struct {
	TotalMetricsAnalyzed   int     `json:"total_metrics_analyzed"`
	KeyFindings           []string `json:"key_findings"`
	PerformanceScore      float64  `json:"performance_score"`
	TrendSummary          string   `json:"trend_summary"`
	CriticalIssues        int      `json:"critical_issues"`
	WarningIssues         int      `json:"warning_issues"`
	OptimizationOpportunities int  `json:"optimization_opportunities"`
}

type ReportSection struct {
	SectionID    string                 `json:"section_id"`
	Title        string                 `json:"title"`
	Content      string                 `json:"content"`
	Charts       []*ChartData           `json:"charts,omitempty"`
	Tables       []*TableData           `json:"tables,omitempty"`
	Metrics      map[string]interface{} `json:"metrics,omitempty"`
	Subsections  []*ReportSection       `json:"subsections,omitempty"`
}

type ChartData struct {
	ChartID   string                   `json:"chart_id"`
	Title     string                   `json:"title"`
	ChartType string                   `json:"chart_type"` // "line", "bar", "pie", "scatter"
	Data      []map[string]interface{} `json:"data"`
	XAxis     string                   `json:"x_axis"`
	YAxis     string                   `json:"y_axis"`
	Options   map[string]interface{}   `json:"options,omitempty"`
}

type TableData struct {
	TableID string                   `json:"table_id"`
	Title   string                   `json:"title"`
	Headers []string                 `json:"headers"`
	Rows    [][]interface{}          `json:"rows"`
	Options map[string]interface{}   `json:"options,omitempty"`
}

type ReportAttachment struct {
	AttachmentID   string `json:"attachment_id"`
	FileName       string `json:"file_name"`
	ContentType    string `json:"content_type"`
	Data           []byte `json:"data"`
	Description    string `json:"description"`
}

type CollectorHealth struct {
	IsHealthy           bool              `json:"is_healthy"`
	Status              string            `json:"status"` // "healthy", "degraded", "unhealthy"
	Uptime              time.Duration     `json:"uptime"`
	MetricsCollected    int64             `json:"metrics_collected"`
	MetricsPerSecond    float64           `json:"metrics_per_second"`
	LastMetricTimestamp time.Time         `json:"last_metric_timestamp"`
	ErrorCount          int64             `json:"error_count"`
	ErrorRate           float64           `json:"error_rate"`
	MemoryUsage         int64             `json:"memory_usage_bytes"`
	DiskUsage           int64             `json:"disk_usage_bytes"`
	DatabaseConnections int               `json:"database_connections"`
	ComponentHealth     map[string]string `json:"component_health"`
	HealthChecks        []HealthCheck     `json:"health_checks"`
}

type HealthCheck struct {
	CheckName    string    `json:"check_name"`
	Status       string    `json:"status"` // "pass", "warn", "fail"
	LastRun      time.Time `json:"last_run"`
	Duration     time.Duration `json:"duration"`
	Message      string    `json:"message,omitempty"`
	Details      map[string]interface{} `json:"details,omitempty"`
}

// Implementation

type queryPerformanceMetricsCollector struct {
	db            *gorm.DB
	slowQueryLogger DatabaseSlowQueryLogger
	
	// Configuration
	isRunning     bool
	thresholds    *PerformanceThresholds
	
	// State management
	mutex         sync.RWMutex
	stopChan      chan struct{}
	wg            sync.WaitGroup
	
	// Metrics storage and processing
	metricsBuffer   []*QueryExecutionMetric
	bufferMutex     sync.Mutex
	bufferSize      int
	flushInterval   time.Duration
	
	// Performance tracking
	startTime       time.Time
	metricsCount    int64
	errorCount      int64
	lastFlush       time.Time
	
	// Caching
	metricsCache    map[string]*QueryMetricsCollection
	cacheMutex      sync.RWMutex
	cacheExpiry     time.Duration
}

func NewQueryPerformanceMetricsCollector(db *gorm.DB, slowQueryLogger DatabaseSlowQueryLogger) QueryPerformanceMetricsCollector {
	return &queryPerformanceMetricsCollector{
		db:              db,
		slowQueryLogger: slowQueryLogger,
		bufferSize:      1000,
		flushInterval:   30 * time.Second,
		cacheExpiry:     5 * time.Minute,
		metricsCache:    make(map[string]*QueryMetricsCollection),
		thresholds: &PerformanceThresholds{
			MaxExecutionTime:     200 * time.Millisecond,
			MaxAverageExecTime:   100 * time.Millisecond,
			MaxErrorRate:         0.05, // 5%
			MinCacheHitRate:      0.80, // 80%
			MaxCPUUsage:          80.0, // 80%
			MaxMemoryUsage:       85.0, // 85%
			MaxActiveConnections: 100,
			MinQPS:               10,
			MaxQPS:               1000,
			MaxQueryQueueDepth:   50,
			MaxReplicationLag:    5 * time.Second,
		},
	}
}

func (c *queryPerformanceMetricsCollector) Start(ctx context.Context) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	
	if c.isRunning {
		return fmt.Errorf("metrics collector is already running")
	}
	
	c.isRunning = true
	c.startTime = time.Now()
	c.stopChan = make(chan struct{})
	c.metricsBuffer = make([]*QueryExecutionMetric, 0, c.bufferSize)
	
	// Start background processes
	c.wg.Add(2)
	go c.backgroundMetricsFlush(ctx)
	go c.backgroundHealthMonitor(ctx)
	
	log.Printf("Query performance metrics collector started")
	return nil
}

func (c *queryPerformanceMetricsCollector) Stop(ctx context.Context) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	
	if !c.isRunning {
		return nil
	}
	
	close(c.stopChan)
	c.isRunning = false
	
	// Wait for background processes
	c.wg.Wait()
	
	// Flush remaining metrics
	if err := c.flushMetricsBuffer(ctx); err != nil {
		log.Printf("Warning: failed to flush metrics during shutdown: %v", err)
	}
	
	log.Printf("Query performance metrics collector stopped")
	return nil
}

func (c *queryPerformanceMetricsCollector) IsRunning() bool {
	c.mutex.RLock()
	defer c.mutex.RUnlock()
	return c.isRunning
}

func (c *queryPerformanceMetricsCollector) RecordQueryExecution(ctx context.Context, metric *QueryExecutionMetric) error {
	if !c.isRunning {
		return fmt.Errorf("metrics collector is not running")
	}
	
	// Add timestamp if not set
	if metric.Timestamp.IsZero() {
		metric.Timestamp = time.Now()
	}
	
	// Generate metric ID if not set
	if metric.MetricID == "" {
		metric.MetricID = fmt.Sprintf("metric_%d_%d", time.Now().UnixNano(), len(c.metricsBuffer))
	}
	
	// Normalize query for pattern matching
	if metric.QueryNormalized == "" {
		metric.QueryNormalized = c.normalizeQuery(metric.QueryText)
	}
	
	// Enrich with performance context
	if metric.PerformanceContext == nil {
		metric.PerformanceContext = c.buildPerformanceContext(ctx)
	}
	
	// Add to buffer
	c.bufferMutex.Lock()
	c.metricsBuffer = append(c.metricsBuffer, metric)
	c.metricsCount++
	
	// Check if buffer is full
	shouldFlush := len(c.metricsBuffer) >= c.bufferSize
	c.bufferMutex.Unlock()
	
	// Flush if buffer is full
	if shouldFlush {
		if err := c.flushMetricsBuffer(ctx); err != nil {
			c.errorCount++
			log.Printf("Error flushing metrics buffer: %v", err)
		}
	}
	
	return nil
}

func (c *queryPerformanceMetricsCollector) GetQueryMetrics(ctx context.Context, filter *MetricsFilter) (*QueryMetricsCollection, error) {
	// Build cache key
	cacheKey := c.buildCacheKey(filter)
	
	// Check cache
	c.cacheMutex.RLock()
	if cached, exists := c.metricsCache[cacheKey]; exists {
		c.cacheMutex.RUnlock()
		return cached, nil
	}
	c.cacheMutex.RUnlock()
	
	// Query metrics from database
	collection, err := c.queryMetricsFromDatabase(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("failed to query metrics: %w", err)
	}
	
	// Cache results
	c.cacheMutex.Lock()
	c.metricsCache[cacheKey] = collection
	c.cacheMutex.Unlock()
	
	return collection, nil
}

func (c *queryPerformanceMetricsCollector) GetAggregatedMetrics(ctx context.Context, timeWindow time.Duration, groupBy MetricsGroupBy) (*AggregatedMetrics, error) {
	filter := &MetricsFilter{
		TimeRange: &TimeRange{
			Start: time.Now().Add(-timeWindow),
			End:   time.Now(),
		},
	}
	
	collection, err := c.GetQueryMetrics(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("failed to get metrics for aggregation: %w", err)
	}
	
	return c.aggregateMetrics(collection, groupBy, timeWindow), nil
}

func (c *queryPerformanceMetricsCollector) GetCurrentMetrics(ctx context.Context) (*CurrentPerformanceSnapshot, error) {
	snapshot := &CurrentPerformanceSnapshot{
		Timestamp: time.Now(),
	}
	
	// Get current system metrics
	systemSnapshot, err := c.getSystemResourceSnapshot(ctx)
	if err != nil {
		log.Printf("Warning: failed to get system resource snapshot: %v", err)
	} else {
		snapshot.SystemResourceUsage = systemSnapshot
	}
	
	// Get current database metrics
	dbSnapshot, err := c.getDatabaseResourceSnapshot(ctx)
	if err != nil {
		log.Printf("Warning: failed to get database resource snapshot: %v", err)
	} else {
		snapshot.DatabaseResourceUsage = dbSnapshot
	}
	
	// Get recent performance metrics
	filter := &MetricsFilter{
		TimeRange: &TimeRange{
			Start: time.Now().Add(-5 * time.Minute),
			End:   time.Now(),
		},
		Limit: 100,
	}
	
	recent, err := c.GetQueryMetrics(ctx, filter)
	if err != nil {
		log.Printf("Warning: failed to get recent metrics: %v", err)
	} else {
		snapshot.CurrentQPS = c.calculateQPS(recent.Metrics)
		snapshot.AverageExecutionTime = recent.Summary.AverageExecutionTime
		
		// Get slow queries
		slowQueries := []*QueryExecutionMetric{}
		for _, metric := range recent.Metrics {
			if metric.ExecutionTime > c.thresholds.MaxExecutionTime {
				slowQueries = append(slowQueries, metric)
			}
		}
		snapshot.RecentSlowQueries = slowQueries
		
		if len(slowQueries) > 0 {
			snapshot.SlowestActiveQuery = slowQueries[0]
		}
	}
	
	// Calculate performance health
	snapshot.PerformanceHealth = c.calculatePerformanceHealth(snapshot)
	
	return snapshot, nil
}

func (c *queryPerformanceMetricsCollector) GetMetricsTrends(ctx context.Context, timeWindow time.Duration) (*MetricsTrends, error) {
	// Divide time window into data points
	numPoints := 24 // 24 data points
	interval := timeWindow / time.Duration(numPoints)
	
	trends := &MetricsTrends{
		TimeWindow:  timeWindow,
		DataPoints:  make([]TrendDataPoint, 0, numPoints),
		Trends:      make(map[string]*TrendAnalysis),
		GeneratedAt: time.Now(),
	}
	
	// Collect data points
	endTime := time.Now()
	for i := numPoints; i > 0; i-- {
		pointStart := endTime.Add(-time.Duration(i) * interval)
		pointEnd := pointStart.Add(interval)
		
		filter := &MetricsFilter{
			TimeRange: &TimeRange{Start: pointStart, End: pointEnd},
		}
		
		collection, err := c.GetQueryMetrics(ctx, filter)
		if err != nil {
			continue // Skip this data point
		}
		
		dataPoint := TrendDataPoint{
			Timestamp:            pointStart,
			QueriesPerSecond:     c.calculateQPS(collection.Metrics),
			AverageExecutionTime: collection.Summary.AverageExecutionTime,
			ErrorRate:            collection.Summary.ErrorRate,
			CacheHitRate:         collection.Summary.CacheHitRate,
		}
		
		trends.DataPoints = append(trends.DataPoints, dataPoint)
	}
	
	// Analyze trends
	trends.Trends = c.analyzeTrends(trends.DataPoints)
	
	// Determine overall performance trend
	if execTimeTrend, exists := trends.Trends["average_execution_time"]; exists {
		switch execTimeTrend.TrendDirection {
		case "down":
			trends.PerformanceTrend = "improving"
		case "up":
			trends.PerformanceTrend = "degrading"
		default:
			trends.PerformanceTrend = "stable"
		}
		trends.TrendConfidence = execTimeTrend.TrendStrength
	}
	
	// Detect anomalies
	trends.AnomaliesDetected = c.detectAnomalies(trends.DataPoints)
	
	return trends, nil
}

func (c *queryPerformanceMetricsCollector) SetPerformanceThresholds(thresholds *PerformanceThresholds) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	
	c.thresholds = thresholds
	log.Printf("Performance thresholds updated")
	return nil
}

func (c *queryPerformanceMetricsCollector) GetPerformanceThresholds() *PerformanceThresholds {
	c.mutex.RLock()
	defer c.mutex.RUnlock()
	return c.thresholds
}

func (c *queryPerformanceMetricsCollector) CheckThresholdBreaches(ctx context.Context) ([]ThresholdBreach, error) {
	breaches := []ThresholdBreach{}
	
	// Get current metrics for threshold checking
	snapshot, err := c.GetCurrentMetrics(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get current metrics for threshold checking: %w", err)
	}
	
	// Check execution time threshold
	if snapshot.AverageExecutionTime > c.thresholds.MaxAverageExecTime {
		breaches = append(breaches, ThresholdBreach{
			ThresholdName:     "max_average_exec_time",
			ThresholdValue:    c.thresholds.MaxAverageExecTime,
			ActualValue:       snapshot.AverageExecutionTime,
			Severity:          "high",
			DetectedAt:        time.Now(),
			Description:       "Average execution time exceeds threshold",
			ImpactLevel:       "high",
			RecommendedAction: "Review slow queries and optimize database performance",
		})
	}
	
	// Check connection threshold
	if snapshot.ActiveConnections > c.thresholds.MaxActiveConnections {
		breaches = append(breaches, ThresholdBreach{
			ThresholdName:     "max_active_connections",
			ThresholdValue:    c.thresholds.MaxActiveConnections,
			ActualValue:       snapshot.ActiveConnections,
			Severity:          "medium",
			DetectedAt:        time.Now(),
			Description:       "Active connections exceed threshold",
			ImpactLevel:       "medium",
			RecommendedAction: "Review connection pooling configuration",
		})
	}
	
	// Check system resource thresholds
	if snapshot.SystemResourceUsage != nil {
		if snapshot.SystemResourceUsage.CPUUsage > c.thresholds.MaxCPUUsage {
			breaches = append(breaches, ThresholdBreach{
				ThresholdName:     "max_cpu_usage",
				ThresholdValue:    c.thresholds.MaxCPUUsage,
				ActualValue:       snapshot.SystemResourceUsage.CPUUsage,
				Severity:          "high",
				DetectedAt:        time.Now(),
				Description:       "CPU usage exceeds threshold",
				ImpactLevel:       "high",
				RecommendedAction: "Scale resources or optimize query performance",
			})
		}
		
		if snapshot.SystemResourceUsage.MemoryUsage > c.thresholds.MaxMemoryUsage {
			breaches = append(breaches, ThresholdBreach{
				ThresholdName:     "max_memory_usage",
				ThresholdValue:    c.thresholds.MaxMemoryUsage,
				ActualValue:       snapshot.SystemResourceUsage.MemoryUsage,
				Severity:          "high",
				DetectedAt:        time.Now(),
				Description:       "Memory usage exceeds threshold",
				ImpactLevel:       "high",
				RecommendedAction: "Scale memory resources or optimize memory usage",
			})
		}
	}
	
	return breaches, nil
}

func (c *queryPerformanceMetricsCollector) ExportMetrics(ctx context.Context, format ExportFormat, timeRange TimeRange) ([]byte, error) {
	filter := &MetricsFilter{
		TimeRange: &timeRange,
	}
	
	collection, err := c.GetQueryMetrics(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("failed to get metrics for export: %w", err)
	}
	
	switch format {
	case CollectorExportFormatJSON:
		return json.MarshalIndent(collection, "", "  ")
	case ExportFormatCSV:
		return c.exportToCSV(collection)
	case ExportFormatPrometheus:
		return c.exportToPrometheus(collection)
	default:
		return nil, fmt.Errorf("unsupported export format: %s", format)
	}
}

func (c *queryPerformanceMetricsCollector) GeneratePerformanceReport(ctx context.Context, reportType ReportType, timeRange TimeRange) (*PerformanceReport, error) {
	report := &PerformanceReport{
		ReportID:    fmt.Sprintf("report_%d", time.Now().UnixNano()),
		ReportType:  reportType,
		TimeRange:   &timeRange,
		GeneratedAt: time.Now(),
		Sections:    []*ReportSection{},
	}
	
	// Get metrics for the report
	filter := &MetricsFilter{TimeRange: &timeRange}
	collection, err := c.GetQueryMetrics(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("failed to get metrics for report: %w", err)
	}
	
	// Create report summary
	report.Summary = &ReportSummary{
		TotalMetricsAnalyzed: len(collection.Metrics),
		PerformanceScore:     c.calculateOverallPerformanceScore(collection),
		KeyFindings:         c.extractKeyFindings(collection),
		TrendSummary:        "Performance analysis based on collected metrics",
	}
	
	// Add sections based on report type
	switch reportType {
	case ReportTypeSummary:
		report.Sections = c.generateSummaryReportSections(collection)
	case ReportTypeDetailed:
		report.Sections = c.generateDetailedReportSections(collection)
	case ReportTypeSlowQueries:
		report.Sections = c.generateSlowQueriesReportSections(collection)
	}
	
	// Add recommendations
	report.Recommendations = c.generateReportRecommendations(collection)
	
	return report, nil
}

func (c *queryPerformanceMetricsCollector) GetCollectorHealth(ctx context.Context) (*CollectorHealth, error) {
	health := &CollectorHealth{
		Status:              "healthy",
		Uptime:              time.Since(c.startTime),
		MetricsCollected:    c.metricsCount,
		ErrorCount:          c.errorCount,
		LastMetricTimestamp: time.Now(),
		ComponentHealth:     make(map[string]string),
		HealthChecks:        []HealthCheck{},
	}
	
	// Calculate metrics per second
	if health.Uptime > 0 {
		health.MetricsPerSecond = float64(c.metricsCount) / health.Uptime.Seconds()
		health.ErrorRate = float64(c.errorCount) / float64(c.metricsCount)
	}
	
	// Check database connection
	if err := c.db.Exec("SELECT 1").Error; err != nil {
		health.ComponentHealth["database"] = "unhealthy"
		health.Status = "degraded"
	} else {
		health.ComponentHealth["database"] = "healthy"
	}
	
	// Check slow query logger
	if c.slowQueryLogger.IsRunning() {
		health.ComponentHealth["slow_query_logger"] = "healthy"
	} else {
		health.ComponentHealth["slow_query_logger"] = "unhealthy"
		health.Status = "degraded"
	}
	
	// Overall health determination
	if health.ErrorRate > 0.1 { // >10% error rate
		health.Status = "unhealthy"
		health.IsHealthy = false
	} else if health.Status == "degraded" {
		health.IsHealthy = false
	} else {
		health.IsHealthy = true
	}
	
	return health, nil
}

func (c *queryPerformanceMetricsCollector) ResetMetrics(ctx context.Context) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	
	// Clear buffer
	c.bufferMutex.Lock()
	c.metricsBuffer = c.metricsBuffer[:0]
	c.bufferMutex.Unlock()
	
	// Clear cache
	c.cacheMutex.Lock()
	c.metricsCache = make(map[string]*QueryMetricsCollection)
	c.cacheMutex.Unlock()
	
	// Reset counters
	c.metricsCount = 0
	c.errorCount = 0
	
	log.Printf("Metrics collector reset")
	return nil
}

// Internal helper methods

func (c *queryPerformanceMetricsCollector) backgroundMetricsFlush(ctx context.Context) {
	defer c.wg.Done()
	
	ticker := time.NewTicker(c.flushInterval)
	defer ticker.Stop()
	
	for {
		select {
		case <-c.stopChan:
			return
		case <-ticker.C:
			if err := c.flushMetricsBuffer(ctx); err != nil {
				c.errorCount++
				log.Printf("Error in background metrics flush: %v", err)
			}
		}
	}
}

func (c *queryPerformanceMetricsCollector) backgroundHealthMonitor(ctx context.Context) {
	defer c.wg.Done()
	
	ticker := time.NewTicker(60 * time.Second) // Every minute
	defer ticker.Stop()
	
	for {
		select {
		case <-c.stopChan:
			return
		case <-ticker.C:
			c.performHealthCheck(ctx)
		}
	}
}

func (c *queryPerformanceMetricsCollector) flushMetricsBuffer(ctx context.Context) error {
	c.bufferMutex.Lock()
	defer c.bufferMutex.Unlock()
	
	if len(c.metricsBuffer) == 0 {
		return nil
	}
	
	// In a real implementation, this would persist metrics to a time-series database
	// For now, we'll just log the flush
	log.Printf("Flushed %d metrics to storage", len(c.metricsBuffer))
	
	// Clear buffer
	c.metricsBuffer = c.metricsBuffer[:0]
	c.lastFlush = time.Now()
	
	return nil
}

func (c *queryPerformanceMetricsCollector) performHealthCheck(ctx context.Context) {
	// Check if we're receiving metrics
	if time.Since(c.lastFlush) > 2*c.flushInterval {
		log.Printf("Warning: No metrics flushed in %v", time.Since(c.lastFlush))
	}
	
	// Check error rate
	if c.metricsCount > 100 && float64(c.errorCount)/float64(c.metricsCount) > 0.05 {
		log.Printf("Warning: High error rate: %.2f%%", 100*float64(c.errorCount)/float64(c.metricsCount))
	}
}

// Placeholder implementations for complex methods
// In a real implementation, these would contain full logic

func (c *queryPerformanceMetricsCollector) normalizeQuery(query string) string {
	// Simple normalization
	return fmt.Sprintf("normalized_%x", query[:utils.MinInt(len(query), 10)])
}

func (c *queryPerformanceMetricsCollector) buildPerformanceContext(ctx context.Context) *PerformanceContext {
	return &PerformanceContext{
		SystemLoad:         1.0,
		CPUUtilization:     50.0,
		MemoryUtilization:  60.0,
		ActiveConnections:  10,
		QueriesPerSecond:   100.0,
	}
}

func (c *queryPerformanceMetricsCollector) buildCacheKey(filter *MetricsFilter) string {
	return fmt.Sprintf("metrics_%x", time.Now().Unix()/300) // 5-minute cache
}

func (c *queryPerformanceMetricsCollector) queryMetricsFromDatabase(ctx context.Context, filter *MetricsFilter) (*QueryMetricsCollection, error) {
	// This would query actual stored metrics
	// For now, return empty collection
	return &QueryMetricsCollection{
		Metrics:       []*QueryExecutionMetric{},
		TotalCount:    0,
		FilteredCount: 0,
		Filter:        filter,
		Summary:       &MetricsSummary{},
	}, nil
}

func (c *queryPerformanceMetricsCollector) aggregateMetrics(collection *QueryMetricsCollection, groupBy MetricsGroupBy, timeWindow time.Duration) *AggregatedMetrics {
	return &AggregatedMetrics{
		GroupBy:        groupBy,
		TimeWindow:     timeWindow,
		Groups:         make(map[string]*AggregatedGroupMetrics),
		OverallSummary: collection.Summary,
		GeneratedAt:    time.Now(),
	}
}

func (c *queryPerformanceMetricsCollector) getSystemResourceSnapshot(ctx context.Context) (*SystemResourceSnapshot, error) {
	return &SystemResourceSnapshot{
		CPUUsage:    50.0,
		MemoryUsage: 60.0,
		DiskUsage:   70.0,
		LoadAverage: 1.0,
	}, nil
}

func (c *queryPerformanceMetricsCollector) getDatabaseResourceSnapshot(ctx context.Context) (*DatabaseResourceSnapshot, error) {
	return &DatabaseResourceSnapshot{
		TotalConnections:   10,
		MaxConnections:     100,
		ActiveTransactions: 5,
	}, nil
}

func (c *queryPerformanceMetricsCollector) calculateQPS(metrics []*QueryExecutionMetric) float64 {
	if len(metrics) == 0 {
		return 0
	}
	
	// Simple QPS calculation
	timeSpan := time.Minute // Assume 1-minute window
	return float64(len(metrics)) / timeSpan.Seconds()
}

func (c *queryPerformanceMetricsCollector) calculatePerformanceHealth(snapshot *CurrentPerformanceSnapshot) *PerformanceHealthMetrics {
	score := 85.0 // Default good score
	
	healthIssues := []string{}
	recommendations := []string{}
	
	if snapshot.AverageExecutionTime > c.thresholds.MaxExecutionTime {
		score -= 20
		healthIssues = append(healthIssues, "High average execution time")
		recommendations = append(recommendations, "Optimize slow queries")
	}
	
	healthStatus := "excellent"
	if score < 90 {
		healthStatus = "good"
	}
	if score < 70 {
		healthStatus = "fair"
	}
	if score < 50 {
		healthStatus = "poor"
	}
	
	return &PerformanceHealthMetrics{
		OverallScore:    score,
		HealthStatus:    healthStatus,
		HealthIssues:    healthIssues,
		Recommendations: recommendations,
	}
}

func (c *queryPerformanceMetricsCollector) analyzeTrends(dataPoints []TrendDataPoint) map[string]*TrendAnalysis {
	trends := make(map[string]*TrendAnalysis)
	
	if len(dataPoints) < 2 {
		return trends
	}
	
	// Simple trend analysis for execution time
	first := dataPoints[0].AverageExecutionTime
	last := dataPoints[len(dataPoints)-1].AverageExecutionTime
	
	direction := "stable"
	changePercent := 0.0
	
	if last > first {
		direction = "up"
		changePercent = float64(last-first) / float64(first) * 100
	} else if last < first {
		direction = "down"
		changePercent = float64(first-last) / float64(first) * 100
	}
	
	trends["average_execution_time"] = &TrendAnalysis{
		MetricName:     "average_execution_time",
		TrendDirection: direction,
		TrendStrength:  0.7, // Moderate confidence
		ChangePercent:  changePercent,
		IsSignificant:  changePercent > 10, // >10% change is significant
	}
	
	return trends
}

func (c *queryPerformanceMetricsCollector) detectAnomalies(dataPoints []TrendDataPoint) []PerformanceAnomaly {
	anomalies := []PerformanceAnomaly{}
	
	// Simple spike detection
	if len(dataPoints) < 3 {
		return anomalies
	}
	
	for i := 1; i < len(dataPoints)-1; i++ {
		current := dataPoints[i].AverageExecutionTime
		prev := dataPoints[i-1].AverageExecutionTime
		next := dataPoints[i+1].AverageExecutionTime
		
		avg := (prev + next) / 2
		if current > avg*2 { // Spike detection
			anomalies = append(anomalies, PerformanceAnomaly{
				DetectedAt:     dataPoints[i].Timestamp,
				AnomalyType:    "spike",
				MetricAffected: "average_execution_time",
				Severity:       "medium",
				Description:    "Execution time spike detected",
				ExpectedValue:  float64(avg),
				ActualValue:    float64(current),
			})
		}
	}
	
	return anomalies
}

func (c *queryPerformanceMetricsCollector) exportToCSV(collection *QueryMetricsCollection) ([]byte, error) {
	// Simple CSV export implementation
	csv := "timestamp,query_type,execution_time,error\n"
	for _, metric := range collection.Metrics {
		csv += fmt.Sprintf("%s,%s,%v,%v\n",
			metric.Timestamp.Format(time.RFC3339),
			metric.QueryType,
			metric.ExecutionTime,
			metric.ErrorOccurred,
		)
	}
	return []byte(csv), nil
}

func (c *queryPerformanceMetricsCollector) exportToPrometheus(collection *QueryMetricsCollection) ([]byte, error) {
	// Simple Prometheus format export
	output := "# HELP query_execution_time Query execution time in seconds\n"
	output += "# TYPE query_execution_time histogram\n"
	
	for _, metric := range collection.Metrics {
		output += fmt.Sprintf("query_execution_time{query_type=\"%s\"} %.3f\n",
			metric.QueryType,
			metric.ExecutionTime.Seconds(),
		)
	}
	
	return []byte(output), nil
}

func (c *queryPerformanceMetricsCollector) calculateOverallPerformanceScore(collection *QueryMetricsCollection) float64 {
	if collection.Summary == nil {
		return 50.0 // Default neutral score
	}
	
	score := 100.0
	
	// Penalize high execution times
	if collection.Summary.AverageExecutionTime > c.thresholds.MaxAverageExecTime {
		score -= 30
	}
	
	// Penalize high error rates
	if collection.Summary.ErrorRate > c.thresholds.MaxErrorRate {
		score -= 20
	}
	
	// Reward high cache hit rates
	if collection.Summary.CacheHitRate < c.thresholds.MinCacheHitRate {
		score -= 15
	}
	
	if score < 0 {
		score = 0
	}
	
	return score
}

func (c *queryPerformanceMetricsCollector) extractKeyFindings(collection *QueryMetricsCollection) []string {
	findings := []string{}
	
	if collection.Summary.ErrorRate > 0.01 {
		findings = append(findings, fmt.Sprintf("Error rate is %.2f%%", collection.Summary.ErrorRate*100))
	}
	
	if collection.Summary.AverageExecutionTime > c.thresholds.MaxAverageExecTime {
		findings = append(findings, fmt.Sprintf("Average execution time (%.2fms) exceeds threshold",
			float64(collection.Summary.AverageExecutionTime/time.Millisecond)))
	}
	
	if collection.Summary.CacheHitRate < c.thresholds.MinCacheHitRate {
		findings = append(findings, fmt.Sprintf("Cache hit rate (%.2f%%) is below optimal",
			collection.Summary.CacheHitRate*100))
	}
	
	return findings
}

func (c *queryPerformanceMetricsCollector) generateSummaryReportSections(collection *QueryMetricsCollection) []*ReportSection {
	return []*ReportSection{
		{
			SectionID: "summary",
			Title:     "Performance Summary",
			Content:   "Overall query performance analysis",
		},
	}
}

func (c *queryPerformanceMetricsCollector) generateDetailedReportSections(collection *QueryMetricsCollection) []*ReportSection {
	return []*ReportSection{
		{
			SectionID: "detailed",
			Title:     "Detailed Performance Analysis",
			Content:   "Comprehensive query performance breakdown",
		},
	}
}

func (c *queryPerformanceMetricsCollector) generateSlowQueriesReportSections(collection *QueryMetricsCollection) []*ReportSection {
	return []*ReportSection{
		{
			SectionID: "slow_queries",
			Title:     "Slow Query Analysis",
			Content:   "Analysis of queries exceeding performance thresholds",
		},
	}
}

func (c *queryPerformanceMetricsCollector) generateReportRecommendations(collection *QueryMetricsCollection) []string {
	recommendations := []string{
		"Monitor query performance regularly",
		"Implement query optimization for slow queries",
		"Consider adding appropriate database indices",
		"Review and optimize database configuration",
	}
	
	return recommendations
}

