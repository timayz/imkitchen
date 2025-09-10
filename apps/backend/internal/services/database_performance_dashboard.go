package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sort"
	"sync"
	"time"

	"gorm.io/gorm"
)

type DatabasePerformanceDashboard interface {
	// Dashboard lifecycle
	Initialize(ctx context.Context) error
	GetDashboardConfig() (*DashboardConfiguration, error)
	UpdateDashboardConfig(config *DashboardConfiguration) error
	
	// Dashboard data retrieval
	GetDashboardData(ctx context.Context, dashboardID string, timeRange *TimeRange, options *DashboardOptions) (*DashboardData, error)
	GetDashboardWidgets(ctx context.Context, dashboardID string) ([]*DashboardWidget, error)
	RefreshDashboard(ctx context.Context, dashboardID string) (*DashboardRefreshResult, error)
	
	// Real-time dashboard updates
	GetRealTimeData(ctx context.Context, widgetID string) (*RealTimeWidgetData, error)
	SubscribeToRealTimeUpdates(ctx context.Context, dashboardID string, callback RealTimeCallback) error
	UnsubscribeFromRealTimeUpdates(dashboardID string) error
	
	// Widget management
	CreateWidget(ctx context.Context, widget *DashboardWidget) error
	UpdateWidget(ctx context.Context, widgetID string, widget *DashboardWidget) error
	DeleteWidget(ctx context.Context, widgetID string) error
	CloneWidget(ctx context.Context, widgetID string, newWidgetID string) (*DashboardWidget, error)
	
	// Dashboard templates and presets
	GetDashboardTemplates() ([]*DashboardTemplate, error)
	CreateDashboardFromTemplate(ctx context.Context, templateID string, customization *DashboardCustomization) (*DashboardData, error)
	SaveDashboardAsTemplate(ctx context.Context, dashboardID string, templateName string) (*DashboardTemplate, error)
	
	// Export and sharing
	ExportDashboard(ctx context.Context, dashboardID string, format DashboardExportFormat) ([]byte, error)
	GetDashboardShareLink(ctx context.Context, dashboardID string, shareOptions *ShareOptions) (*ShareLink, error)
	ImportDashboard(ctx context.Context, data []byte, format DashboardExportFormat) (*DashboardData, error)
	
	// Analytics and insights
	GetDashboardUsageAnalytics(ctx context.Context, dashboardID string, timeRange *TimeRange) (*DashboardAnalytics, error)
	GetPerformanceInsights(ctx context.Context, timeRange *TimeRange) (*PerformanceInsights, error)
	GenerateDashboardReport(ctx context.Context, dashboardID string, reportOptions *DashboardReportOptions) (*DashboardReport, error)
}

type DashboardConfiguration struct {
	DashboardID     string                 `json:"dashboard_id"`
	Name            string                 `json:"name"`
	Description     string                 `json:"description"`
	Theme           string                 `json:"theme"` // "light", "dark", "auto"
	Layout          *DashboardLayout       `json:"layout"`
	RefreshInterval time.Duration          `json:"refresh_interval"`
	AutoRefresh     bool                   `json:"auto_refresh"`
	TimeZone        string                 `json:"time_zone"`
	DefaultTimeRange *TimeRange            `json:"default_time_range"`
	Widgets         []*DashboardWidget     `json:"widgets"`
	Permissions     *DashboardPermissions  `json:"permissions"`
	Customization   *DashboardCustomization `json:"customization"`
	CreatedAt       time.Time              `json:"created_at"`
	UpdatedAt       time.Time              `json:"updated_at"`
	CreatedBy       string                 `json:"created_by"`
	Tags            []string               `json:"tags"`
	IsPublic        bool                   `json:"is_public"`
	Version         string                 `json:"version"`
}

type DashboardLayout struct {
	Columns     int                    `json:"columns"`
	Rows        int                    `json:"rows"`
	GridSize    *GridSize              `json:"grid_size"`
	Responsive  bool                   `json:"responsive"`
	Breakpoints map[string]*GridSize   `json:"breakpoints"`
	Spacing     int                    `json:"spacing"`
	Margins     *Margins               `json:"margins"`
}

type GridSize struct {
	Width  int `json:"width"`
	Height int `json:"height"`
}

type Margins struct {
	Top    int `json:"top"`
	Right  int `json:"right"`
	Bottom int `json:"bottom"`
	Left   int `json:"left"`
}

type DashboardWidget struct {
	WidgetID      string                 `json:"widget_id"`
	Title         string                 `json:"title"`
	Type          WidgetType             `json:"type"`
	Position      *WidgetPosition        `json:"position"`
	Size          *WidgetSize            `json:"size"`
	Configuration *WidgetConfiguration   `json:"configuration"`
	DataSource    *WidgetDataSource      `json:"data_source"`
	Styling       *WidgetStyling         `json:"styling"`
	Options       *WidgetOptions         `json:"options"`
	Filters       []*WidgetFilter        `json:"filters,omitempty"`
	Thresholds    []*WidgetThreshold     `json:"thresholds,omitempty"`
	CreatedAt     time.Time              `json:"created_at"`
	UpdatedAt     time.Time              `json:"updated_at"`
	IsVisible     bool                   `json:"is_visible"`
	Order         int                    `json:"order"`
}

type WidgetType string

const (
	WidgetTypeChart           WidgetType = "chart"
	WidgetTypeGauge          WidgetType = "gauge"
	WidgetTypeTable          WidgetType = "table"
	WidgetTypeCounter        WidgetType = "counter"
	WidgetTypeProgress       WidgetType = "progress"
	WidgetTypeHeatmap        WidgetType = "heatmap"
	WidgetTypeText           WidgetType = "text"
	WidgetTypeAlert          WidgetType = "alert"
	WidgetTypeTimeline       WidgetType = "timeline"
	WidgetTypeTopList        WidgetType = "top_list"
	WidgetTypeDistribution   WidgetType = "distribution"
	WidgetTypeComparison     WidgetType = "comparison"
	WidgetTypeTrend          WidgetType = "trend"
	WidgetTypeHealthCheck    WidgetType = "health_check"
	WidgetTypeCustom         WidgetType = "custom"
)

type WidgetPosition struct {
	X int `json:"x"`
	Y int `json:"y"`
}

type WidgetSize struct {
	Width  int `json:"width"`
	Height int `json:"height"`
}

type WidgetConfiguration struct {
	ChartType       string                 `json:"chart_type,omitempty"`       // "line", "bar", "pie", "area"
	AggregationType string                 `json:"aggregation_type,omitempty"` // "avg", "sum", "count", "max", "min"
	TimeGrouping    string                 `json:"time_grouping,omitempty"`    // "1m", "5m", "15m", "1h", "1d"
	MetricField     string                 `json:"metric_field,omitempty"`
	GroupByField    string                 `json:"group_by_field,omitempty"`
	SortBy          string                 `json:"sort_by,omitempty"`
	SortOrder       string                 `json:"sort_order,omitempty"`        // "asc", "desc"
	Limit           int                    `json:"limit,omitempty"`
	ShowLegend      bool                   `json:"show_legend"`
	ShowTooltip     bool                   `json:"show_tooltip"`
	ShowGrid        bool                   `json:"show_grid"`
	ShowAxes        bool                   `json:"show_axes"`
	LogScale        bool                   `json:"log_scale"`
	StepsLine       bool                   `json:"steps_line"`
	FillArea        bool                   `json:"fill_area"`
	CustomQuery     string                 `json:"custom_query,omitempty"`
	RefreshRate     time.Duration          `json:"refresh_rate"`
	CacheEnabled    bool                   `json:"cache_enabled"`
	CustomOptions   map[string]interface{} `json:"custom_options,omitempty"`
}

type WidgetDataSource struct {
	SourceType   DataSourceType         `json:"source_type"`
	Query        string                 `json:"query,omitempty"`
	Metrics      []string               `json:"metrics,omitempty"`
	Filters      map[string]interface{} `json:"filters,omitempty"`
	TimeField    string                 `json:"time_field,omitempty"`
	ValueFields  []string               `json:"value_fields,omitempty"`
	GroupFields  []string               `json:"group_fields,omitempty"`
	Aggregations map[string]string      `json:"aggregations,omitempty"`
	Parameters   map[string]interface{} `json:"parameters,omitempty"`
}

type DataSourceType string

const (
	DataSourceMetrics     DataSourceType = "metrics"
	DataSourceSlowQueries DataSourceType = "slow_queries"
	DataSourceDatabase    DataSourceType = "database"
	DataSourceSystem      DataSourceType = "system"
	DataSourceCustom      DataSourceType = "custom"
)

type WidgetStyling struct {
	ColorScheme   string            `json:"color_scheme"`   // "default", "blue", "green", "red", "custom"
	Colors        []string          `json:"colors,omitempty"`
	BackgroundColor string          `json:"background_color,omitempty"`
	BorderColor   string            `json:"border_color,omitempty"`
	TextColor     string            `json:"text_color,omitempty"`
	FontSize      int               `json:"font_size,omitempty"`
	FontWeight    string            `json:"font_weight,omitempty"`
	BorderWidth   int               `json:"border_width,omitempty"`
	BorderRadius  int               `json:"border_radius,omitempty"`
	Padding       int               `json:"padding,omitempty"`
	Margin        int               `json:"margin,omitempty"`
	CustomCSS     map[string]string `json:"custom_css,omitempty"`
}

type WidgetOptions struct {
	ShowTitle       bool                   `json:"show_title"`
	ShowDescription bool                   `json:"show_description"`
	ShowTimestamp   bool                   `json:"show_timestamp"`
	ShowUnits       bool                   `json:"show_units"`
	DecimalPlaces   int                    `json:"decimal_places"`
	UnitFormat      string                 `json:"unit_format"`
	DateFormat      string                 `json:"date_format"`
	TimeFormat      string                 `json:"time_format"`
	Interactive     bool                   `json:"interactive"`
	Exportable      bool                   `json:"exportable"`
	Drilldown       bool                   `json:"drilldown"`
	Animation       bool                   `json:"animation"`
	Responsive      bool                   `json:"responsive"`
	CustomSettings  map[string]interface{} `json:"custom_settings,omitempty"`
}

type WidgetFilter struct {
	Field     string      `json:"field"`
	Operator  string      `json:"operator"` // "eq", "ne", "gt", "lt", "gte", "lte", "in", "like"
	Value     interface{} `json:"value"`
	Label     string      `json:"label,omitempty"`
	IsActive  bool        `json:"is_active"`
}

type WidgetThreshold struct {
	Name      string      `json:"name"`
	Value     float64     `json:"value"`
	Operator  string      `json:"operator"` // "gt", "lt", "gte", "lte"
	Color     string      `json:"color"`
	Severity  string      `json:"severity"` // "info", "warning", "critical"
	Action    string      `json:"action,omitempty"`
	IsEnabled bool        `json:"is_enabled"`
}

type DashboardPermissions struct {
	Owner       string   `json:"owner"`
	Viewers     []string `json:"viewers,omitempty"`
	Editors     []string `json:"editors,omitempty"`
	Admins      []string `json:"admins,omitempty"`
	PublicRead  bool     `json:"public_read"`
	PublicWrite bool     `json:"public_write"`
}

type DashboardCustomization struct {
	BrandingEnabled  bool                   `json:"branding_enabled"`
	LogoURL         string                 `json:"logo_url,omitempty"`
	CompanyName     string                 `json:"company_name,omitempty"`
	HeaderColor     string                 `json:"header_color,omitempty"`
	AccentColor     string                 `json:"accent_color,omitempty"`
	CustomCSS       string                 `json:"custom_css,omitempty"`
	CustomJS        string                 `json:"custom_js,omitempty"`
	FooterText      string                 `json:"footer_text,omitempty"`
	CustomVariables map[string]interface{} `json:"custom_variables,omitempty"`
}

type DashboardOptions struct {
	IncludeData      bool     `json:"include_data"`
	IncludeMetadata  bool     `json:"include_metadata"`
	IncludeWidgets   []string `json:"include_widgets,omitempty"`
	ExcludeWidgets   []string `json:"exclude_widgets,omitempty"`
	DataResolution   string   `json:"data_resolution,omitempty"` // "raw", "1m", "5m", "15m", "1h"
	MaxDataPoints    int      `json:"max_data_points,omitempty"`
	CacheEnabled     bool     `json:"cache_enabled"`
	RealTimeEnabled  bool     `json:"real_time_enabled"`
}

type DashboardData struct {
	Dashboard       *DashboardConfiguration `json:"dashboard"`
	WidgetsData     map[string]*WidgetData  `json:"widgets_data"`
	GeneratedAt     time.Time               `json:"generated_at"`
	DataTimeRange   *TimeRange              `json:"data_time_range"`
	RefreshMetadata *RefreshMetadata        `json:"refresh_metadata"`
	PerformanceInfo *DashboardPerformance   `json:"performance_info,omitempty"`
}

type WidgetData struct {
	WidgetID    string                 `json:"widget_id"`
	Data        interface{}            `json:"data"`
	Metadata    *WidgetDataMetadata    `json:"metadata"`
	Error       string                 `json:"error,omitempty"`
	LastUpdated time.Time              `json:"last_updated"`
	CacheInfo   *WidgetCacheInfo       `json:"cache_info,omitempty"`
	Annotations []*DataAnnotation      `json:"annotations,omitempty"`
}

type WidgetDataMetadata struct {
	RecordCount      int           `json:"record_count"`
	TimeRange        *TimeRange    `json:"time_range"`
	QueryExecutionTime time.Duration `json:"query_execution_time"`
	DataFreshness    time.Duration `json:"data_freshness"`
	IsComplete       bool          `json:"is_complete"`
	HasMoreData      bool          `json:"has_more_data"`
	SamplingRate     float64       `json:"sampling_rate,omitempty"`
	WarningMessages  []string      `json:"warning_messages,omitempty"`
}

type WidgetCacheInfo struct {
	IsCached      bool          `json:"is_cached"`
	CacheHit      bool          `json:"cache_hit"`
	CacheExpiry   time.Time     `json:"cache_expiry"`
	CacheDuration time.Duration `json:"cache_duration"`
	CacheKey      string        `json:"cache_key"`
}

type DataAnnotation struct {
	Timestamp   time.Time `json:"timestamp"`
	Value       float64   `json:"value,omitempty"`
	Text        string    `json:"text"`
	Type        string    `json:"type"` // "event", "alert", "info", "warning", "error"
	Color       string    `json:"color,omitempty"`
	Source      string    `json:"source,omitempty"`
	Description string    `json:"description,omitempty"`
}

type RefreshMetadata struct {
	RequestedAt       time.Time     `json:"requested_at"`
	CompletedAt       time.Time     `json:"completed_at"`
	Duration          time.Duration `json:"duration"`
	WidgetsRefreshed  int           `json:"widgets_refreshed"`
	WidgetsFailed     int           `json:"widgets_failed"`
	CacheHitRate      float64       `json:"cache_hit_rate"`
	DataPointsLoaded  int           `json:"data_points_loaded"`
	QueriesExecuted   int           `json:"queries_executed"`
	IsPartialRefresh  bool          `json:"is_partial_refresh"`
}

type DashboardPerformance struct {
	LoadTime          time.Duration          `json:"load_time"`
	QueryPerformance  map[string]time.Duration `json:"query_performance"`
	RenderTime        time.Duration          `json:"render_time"`
	DataTransferSize  int64                  `json:"data_transfer_size_bytes"`
	MemoryUsage       int64                  `json:"memory_usage_bytes"`
	CacheEfficiency   float64                `json:"cache_efficiency"`
	OptimizationTips  []string               `json:"optimization_tips,omitempty"`
}

type RealTimeWidgetData struct {
	WidgetID    string      `json:"widget_id"`
	Data        interface{} `json:"data"`
	Timestamp   time.Time   `json:"timestamp"`
	ChangeType  string      `json:"change_type"` // "update", "append", "replace"
	ChangeCount int         `json:"change_count"`
}

type RealTimeCallback func(ctx context.Context, data *RealTimeWidgetData) error

type DashboardRefreshResult struct {
	DashboardID       string                `json:"dashboard_id"`
	RefreshStartTime  time.Time             `json:"refresh_start_time"`
	RefreshEndTime    time.Time             `json:"refresh_end_time"`
	Duration          time.Duration         `json:"duration"`
	SuccessfulWidgets []string              `json:"successful_widgets"`
	FailedWidgets     map[string]string     `json:"failed_widgets"` // widgetID -> error
	DataPointsLoaded  int                   `json:"data_points_loaded"`
	CacheHitRate      float64               `json:"cache_hit_rate"`
	PerformanceInfo   *DashboardPerformance `json:"performance_info"`
}

type DashboardTemplate struct {
	TemplateID    string                  `json:"template_id"`
	Name          string                  `json:"name"`
	Description   string                  `json:"description"`
	Category      string                  `json:"category"` // "overview", "performance", "errors", "custom"
	PreviewURL    string                  `json:"preview_url,omitempty"`
	Configuration *DashboardConfiguration `json:"configuration"`
	Variables     []*TemplateVariable     `json:"variables,omitempty"`
	IsOfficial    bool                    `json:"is_official"`
	Rating        float64                 `json:"rating"`
	UsageCount    int                     `json:"usage_count"`
	CreatedBy     string                  `json:"created_by"`
	CreatedAt     time.Time               `json:"created_at"`
	UpdatedAt     time.Time               `json:"updated_at"`
	Tags          []string                `json:"tags"`
}

type TemplateVariable struct {
	Name         string      `json:"name"`
	Label        string      `json:"label"`
	Type         string      `json:"type"` // "text", "number", "select", "multi-select", "date", "time-range"
	DefaultValue interface{} `json:"default_value"`
	Options      []string    `json:"options,omitempty"`
	Required     bool        `json:"required"`
	Description  string      `json:"description,omitempty"`
}

type DashboardExportFormat string

const (
	ExportFormatJSON     DashboardExportFormat = "json"
	ExportFormatYAML     DashboardExportFormat = "yaml"
	ExportFormatTOML     DashboardExportFormat = "toml"
	ExportFormatPDF      DashboardExportFormat = "pdf"
	ExportFormatPNG      DashboardExportFormat = "png"
	ExportFormatSVG      DashboardExportFormat = "svg"
	ExportFormatHTML     DashboardExportFormat = "html"
)

type ShareOptions struct {
	ExpiresAt    time.Time `json:"expires_at,omitempty"`
	IsPublic     bool      `json:"is_public"`
	AllowEdit    bool      `json:"allow_edit"`
	RequireAuth  bool      `json:"require_auth"`
	Password     string    `json:"password,omitempty"`
	AccessLimit  int       `json:"access_limit,omitempty"`
	Description  string    `json:"description,omitempty"`
}

type ShareLink struct {
	ShareID     string       `json:"share_id"`
	URL         string       `json:"url"`
	ShortURL    string       `json:"short_url,omitempty"`
	QRCode      string       `json:"qr_code,omitempty"`
	Options     *ShareOptions `json:"options"`
	CreatedAt   time.Time    `json:"created_at"`
	AccessCount int          `json:"access_count"`
	LastAccess  time.Time    `json:"last_access,omitempty"`
}

type DashboardAnalytics struct {
	DashboardID      string                  `json:"dashboard_id"`
	TimeRange        *TimeRange              `json:"time_range"`
	ViewCount        int                     `json:"view_count"`
	UniqueViewers    int                     `json:"unique_viewers"`
	AverageViewTime  time.Duration           `json:"average_view_time"`
	PopularWidgets   []*WidgetPopularity     `json:"popular_widgets"`
	UserInteractions []*UserInteraction      `json:"user_interactions"`
	PerformanceMetrics *AnalyticsPerformance `json:"performance_metrics"`
	TrendAnalysis    *ViewTrendAnalysis      `json:"trend_analysis"`
	GeneratedAt      time.Time               `json:"generated_at"`
}

type WidgetPopularity struct {
	WidgetID         string        `json:"widget_id"`
	WidgetTitle      string        `json:"widget_title"`
	ViewCount        int           `json:"view_count"`
	InteractionCount int           `json:"interaction_count"`
	AverageViewTime  time.Duration `json:"average_view_time"`
	PopularityScore  float64       `json:"popularity_score"`
}

type UserInteraction struct {
	Timestamp    time.Time `json:"timestamp"`
	UserID       string    `json:"user_id"`
	ActionType   string    `json:"action_type"` // "view", "click", "hover", "filter", "export"
	WidgetID     string    `json:"widget_id,omitempty"`
	Details      map[string]interface{} `json:"details,omitempty"`
	Duration     time.Duration `json:"duration,omitempty"`
	SessionID    string    `json:"session_id"`
}

type AnalyticsPerformance struct {
	AverageLoadTime    time.Duration `json:"average_load_time"`
	AverageQueryTime   time.Duration `json:"average_query_time"`
	CacheHitRate       float64       `json:"cache_hit_rate"`
	ErrorRate          float64       `json:"error_rate"`
	DataFreshnessScore float64       `json:"data_freshness_score"`
	UserSatisfactionScore float64    `json:"user_satisfaction_score"`
}

type ViewTrendAnalysis struct {
	TrendDirection   string    `json:"trend_direction"` // "increasing", "decreasing", "stable"
	GrowthRate       float64   `json:"growth_rate"`
	PeakUsageTime    time.Time `json:"peak_usage_time"`
	LowUsageTime     time.Time `json:"low_usage_time"`
	WeeklyPattern    []float64 `json:"weekly_pattern"`
	HourlyPattern    []float64 `json:"hourly_pattern"`
	Seasonality      string    `json:"seasonality,omitempty"`
}

type PerformanceInsights struct {
	TimeRange             *TimeRange                    `json:"time_range"`
	OverallHealthScore    float64                       `json:"overall_health_score"`
	KeyMetrics            *KeyPerformanceMetrics        `json:"key_metrics"`
	TrendAnalysis         *PerformanceTrendAnalysis     `json:"trend_analysis"`
	AnomaliesDetected     []*PerformanceAnomaly         `json:"anomalies_detected"`
	Recommendations       []*PerformanceRecommendation  `json:"recommendations"`
	ComparisonToPrevious  *PeriodComparison             `json:"comparison_to_previous"`
	ResourceUtilization   *ResourceUtilizationInsights  `json:"resource_utilization"`
	QueryPerformanceInsights *QueryPerformanceInsights  `json:"query_performance_insights"`
	GeneratedAt           time.Time                     `json:"generated_at"`
}

type KeyPerformanceMetrics struct {
	AverageQueryTime      time.Duration `json:"average_query_time"`
	P95QueryTime          time.Duration `json:"p95_query_time"`
	QueriesPerSecond      float64       `json:"queries_per_second"`
	ErrorRate             float64       `json:"error_rate"`
	CacheHitRate          float64       `json:"cache_hit_rate"`
	DatabaseConnections   int           `json:"database_connections"`
	SlowQueryCount        int           `json:"slow_query_count"`
	ThroughputTrend       string        `json:"throughput_trend"`
}

type PerformanceTrendAnalysis struct {
	QueryTimeTrend        *TrendAnalysis `json:"query_time_trend"`
	ThroughputTrend       *TrendAnalysis `json:"throughput_trend"`
	ErrorRateTrend        *TrendAnalysis `json:"error_rate_trend"`
	ResourceUsageTrend    *TrendAnalysis `json:"resource_usage_trend"`
	OverallTrend          string         `json:"overall_trend"`
	Confidence            float64        `json:"confidence"`
}

type PerformanceRecommendation struct {
	Type            string   `json:"type"` // "optimization", "scaling", "configuration", "monitoring"
	Priority        string   `json:"priority"` // "high", "medium", "low"
	Title           string   `json:"title"`
	Description     string   `json:"description"`
	Impact          string   `json:"impact"` // "high", "medium", "low"
	Effort          string   `json:"effort"` // "high", "medium", "low"
	ActionItems     []string `json:"action_items"`
	EstimatedBenefit string  `json:"estimated_benefit"`
	RiskLevel       string   `json:"risk_level"`
}

type PeriodComparison struct {
	PreviousPeriod       *TimeRange             `json:"previous_period"`
	QueryTimeChange      *PercentageChange      `json:"query_time_change"`
	ThroughputChange     *PercentageChange      `json:"throughput_change"`
	ErrorRateChange      *PercentageChange      `json:"error_rate_change"`
	CacheHitRateChange   *PercentageChange      `json:"cache_hit_rate_change"`
	OverallImprovement   bool                   `json:"overall_improvement"`
	SignificantChanges   []*SignificantChange   `json:"significant_changes"`
}

type PercentageChange struct {
	Previous   float64 `json:"previous"`
	Current    float64 `json:"current"`
	Change     float64 `json:"change"`
	Percentage float64 `json:"percentage"`
	IsPositive bool    `json:"is_positive"`
}

type SignificantChange struct {
	MetricName  string  `json:"metric_name"`
	Change      float64 `json:"change"`
	Significance string `json:"significance"` // "high", "medium", "low"
	Impact      string  `json:"impact"`
	Description string  `json:"description"`
}

type ResourceUtilizationInsights struct {
	CPUUtilization     *UtilizationMetrics `json:"cpu_utilization"`
	MemoryUtilization  *UtilizationMetrics `json:"memory_utilization"`
	DiskUtilization    *UtilizationMetrics `json:"disk_utilization"`
	NetworkUtilization *UtilizationMetrics `json:"network_utilization"`
	CapacityWarnings   []string            `json:"capacity_warnings,omitempty"`
	ScalingRecommendations []string        `json:"scaling_recommendations,omitempty"`
}

type UtilizationMetrics struct {
	Average    float64 `json:"average"`
	Peak       float64 `json:"peak"`
	P95        float64 `json:"p95"`
	Trend      string  `json:"trend"`
	HealthStatus string `json:"health_status"`
}

type QueryPerformanceInsights struct {
	TopSlowQueries      []*SlowQueryInsight    `json:"top_slow_queries"`
	QueryPatternAnalysis []*QueryPatternInsight `json:"query_pattern_analysis"`
	IndexRecommendations []*IndexRecommendation `json:"index_recommendations"`
	OptimizationImpact   *OptimizationImpact    `json:"optimization_impact"`
}

type SlowQueryInsight struct {
	Query           string        `json:"query"`
	AverageTime     time.Duration `json:"average_time"`
	CallCount       int           `json:"call_count"`
	TotalTime       time.Duration `json:"total_time"`
	ImpactScore     float64       `json:"impact_score"`
	Recommendation  string        `json:"recommendation"`
}

type QueryPatternInsight struct {
	Pattern         string  `json:"pattern"`
	Frequency       int     `json:"frequency"`
	AverageTime     time.Duration `json:"average_time"`
	TrendDirection  string  `json:"trend_direction"`
	OptimizationTip string  `json:"optimization_tip"`
}

type OptimizationImpact struct {
	PotentialTimeSaving time.Duration `json:"potential_time_saving"`
	AffectedQueries     int           `json:"affected_queries"`
	ImprovementPercent  float64       `json:"improvement_percent"`
	EffortRequired      string        `json:"effort_required"`
}

type DashboardReportOptions struct {
	IncludeSummary      bool     `json:"include_summary"`
	IncludeWidgets      []string `json:"include_widgets,omitempty"`
	IncludeAnalytics    bool     `json:"include_analytics"`
	IncludeInsights     bool     `json:"include_insights"`
	Format              string   `json:"format"` // "pdf", "html", "json"
	EmailRecipients     []string `json:"email_recipients,omitempty"`
	ScheduleFrequency   string   `json:"schedule_frequency,omitempty"` // "daily", "weekly", "monthly"
	CustomSections      []string `json:"custom_sections,omitempty"`
}

type DashboardReport struct {
	ReportID        string                   `json:"report_id"`
	DashboardID     string                   `json:"dashboard_id"`
	ReportOptions   *DashboardReportOptions  `json:"report_options"`
	GeneratedAt     time.Time                `json:"generated_at"`
	TimeRange       *TimeRange               `json:"time_range"`
	ExecutiveSummary string                  `json:"executive_summary"`
	Sections        []*DashboardReportSection `json:"sections"`
	Attachments     []*ReportAttachment       `json:"attachments,omitempty"`
	Metadata        map[string]interface{}    `json:"metadata,omitempty"`
}

type DashboardReportSection struct {
	SectionID   string                 `json:"section_id"`
	Title       string                 `json:"title"`
	Content     string                 `json:"content"`
	WidgetData  []*WidgetData          `json:"widget_data,omitempty"`
	Charts      []*ChartData           `json:"charts,omitempty"`
	Tables      []*TableData           `json:"tables,omitempty"`
	Insights    []string               `json:"insights,omitempty"`
	Metrics     map[string]interface{} `json:"metrics,omitempty"`
}

// Implementation

type databasePerformanceDashboard struct {
	db                    *gorm.DB
	metricsCollector     QueryPerformanceMetricsCollector
	slowQueryLogger      DatabaseSlowQueryLogger
	
	// Dashboard storage and management
	dashboards           map[string]*DashboardConfiguration
	dashboardsMutex      sync.RWMutex
	templates            []*DashboardTemplate
	templatesMutex       sync.RWMutex
	
	// Real-time subscriptions
	realTimeSubscriptions map[string]RealTimeCallback
	subscriptionsMutex    sync.RWMutex
	
	// Caching
	dataCache            map[string]*WidgetData
	cacheMutex           sync.RWMutex
	cacheExpiry          time.Duration
	
	// Performance tracking
	analyticsData        map[string]*DashboardAnalytics
	analyticsMutex       sync.RWMutex
}

func NewDatabasePerformanceDashboard(db *gorm.DB, metricsCollector QueryPerformanceMetricsCollector, slowQueryLogger DatabaseSlowQueryLogger) DatabasePerformanceDashboard {
	dashboard := &databasePerformanceDashboard{
		db:                   db,
		metricsCollector:    metricsCollector,
		slowQueryLogger:     slowQueryLogger,
		dashboards:          make(map[string]*DashboardConfiguration),
		dataCache:           make(map[string]*WidgetData),
		realTimeSubscriptions: make(map[string]RealTimeCallback),
		analyticsData:       make(map[string]*DashboardAnalytics),
		cacheExpiry:         5 * time.Minute,
	}
	
	// Initialize default templates
	dashboard.initializeDefaultTemplates()
	
	return dashboard
}

func (d *databasePerformanceDashboard) Initialize(ctx context.Context) error {
	// Create default dashboards
	if err := d.createDefaultDashboards(ctx); err != nil {
		return fmt.Errorf("failed to create default dashboards: %w", err)
	}
	
	log.Printf("Database performance dashboard initialized")
	return nil
}

func (d *databasePerformanceDashboard) GetDashboardConfig() (*DashboardConfiguration, error) {
	// Return the main performance dashboard config
	d.dashboardsMutex.RLock()
	defer d.dashboardsMutex.RUnlock()
	
	if mainDashboard, exists := d.dashboards["main_performance"]; exists {
		return mainDashboard, nil
	}
	
	return nil, fmt.Errorf("main dashboard not found")
}

func (d *databasePerformanceDashboard) UpdateDashboardConfig(config *DashboardConfiguration) error {
	d.dashboardsMutex.Lock()
	defer d.dashboardsMutex.Unlock()
	
	config.UpdatedAt = time.Now()
	if config.Version == "" {
		config.Version = "1.0"
	}
	
	d.dashboards[config.DashboardID] = config
	
	log.Printf("Dashboard %s configuration updated", config.DashboardID)
	return nil
}

func (d *databasePerformanceDashboard) GetDashboardData(ctx context.Context, dashboardID string, timeRange *TimeRange, options *DashboardOptions) (*DashboardData, error) {
	// Get dashboard configuration
	d.dashboardsMutex.RLock()
	dashboard, exists := d.dashboards[dashboardID]
	d.dashboardsMutex.RUnlock()
	
	if !exists {
		return nil, fmt.Errorf("dashboard %s not found", dashboardID)
	}
	
	// Use default time range if not specified
	if timeRange == nil {
		timeRange = dashboard.DefaultTimeRange
	}
	if timeRange == nil {
		timeRange = &TimeRange{
			Start: time.Now().Add(-24 * time.Hour),
			End:   time.Now(),
		}
	}
	
	// Set default options
	if options == nil {
		options = &DashboardOptions{
			IncludeData:     true,
			IncludeMetadata: true,
			CacheEnabled:    true,
		}
	}
	
	startTime := time.Now()
	
	dashboardData := &DashboardData{
		Dashboard:     dashboard,
		WidgetsData:   make(map[string]*WidgetData),
		GeneratedAt:   startTime,
		DataTimeRange: timeRange,
	}
	
	// Load data for each widget
	var wg sync.WaitGroup
	var dataMutex sync.Mutex
	errors := make(map[string]string)
	
	for _, widget := range dashboard.Widgets {
		if !widget.IsVisible {
			continue
		}
		
		// Skip if widget is excluded
		if d.isWidgetExcluded(widget.WidgetID, options) {
			continue
		}
		
		wg.Add(1)
		go func(w *DashboardWidget) {
			defer wg.Done()
			
			widgetData, err := d.loadWidgetData(ctx, w, timeRange, options)
			if err != nil {
				dataMutex.Lock()
				errors[w.WidgetID] = err.Error()
				dataMutex.Unlock()
				return
			}
			
			dataMutex.Lock()
			dashboardData.WidgetsData[w.WidgetID] = widgetData
			dataMutex.Unlock()
		}(widget)
	}
	
	wg.Wait()
	
	// Create refresh metadata
	endTime := time.Now()
	successCount := len(dashboardData.WidgetsData)
	failCount := len(errors)
	
	dashboardData.RefreshMetadata = &RefreshMetadata{
		RequestedAt:      startTime,
		CompletedAt:      endTime,
		Duration:         endTime.Sub(startTime),
		WidgetsRefreshed: successCount,
		WidgetsFailed:    failCount,
		IsPartialRefresh: failCount > 0,
	}
	
	// Calculate performance info
	if options.IncludeMetadata {
		dashboardData.PerformanceInfo = d.calculateDashboardPerformance(dashboardData)
	}
	
	// Track analytics
	d.trackDashboardView(dashboardID, time.Now())
	
	return dashboardData, nil
}

func (d *databasePerformanceDashboard) GetDashboardWidgets(ctx context.Context, dashboardID string) ([]*DashboardWidget, error) {
	d.dashboardsMutex.RLock()
	defer d.dashboardsMutex.RUnlock()
	
	dashboard, exists := d.dashboards[dashboardID]
	if !exists {
		return nil, fmt.Errorf("dashboard %s not found", dashboardID)
	}
	
	return dashboard.Widgets, nil
}

func (d *databasePerformanceDashboard) RefreshDashboard(ctx context.Context, dashboardID string) (*DashboardRefreshResult, error) {
	startTime := time.Now()
	
	// Clear cache for this dashboard
	d.clearDashboardCache(dashboardID)
	
	// Get fresh dashboard data
	dashboardData, err := d.GetDashboardData(ctx, dashboardID, nil, &DashboardOptions{
		IncludeData:     true,
		IncludeMetadata: true,
		CacheEnabled:    false, // Force refresh
	})
	
	endTime := time.Now()
	
	if err != nil {
		return nil, fmt.Errorf("failed to refresh dashboard: %w", err)
	}
	
	// Build refresh result
	result := &DashboardRefreshResult{
		DashboardID:      dashboardID,
		RefreshStartTime: startTime,
		RefreshEndTime:   endTime,
		Duration:         endTime.Sub(startTime),
		PerformanceInfo:  dashboardData.PerformanceInfo,
	}
	
	// Collect successful and failed widgets
	for widgetID, widgetData := range dashboardData.WidgetsData {
		if widgetData.Error == "" {
			result.SuccessfulWidgets = append(result.SuccessfulWidgets, widgetID)
		} else {
			if result.FailedWidgets == nil {
				result.FailedWidgets = make(map[string]string)
			}
			result.FailedWidgets[widgetID] = widgetData.Error
		}
	}
	
	if dashboardData.RefreshMetadata != nil {
		result.DataPointsLoaded = dashboardData.RefreshMetadata.DataPointsLoaded
		result.CacheHitRate = dashboardData.RefreshMetadata.CacheHitRate
	}
	
	log.Printf("Dashboard %s refreshed in %v", dashboardID, result.Duration)
	return result, nil
}

func (d *databasePerformanceDashboard) GetRealTimeData(ctx context.Context, widgetID string) (*RealTimeWidgetData, error) {
	// Find the widget configuration
	var widget *DashboardWidget
	d.dashboardsMutex.RLock()
	for _, dashboard := range d.dashboards {
		for _, w := range dashboard.Widgets {
			if w.WidgetID == widgetID {
				widget = w
				break
			}
		}
		if widget != nil {
			break
		}
	}
	d.dashboardsMutex.RUnlock()
	
	if widget == nil {
		return nil, fmt.Errorf("widget %s not found", widgetID)
	}
	
	// Get current data for the widget
	timeRange := &TimeRange{
		Start: time.Now().Add(-5 * time.Minute),
		End:   time.Now(),
	}
	
	widgetData, err := d.loadWidgetData(ctx, widget, timeRange, &DashboardOptions{
		IncludeData:  true,
		CacheEnabled: false,
	})
	
	if err != nil {
		return nil, fmt.Errorf("failed to get real-time data: %w", err)
	}
	
	return &RealTimeWidgetData{
		WidgetID:    widgetID,
		Data:        widgetData.Data,
		Timestamp:   time.Now(),
		ChangeType:  "update",
		ChangeCount: 1,
	}, nil
}

func (d *databasePerformanceDashboard) SubscribeToRealTimeUpdates(ctx context.Context, dashboardID string, callback RealTimeCallback) error {
	d.subscriptionsMutex.Lock()
	defer d.subscriptionsMutex.Unlock()
	
	d.realTimeSubscriptions[dashboardID] = callback
	
	log.Printf("Subscribed to real-time updates for dashboard %s", dashboardID)
	return nil
}

func (d *databasePerformanceDashboard) UnsubscribeFromRealTimeUpdates(dashboardID string) error {
	d.subscriptionsMutex.Lock()
	defer d.subscriptionsMutex.Unlock()
	
	delete(d.realTimeSubscriptions, dashboardID)
	
	log.Printf("Unsubscribed from real-time updates for dashboard %s", dashboardID)
	return nil
}

// Widget management methods

func (d *databasePerformanceDashboard) CreateWidget(ctx context.Context, widget *DashboardWidget) error {
	widget.CreatedAt = time.Now()
	widget.UpdatedAt = time.Now()
	
	if widget.WidgetID == "" {
		widget.WidgetID = fmt.Sprintf("widget_%d", time.Now().UnixNano())
	}
	
	log.Printf("Widget %s created", widget.WidgetID)
	return nil
}

func (d *databasePerformanceDashboard) UpdateWidget(ctx context.Context, widgetID string, widget *DashboardWidget) error {
	widget.WidgetID = widgetID
	widget.UpdatedAt = time.Now()
	
	// Clear cache for this widget
	d.clearWidgetCache(widgetID)
	
	log.Printf("Widget %s updated", widgetID)
	return nil
}

func (d *databasePerformanceDashboard) DeleteWidget(ctx context.Context, widgetID string) error {
	// Clear cache for this widget
	d.clearWidgetCache(widgetID)
	
	log.Printf("Widget %s deleted", widgetID)
	return nil
}

func (d *databasePerformanceDashboard) CloneWidget(ctx context.Context, widgetID string, newWidgetID string) (*DashboardWidget, error) {
	// Find the original widget
	var originalWidget *DashboardWidget
	d.dashboardsMutex.RLock()
	for _, dashboard := range d.dashboards {
		for _, w := range dashboard.Widgets {
			if w.WidgetID == widgetID {
				originalWidget = w
				break
			}
		}
		if originalWidget != nil {
			break
		}
	}
	d.dashboardsMutex.RUnlock()
	
	if originalWidget == nil {
		return nil, fmt.Errorf("widget %s not found", widgetID)
	}
	
	// Clone the widget
	clonedWidget := *originalWidget
	clonedWidget.WidgetID = newWidgetID
	clonedWidget.Title = fmt.Sprintf("%s (Copy)", originalWidget.Title)
	clonedWidget.CreatedAt = time.Now()
	clonedWidget.UpdatedAt = time.Now()
	
	log.Printf("Widget %s cloned as %s", widgetID, newWidgetID)
	return &clonedWidget, nil
}

// Template management methods

func (d *databasePerformanceDashboard) GetDashboardTemplates() ([]*DashboardTemplate, error) {
	d.templatesMutex.RLock()
	defer d.templatesMutex.RUnlock()
	
	return d.templates, nil
}

func (d *databasePerformanceDashboard) CreateDashboardFromTemplate(ctx context.Context, templateID string, customization *DashboardCustomization) (*DashboardData, error) {
	d.templatesMutex.RLock()
	var template *DashboardTemplate
	for _, t := range d.templates {
		if t.TemplateID == templateID {
			template = t
			break
		}
	}
	d.templatesMutex.RUnlock()
	
	if template == nil {
		return nil, fmt.Errorf("template %s not found", templateID)
	}
	
	// Clone the template configuration
	dashboardConfig := *template.Configuration
	dashboardConfig.DashboardID = fmt.Sprintf("dashboard_%d", time.Now().UnixNano())
	dashboardConfig.CreatedAt = time.Now()
	dashboardConfig.UpdatedAt = time.Now()
	
	// Apply customization
	if customization != nil {
		dashboardConfig.Customization = customization
	}
	
	// Store the new dashboard
	d.dashboardsMutex.Lock()
	d.dashboards[dashboardConfig.DashboardID] = &dashboardConfig
	d.dashboardsMutex.Unlock()
	
	// Generate initial dashboard data
	return d.GetDashboardData(ctx, dashboardConfig.DashboardID, nil, nil)
}

func (d *databasePerformanceDashboard) SaveDashboardAsTemplate(ctx context.Context, dashboardID string, templateName string) (*DashboardTemplate, error) {
	d.dashboardsMutex.RLock()
	dashboard, exists := d.dashboards[dashboardID]
	d.dashboardsMutex.RUnlock()
	
	if !exists {
		return nil, fmt.Errorf("dashboard %s not found", dashboardID)
	}
	
	template := &DashboardTemplate{
		TemplateID:    fmt.Sprintf("template_%d", time.Now().UnixNano()),
		Name:          templateName,
		Description:   fmt.Sprintf("Template created from dashboard %s", dashboardID),
		Category:      "custom",
		Configuration: dashboard,
		IsOfficial:    false,
		CreatedAt:     time.Now(),
		UpdatedAt:     time.Now(),
	}
	
	d.templatesMutex.Lock()
	d.templates = append(d.templates, template)
	d.templatesMutex.Unlock()
	
	log.Printf("Dashboard %s saved as template %s", dashboardID, template.TemplateID)
	return template, nil
}

// Export and sharing methods

func (d *databasePerformanceDashboard) ExportDashboard(ctx context.Context, dashboardID string, format DashboardExportFormat) ([]byte, error) {
	dashboardData, err := d.GetDashboardData(ctx, dashboardID, nil, &DashboardOptions{
		IncludeData:     true,
		IncludeMetadata: true,
	})
	
	if err != nil {
		return nil, fmt.Errorf("failed to get dashboard data for export: %w", err)
	}
	
	switch format {
	case ExportFormatJSON:
		return json.MarshalIndent(dashboardData, "", "  ")
	case ExportFormatHTML:
		return d.exportToHTML(dashboardData)
	default:
		return nil, fmt.Errorf("unsupported export format: %s", format)
	}
}

func (d *databasePerformanceDashboard) GetDashboardShareLink(ctx context.Context, dashboardID string, shareOptions *ShareOptions) (*ShareLink, error) {
	if shareOptions == nil {
		shareOptions = &ShareOptions{
			IsPublic:    true,
			RequireAuth: false,
		}
	}
	
	shareLink := &ShareLink{
		ShareID:   fmt.Sprintf("share_%d", time.Now().UnixNano()),
		URL:       fmt.Sprintf("/shared/dashboard/%s/%s", dashboardID, fmt.Sprintf("share_%d", time.Now().UnixNano())),
		Options:   shareOptions,
		CreatedAt: time.Now(),
	}
	
	log.Printf("Share link created for dashboard %s", dashboardID)
	return shareLink, nil
}

func (d *databasePerformanceDashboard) ImportDashboard(ctx context.Context, data []byte, format DashboardExportFormat) (*DashboardData, error) {
	var dashboardData *DashboardData
	
	switch format {
	case ExportFormatJSON:
		if err := json.Unmarshal(data, &dashboardData); err != nil {
			return nil, fmt.Errorf("failed to unmarshal dashboard data: %w", err)
		}
	default:
		return nil, fmt.Errorf("unsupported import format: %s", format)
	}
	
	// Generate new ID for imported dashboard
	dashboardData.Dashboard.DashboardID = fmt.Sprintf("imported_%d", time.Now().UnixNano())
	dashboardData.Dashboard.CreatedAt = time.Now()
	dashboardData.Dashboard.UpdatedAt = time.Now()
	
	// Store the imported dashboard
	d.dashboardsMutex.Lock()
	d.dashboards[dashboardData.Dashboard.DashboardID] = dashboardData.Dashboard
	d.dashboardsMutex.Unlock()
	
	log.Printf("Dashboard imported as %s", dashboardData.Dashboard.DashboardID)
	return dashboardData, nil
}

// Analytics and insights methods

func (d *databasePerformanceDashboard) GetDashboardUsageAnalytics(ctx context.Context, dashboardID string, timeRange *TimeRange) (*DashboardAnalytics, error) {
	d.analyticsMutex.RLock()
	defer d.analyticsMutex.RUnlock()
	
	if analytics, exists := d.analyticsData[dashboardID]; exists {
		return analytics, nil
	}
	
	// Return basic analytics
	return &DashboardAnalytics{
		DashboardID: dashboardID,
		TimeRange:   timeRange,
		ViewCount:   1,
		UniqueViewers: 1,
		GeneratedAt: time.Now(),
	}, nil
}

func (d *databasePerformanceDashboard) GetPerformanceInsights(ctx context.Context, timeRange *TimeRange) (*PerformanceInsights, error) {
	if timeRange == nil {
		timeRange = &TimeRange{
			Start: time.Now().Add(-24 * time.Hour),
			End:   time.Now(),
		}
	}
	
	// Get metrics for analysis
	filter := &MetricsFilter{TimeRange: timeRange}
	collection, err := d.metricsCollector.GetQueryMetrics(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("failed to get metrics for insights: %w", err)
	}
	
	insights := &PerformanceInsights{
		TimeRange:      timeRange,
		GeneratedAt:    time.Now(),
	}
	
	// Calculate overall health score
	insights.OverallHealthScore = d.calculateHealthScore(collection.Summary)
	
	// Build key metrics
	insights.KeyMetrics = &KeyPerformanceMetrics{
		AverageQueryTime: collection.Summary.AverageExecutionTime,
		P95QueryTime:     collection.Summary.P95ExecutionTime,
		ErrorRate:        collection.Summary.ErrorRate,
		CacheHitRate:     collection.Summary.CacheHitRate,
		SlowQueryCount:   d.countSlowQueries(collection.Metrics),
	}
	
	// Generate recommendations
	insights.Recommendations = d.generatePerformanceRecommendations(collection.Summary)
	
	// Create comparison to previous period
	insights.ComparisonToPrevious = d.generatePeriodComparison(ctx, timeRange)
	
	return insights, nil
}

func (d *databasePerformanceDashboard) GenerateDashboardReport(ctx context.Context, dashboardID string, reportOptions *DashboardReportOptions) (*DashboardReport, error) {
	if reportOptions == nil {
		reportOptions = &DashboardReportOptions{
			IncludeSummary:   true,
			IncludeAnalytics: true,
			IncludeInsights:  true,
			Format:           "json",
		}
	}
	
	// Get dashboard data
	dashboardData, err := d.GetDashboardData(ctx, dashboardID, nil, &DashboardOptions{
		IncludeData:     true,
		IncludeMetadata: true,
	})
	
	if err != nil {
		return nil, fmt.Errorf("failed to get dashboard data for report: %w", err)
	}
	
	report := &DashboardReport{
		ReportID:      fmt.Sprintf("report_%d", time.Now().UnixNano()),
		DashboardID:   dashboardID,
		ReportOptions: reportOptions,
		GeneratedAt:   time.Now(),
		TimeRange:     dashboardData.DataTimeRange,
		Sections:      []*DashboardReportSection{},
	}
	
	// Generate executive summary
	report.ExecutiveSummary = d.generateExecutiveSummary(dashboardData)
	
	// Add sections based on options
	if reportOptions.IncludeSummary {
		report.Sections = append(report.Sections, d.generateSummarySection(dashboardData))
	}
	
	if reportOptions.IncludeAnalytics {
		analyticsSection, _ := d.generateAnalyticsSection(ctx, dashboardID, dashboardData.DataTimeRange)
		if analyticsSection != nil {
			report.Sections = append(report.Sections, analyticsSection)
		}
	}
	
	if reportOptions.IncludeInsights {
		insightsSection, _ := d.generateInsightsSection(ctx, dashboardData.DataTimeRange)
		if insightsSection != nil {
			report.Sections = append(report.Sections, insightsSection)
		}
	}
	
	log.Printf("Report %s generated for dashboard %s", report.ReportID, dashboardID)
	return report, nil
}

// Internal helper methods

func (d *databasePerformanceDashboard) initializeDefaultTemplates() {
	d.templatesMutex.Lock()
	defer d.templatesMutex.Unlock()
	
	// Performance Overview Template
	overviewTemplate := &DashboardTemplate{
		TemplateID:  "performance_overview",
		Name:        "Performance Overview",
		Description: "Comprehensive database performance monitoring dashboard",
		Category:    "overview",
		IsOfficial:  true,
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
	}
	
	// Slow Queries Template
	slowQueriesTemplate := &DashboardTemplate{
		TemplateID:  "slow_queries",
		Name:        "Slow Queries Analysis",
		Description: "Detailed analysis of slow queries and optimization opportunities",
		Category:    "performance",
		IsOfficial:  true,
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
	}
	
	d.templates = []*DashboardTemplate{
		overviewTemplate,
		slowQueriesTemplate,
	}
}

func (d *databasePerformanceDashboard) createDefaultDashboards(ctx context.Context) error {
	// Create main performance dashboard
	mainDashboard := &DashboardConfiguration{
		DashboardID:     "main_performance",
		Name:            "Database Performance Overview",
		Description:     "Main database performance monitoring dashboard",
		Theme:           "light",
		RefreshInterval: 30 * time.Second,
		AutoRefresh:     true,
		TimeZone:        "UTC",
		DefaultTimeRange: &TimeRange{
			Start: time.Now().Add(-24 * time.Hour),
			End:   time.Now(),
		},
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		IsPublic:  true,
		Version:   "1.0",
	}
	
	// Add widgets to main dashboard
	mainDashboard.Widgets = d.createDefaultWidgets()
	
	d.dashboardsMutex.Lock()
	d.dashboards[mainDashboard.DashboardID] = mainDashboard
	d.dashboardsMutex.Unlock()
	
	log.Printf("Default dashboard created: %s", mainDashboard.DashboardID)
	return nil
}

func (d *databasePerformanceDashboard) createDefaultWidgets() []*DashboardWidget {
	widgets := []*DashboardWidget{
		{
			WidgetID: "query_performance_chart",
			Title:    "Query Performance Over Time",
			Type:     WidgetTypeChart,
			Position: &WidgetPosition{X: 0, Y: 0},
			Size:     &WidgetSize{Width: 6, Height: 4},
			Configuration: &WidgetConfiguration{
				ChartType:       "line",
				MetricField:     "execution_time",
				TimeGrouping:    "5m",
				ShowLegend:      true,
				ShowGrid:        true,
				RefreshRate:     30 * time.Second,
			},
			DataSource: &WidgetDataSource{
				SourceType:   DataSourceMetrics,
				Metrics:      []string{"average_execution_time", "p95_execution_time"},
				TimeField:    "timestamp",
				ValueFields:  []string{"execution_time"},
			},
			IsVisible: true,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			WidgetID: "slow_queries_count",
			Title:    "Slow Queries Count",
			Type:     WidgetTypeCounter,
			Position: &WidgetPosition{X: 6, Y: 0},
			Size:     &WidgetSize{Width: 3, Height: 2},
			Configuration: &WidgetConfiguration{
				AggregationType: "count",
				RefreshRate:     30 * time.Second,
			},
			DataSource: &WidgetDataSource{
				SourceType: DataSourceSlowQueries,
				Filters: map[string]interface{}{
					"min_execution_time": "200ms",
				},
			},
			Thresholds: []*WidgetThreshold{
				{
					Name:      "Warning",
					Value:     10,
					Operator:  "gt",
					Color:     "#FFA500",
					Severity:  "warning",
					IsEnabled: true,
				},
				{
					Name:      "Critical",
					Value:     50,
					Operator:  "gt",
					Color:     "#FF0000",
					Severity:  "critical",
					IsEnabled: true,
				},
			},
			IsVisible: true,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			WidgetID: "error_rate_gauge",
			Title:    "Error Rate",
			Type:     WidgetTypeGauge,
			Position: &WidgetPosition{X: 9, Y: 0},
			Size:     &WidgetSize{Width: 3, Height: 2},
			Configuration: &WidgetConfiguration{
				MetricField:  "error_rate",
				RefreshRate:  30 * time.Second,
			},
			DataSource: &WidgetDataSource{
				SourceType:  DataSourceMetrics,
				Metrics:     []string{"error_rate"},
				ValueFields: []string{"error_rate"},
			},
			Thresholds: []*WidgetThreshold{
				{
					Name:      "Good",
					Value:     0.01, // 1%
					Operator:  "lt",
					Color:     "#00FF00",
					Severity:  "info",
					IsEnabled: true,
				},
				{
					Name:      "Warning",
					Value:     0.05, // 5%
					Operator:  "gt",
					Color:     "#FFA500",
					Severity:  "warning",
					IsEnabled: true,
				},
			},
			IsVisible: true,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			WidgetID: "top_slow_queries_table",
			Title:    "Top Slow Queries",
			Type:     WidgetTypeTable,
			Position: &WidgetPosition{X: 0, Y: 4},
			Size:     &WidgetSize{Width: 12, Height: 4},
			Configuration: &WidgetConfiguration{
				SortBy:    "execution_time",
				SortOrder: "desc",
				Limit:     10,
				RefreshRate: 60 * time.Second,
			},
			DataSource: &WidgetDataSource{
				SourceType: DataSourceSlowQueries,
				ValueFields: []string{"query_text", "execution_time", "call_count"},
				Filters: map[string]interface{}{
					"min_execution_time": "100ms",
				},
			},
			IsVisible: true,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
	}
	
	return widgets
}

func (d *databasePerformanceDashboard) loadWidgetData(ctx context.Context, widget *DashboardWidget, timeRange *TimeRange, options *DashboardOptions) (*WidgetData, error) {
	// Check cache first
	if options.CacheEnabled {
		if cachedData := d.getCachedWidgetData(widget.WidgetID); cachedData != nil {
			return cachedData, nil
		}
	}
	
	startTime := time.Now()
	widgetData := &WidgetData{
		WidgetID:    widget.WidgetID,
		LastUpdated: startTime,
		Metadata: &WidgetDataMetadata{
			TimeRange: timeRange,
		},
	}
	
	// Load data based on widget data source
	var data interface{}
	var err error
	
	switch widget.DataSource.SourceType {
	case DataSourceMetrics:
		data, err = d.loadMetricsData(ctx, widget, timeRange)
	case DataSourceSlowQueries:
		data, err = d.loadSlowQueriesData(ctx, widget, timeRange)
	case DataSourceDatabase:
		data, err = d.loadDatabaseData(ctx, widget, timeRange)
	case DataSourceSystem:
		data, err = d.loadSystemData(ctx, widget, timeRange)
	default:
		err = fmt.Errorf("unsupported data source type: %s", widget.DataSource.SourceType)
	}
	
	if err != nil {
		widgetData.Error = err.Error()
		return widgetData, nil // Return widget with error, don't fail completely
	}
	
	widgetData.Data = data
	
	// Update metadata
	endTime := time.Now()
	widgetData.Metadata.QueryExecutionTime = endTime.Sub(startTime)
	widgetData.Metadata.DataFreshness = time.Since(startTime)
	widgetData.Metadata.IsComplete = true
	
	if dataSlice, ok := data.([]interface{}); ok {
		widgetData.Metadata.RecordCount = len(dataSlice)
	} else if dataMap, ok := data.(map[string]interface{}); ok {
		if count, exists := dataMap["count"]; exists {
			if countInt, ok := count.(int); ok {
				widgetData.Metadata.RecordCount = countInt
			}
		}
	}
	
	// Cache the result
	if options.CacheEnabled {
		d.cacheWidgetData(widget.WidgetID, widgetData)
	}
	
	return widgetData, nil
}

func (d *databasePerformanceDashboard) loadMetricsData(ctx context.Context, widget *DashboardWidget, timeRange *TimeRange) (interface{}, error) {
	// Get metrics from collector
	filter := &MetricsFilter{
		TimeRange: timeRange,
		Limit:     widget.Configuration.Limit,
	}
	
	collection, err := d.metricsCollector.GetQueryMetrics(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("failed to load metrics data: %w", err)
	}
	
	// Format data based on widget type
	switch widget.Type {
	case WidgetTypeChart:
		return d.formatChartData(collection, widget)
	case WidgetTypeCounter:
		return map[string]interface{}{
			"count": len(collection.Metrics),
			"value": len(collection.Metrics),
		}, nil
	case WidgetTypeGauge:
		return map[string]interface{}{
			"value": collection.Summary.ErrorRate,
			"min":   0.0,
			"max":   1.0,
		}, nil
	case WidgetTypeTable:
		return d.formatTableData(collection, widget)
	default:
		return collection.Summary, nil
	}
}

func (d *databasePerformanceDashboard) loadSlowQueriesData(ctx context.Context, widget *DashboardWidget, timeRange *TimeRange) (interface{}, error) {
	// Get slow queries
	slowQueries, err := d.slowQueryLogger.GetSlowQueries(ctx, timeRange.Start, widget.Configuration.Limit)
	if err != nil {
		return nil, fmt.Errorf("failed to load slow queries data: %w", err)
	}
	
	// Format based on widget type
	switch widget.Type {
	case WidgetTypeCounter:
		return map[string]interface{}{
			"count": len(slowQueries),
			"value": len(slowQueries),
		}, nil
	case WidgetTypeTable:
		return d.formatSlowQueriesTableData(slowQueries, widget)
	default:
		return slowQueries, nil
	}
}

func (d *databasePerformanceDashboard) loadDatabaseData(ctx context.Context, widget *DashboardWidget, timeRange *TimeRange) (interface{}, error) {
	// Simple database stats
	var connectionCount int64
	if err := d.db.WithContext(ctx).Raw("SELECT count(*) FROM pg_stat_activity").Scan(&connectionCount).Error; err != nil {
		return nil, fmt.Errorf("failed to get connection count: %w", err)
	}
	
	return map[string]interface{}{
		"active_connections": connectionCount,
		"timestamp":         time.Now(),
	}, nil
}

func (d *databasePerformanceDashboard) loadSystemData(ctx context.Context, widget *DashboardWidget, timeRange *TimeRange) (interface{}, error) {
	// Mock system data - in real implementation, this would query system metrics
	return map[string]interface{}{
		"cpu_usage":    50.0,
		"memory_usage": 60.0,
		"disk_usage":   70.0,
		"timestamp":    time.Now(),
	}, nil
}

// Helper methods for data formatting

func (d *databasePerformanceDashboard) formatChartData(collection *QueryMetricsCollection, widget *DashboardWidget) (interface{}, error) {
	// Group data by time intervals
	dataPoints := make([]map[string]interface{}, 0)
	
	// Simple aggregation - in real implementation, this would be more sophisticated
	timeGroups := make(map[string][]*QueryExecutionMetric)
	
	for _, metric := range collection.Metrics {
		// Group by 5-minute intervals
		timeKey := metric.Timestamp.Truncate(5 * time.Minute).Format(time.RFC3339)
		timeGroups[timeKey] = append(timeGroups[timeKey], metric)
	}
	
	// Convert to chart format
	for timeKey, metrics := range timeGroups {
		if len(metrics) == 0 {
			continue
		}
		
		var totalTime time.Duration
		for _, metric := range metrics {
			totalTime += metric.ExecutionTime
		}
		
		avgTime := totalTime / time.Duration(len(metrics))
		
		dataPoint := map[string]interface{}{
			"timestamp":           timeKey,
			"average_exec_time":   avgTime.Milliseconds(),
			"query_count":         len(metrics),
		}
		
		dataPoints = append(dataPoints, dataPoint)
	}
	
	// Sort by timestamp
	sort.Slice(dataPoints, func(i, j int) bool {
		timeI, _ := time.Parse(time.RFC3339, dataPoints[i]["timestamp"].(string))
		timeJ, _ := time.Parse(time.RFC3339, dataPoints[j]["timestamp"].(string))
		return timeI.Before(timeJ)
	})
	
	return map[string]interface{}{
		"series": []map[string]interface{}{
			{
				"name": "Average Execution Time",
				"data": dataPoints,
			},
		},
	}, nil
}

func (d *databasePerformanceDashboard) formatTableData(collection *QueryMetricsCollection, widget *DashboardWidget) (interface{}, error) {
	rows := make([]map[string]interface{}, 0)
	
	for i, metric := range collection.Metrics {
		if i >= widget.Configuration.Limit && widget.Configuration.Limit > 0 {
			break
		}
		
		row := map[string]interface{}{
			"query_id":       metric.QueryID,
			"query_type":     metric.QueryType,
			"execution_time": metric.ExecutionTime.Milliseconds(),
			"timestamp":      metric.Timestamp.Format(time.RFC3339),
			"error":          metric.ErrorOccurred,
		}
		
		rows = append(rows, row)
	}
	
	return map[string]interface{}{
		"columns": []string{"Query ID", "Type", "Execution Time (ms)", "Timestamp", "Error"},
		"rows":    rows,
	}, nil
}

func (d *databasePerformanceDashboard) formatSlowQueriesTableData(slowQueries []DetectedSlowQuery, widget *DashboardWidget) (interface{}, error) {
	rows := make([]map[string]interface{}, 0)
	
	for i, query := range slowQueries {
		if i >= widget.Configuration.Limit && widget.Configuration.Limit > 0 {
			break
		}
		
		// Truncate query text for display
		queryText := query.QueryText
		if len(queryText) > 100 {
			queryText = queryText[:97] + "..."
		}
		
		row := map[string]interface{}{
			"query_text":     queryText,
			"execution_time": query.ExecutionTime.Milliseconds(),
			"detected_at":    query.DetectedAt.Format("2006-01-02 15:04:05"),
			"severity":       query.Severity,
		}
		
		rows = append(rows, row)
	}
	
	return map[string]interface{}{
		"columns": []string{"Query", "Execution Time (ms)", "Detected At", "Severity"},
		"rows":    rows,
	}, nil
}

// Cache management methods

func (d *databasePerformanceDashboard) getCachedWidgetData(widgetID string) *WidgetData {
	d.cacheMutex.RLock()
	defer d.cacheMutex.RUnlock()
	
	if data, exists := d.dataCache[widgetID]; exists {
		if time.Since(data.LastUpdated) < d.cacheExpiry {
			return data
		}
	}
	
	return nil
}

func (d *databasePerformanceDashboard) cacheWidgetData(widgetID string, data *WidgetData) {
	d.cacheMutex.Lock()
	defer d.cacheMutex.Unlock()
	
	data.CacheInfo = &WidgetCacheInfo{
		IsCached:      true,
		CacheHit:      false,
		CacheExpiry:   time.Now().Add(d.cacheExpiry),
		CacheDuration: d.cacheExpiry,
		CacheKey:      widgetID,
	}
	
	d.dataCache[widgetID] = data
}

func (d *databasePerformanceDashboard) clearWidgetCache(widgetID string) {
	d.cacheMutex.Lock()
	defer d.cacheMutex.Unlock()
	
	delete(d.dataCache, widgetID)
}

func (d *databasePerformanceDashboard) clearDashboardCache(dashboardID string) {
	d.cacheMutex.Lock()
	defer d.cacheMutex.Unlock()
	
	// Clear cache for all widgets in the dashboard
	d.dashboardsMutex.RLock()
	if dashboard, exists := d.dashboards[dashboardID]; exists {
		for _, widget := range dashboard.Widgets {
			delete(d.dataCache, widget.WidgetID)
		}
	}
	d.dashboardsMutex.RUnlock()
}

// Utility methods

func (d *databasePerformanceDashboard) isWidgetExcluded(widgetID string, options *DashboardOptions) bool {
	if len(options.IncludeWidgets) > 0 {
		for _, includeID := range options.IncludeWidgets {
			if includeID == widgetID {
				return false
			}
		}
		return true
	}
	
	for _, excludeID := range options.ExcludeWidgets {
		if excludeID == widgetID {
			return true
		}
	}
	
	return false
}

func (d *databasePerformanceDashboard) calculateDashboardPerformance(dashboardData *DashboardData) *DashboardPerformance {
	if dashboardData.RefreshMetadata == nil {
		return nil
	}
	
	performance := &DashboardPerformance{
		LoadTime:         dashboardData.RefreshMetadata.Duration,
		QueryPerformance: make(map[string]time.Duration),
		DataTransferSize: 1024 * 100, // Mock data
		MemoryUsage:      1024 * 1024 * 10, // Mock data
		CacheEfficiency:  dashboardData.RefreshMetadata.CacheHitRate,
	}
	
	// Collect query performance from widgets
	for widgetID, widgetData := range dashboardData.WidgetsData {
		if widgetData.Metadata != nil {
			performance.QueryPerformance[widgetID] = widgetData.Metadata.QueryExecutionTime
		}
	}
	
	// Calculate render time (mock)
	performance.RenderTime = 50 * time.Millisecond
	
	// Add optimization tips
	if performance.LoadTime > 5*time.Second {
		performance.OptimizationTips = append(performance.OptimizationTips, "Consider reducing the number of widgets or data points")
	}
	if performance.CacheEfficiency < 0.8 {
		performance.OptimizationTips = append(performance.OptimizationTips, "Enable caching for better performance")
	}
	
	return performance
}

func (d *databasePerformanceDashboard) trackDashboardView(dashboardID string, timestamp time.Time) {
	d.analyticsMutex.Lock()
	defer d.analyticsMutex.Unlock()
	
	if _, exists := d.analyticsData[dashboardID]; !exists {
		d.analyticsData[dashboardID] = &DashboardAnalytics{
			DashboardID: dashboardID,
			ViewCount:   0,
			UniqueViewers: 0,
			GeneratedAt: time.Now(),
		}
	}
	
	d.analyticsData[dashboardID].ViewCount++
}

// Analysis and insights helper methods

func (d *databasePerformanceDashboard) calculateHealthScore(summary *MetricsSummary) float64 {
	if summary == nil {
		return 50.0
	}
	
	score := 100.0
	
	// Penalize high execution times
	if summary.AverageExecutionTime > 200*time.Millisecond {
		score -= 30
	}
	
	// Penalize high error rates
	if summary.ErrorRate > 0.05 {
		score -= 25
	}
	
	// Penalize low cache hit rates
	if summary.CacheHitRate < 0.8 {
		score -= 20
	}
	
	return max(score, 0)
}

func (d *databasePerformanceDashboard) countSlowQueries(metrics []*QueryExecutionMetric) int {
	count := 0
	threshold := 200 * time.Millisecond
	
	for _, metric := range metrics {
		if metric.ExecutionTime > threshold {
			count++
		}
	}
	
	return count
}

func (d *databasePerformanceDashboard) generatePerformanceRecommendations(summary *MetricsSummary) []*PerformanceRecommendation {
	recommendations := []*PerformanceRecommendation{}
	
	if summary.AverageExecutionTime > 200*time.Millisecond {
		recommendations = append(recommendations, &PerformanceRecommendation{
			Type:         "optimization",
			Priority:     "high",
			Title:        "Optimize Slow Queries",
			Description:  "Average execution time exceeds 200ms threshold",
			Impact:       "high",
			Effort:       "medium",
			ActionItems:  []string{"Identify slow queries", "Add appropriate indices", "Optimize query structure"},
			EstimatedBenefit: "30-70% performance improvement",
			RiskLevel:    "low",
		})
	}
	
	if summary.ErrorRate > 0.05 {
		recommendations = append(recommendations, &PerformanceRecommendation{
			Type:         "monitoring",
			Priority:     "high",
			Title:        "Reduce Error Rate",
			Description:  "Error rate exceeds 5% threshold",
			Impact:       "high",
			Effort:       "high",
			ActionItems:  []string{"Investigate error patterns", "Improve error handling", "Add monitoring alerts"},
			EstimatedBenefit: "Improved reliability and user experience",
			RiskLevel:    "medium",
		})
	}
	
	if summary.CacheHitRate < 0.8 {
		recommendations = append(recommendations, &PerformanceRecommendation{
			Type:         "configuration",
			Priority:     "medium",
			Title:        "Improve Cache Hit Rate",
			Description:  "Cache hit rate is below 80%",
			Impact:       "medium",
			Effort:       "low",
			ActionItems:  []string{"Tune cache configuration", "Analyze cache patterns", "Optimize cache warming"},
			EstimatedBenefit: "15-25% performance improvement",
			RiskLevel:    "low",
		})
	}
	
	return recommendations
}

func (d *databasePerformanceDashboard) generatePeriodComparison(ctx context.Context, currentTimeRange *TimeRange) *PeriodComparison {
	// Calculate previous period
	duration := currentTimeRange.End.Sub(currentTimeRange.Start)
	previousTimeRange := &TimeRange{
		Start: currentTimeRange.Start.Add(-duration),
		End:   currentTimeRange.Start,
	}
	
	// This would normally get actual previous period data
	// For now, return mock comparison
	return &PeriodComparison{
		PreviousPeriod: previousTimeRange,
		QueryTimeChange: &PercentageChange{
			Previous:   150.0,
			Current:    120.0,
			Change:     -30.0,
			Percentage: -20.0,
			IsPositive: true, // Lower is better for exec time
		},
		ThroughputChange: &PercentageChange{
			Previous:   100.0,
			Current:    110.0,
			Change:     10.0,
			Percentage: 10.0,
			IsPositive: true,
		},
		OverallImprovement: true,
	}
}

// Export helper methods

func (d *databasePerformanceDashboard) exportToHTML(dashboardData *DashboardData) ([]byte, error) {
	html := fmt.Sprintf(`
<!DOCTYPE html>
<html>
<head>
    <title>%s</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .dashboard { border: 1px solid #ddd; padding: 20px; margin: 10px 0; }
        .widget { border: 1px solid #eee; padding: 15px; margin: 10px 0; }
        .metric { display: inline-block; margin: 5px 10px; }
    </style>
</head>
<body>
    <h1>%s</h1>
    <p>Generated: %s</p>
    <div class="dashboard">
        <h2>Dashboard Widgets</h2>
`, 
		dashboardData.Dashboard.Name,
		dashboardData.Dashboard.Name,
		dashboardData.GeneratedAt.Format("2006-01-02 15:04:05"),
	)
	
	for widgetID, widgetData := range dashboardData.WidgetsData {
		html += fmt.Sprintf(`
        <div class="widget">
            <h3>Widget: %s</h3>
            <p>Last Updated: %s</p>
            <p>Data Points: %d</p>
        </div>
`, widgetID, widgetData.LastUpdated.Format("2006-01-02 15:04:05"), widgetData.Metadata.RecordCount)
	}
	
	html += `
    </div>
</body>
</html>`
	
	return []byte(html), nil
}

// Report generation helper methods

func (d *databasePerformanceDashboard) generateExecutiveSummary(dashboardData *DashboardData) string {
	summary := fmt.Sprintf("Dashboard '%s' contains %d widgets with data from %s to %s. ",
		dashboardData.Dashboard.Name,
		len(dashboardData.WidgetsData),
		dashboardData.DataTimeRange.Start.Format("2006-01-02 15:04"),
		dashboardData.DataTimeRange.End.Format("2006-01-02 15:04"),
	)
	
	if dashboardData.RefreshMetadata != nil {
		summary += fmt.Sprintf("Dashboard refresh completed in %v with %d widgets successfully loaded.",
			dashboardData.RefreshMetadata.Duration,
			dashboardData.RefreshMetadata.WidgetsRefreshed,
		)
	}
	
	return summary
}

func (d *databasePerformanceDashboard) generateSummarySection(dashboardData *DashboardData) *DashboardReportSection {
	return &DashboardReportSection{
		SectionID: "summary",
		Title:     "Dashboard Summary",
		Content:   d.generateExecutiveSummary(dashboardData),
		Metrics: map[string]interface{}{
			"total_widgets":      len(dashboardData.WidgetsData),
			"refresh_duration":   dashboardData.RefreshMetadata.Duration.String(),
			"widgets_refreshed":  dashboardData.RefreshMetadata.WidgetsRefreshed,
			"widgets_failed":     dashboardData.RefreshMetadata.WidgetsFailed,
		},
	}
}

func (d *databasePerformanceDashboard) generateAnalyticsSection(ctx context.Context, dashboardID string, timeRange *TimeRange) (*DashboardReportSection, error) {
	analytics, err := d.GetDashboardUsageAnalytics(ctx, dashboardID, timeRange)
	if err != nil {
		return nil, err
	}
	
	return &DashboardReportSection{
		SectionID: "analytics",
		Title:     "Usage Analytics",
		Content:   fmt.Sprintf("Dashboard viewed %d times by %d unique users", analytics.ViewCount, analytics.UniqueViewers),
		Metrics: map[string]interface{}{
			"view_count":       analytics.ViewCount,
			"unique_viewers":   analytics.UniqueViewers,
			"average_view_time": analytics.AverageViewTime.String(),
		},
	}, nil
}

func (d *databasePerformanceDashboard) generateInsightsSection(ctx context.Context, timeRange *TimeRange) (*DashboardReportSection, error) {
	insights, err := d.GetPerformanceInsights(ctx, timeRange)
	if err != nil {
		return nil, err
	}
	
	insightsText := fmt.Sprintf("Overall health score: %.1f/100. ", insights.OverallHealthScore)
	if insights.KeyMetrics != nil {
		insightsText += fmt.Sprintf("Average query time: %v. Error rate: %.2f%%.",
			insights.KeyMetrics.AverageQueryTime,
			insights.KeyMetrics.ErrorRate*100,
		)
	}
	
	return &DashboardReportSection{
		SectionID: "insights",
		Title:     "Performance Insights",
		Content:   insightsText,
		Metrics: map[string]interface{}{
			"health_score":       insights.OverallHealthScore,
			"recommendations":    len(insights.Recommendations),
			"anomalies_detected": len(insights.AnomaliesDetected),
		},
		Insights: d.formatRecommendationsAsInsights(insights.Recommendations),
	}, nil
}

func (d *databasePerformanceDashboard) formatRecommendationsAsInsights(recommendations []*PerformanceRecommendation) []string {
	insights := make([]string, len(recommendations))
	for i, rec := range recommendations {
		insights[i] = fmt.Sprintf("[%s] %s: %s", rec.Priority, rec.Title, rec.Description)
	}
	return insights
}

// Helper function
func max(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}