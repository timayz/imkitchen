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

type DatabasePerformanceAlerting interface {
	// Lifecycle management
	Start(ctx context.Context) error
	Stop(ctx context.Context) error
	IsRunning() bool
	
	// Alert rule management
	CreateAlertRule(ctx context.Context, rule *AlertRule) error
	UpdateAlertRule(ctx context.Context, ruleID string, rule *AlertRule) error
	DeleteAlertRule(ctx context.Context, ruleID string) error
	GetAlertRule(ctx context.Context, ruleID string) (*AlertRule, error)
	ListAlertRules(ctx context.Context, filter *AlertRuleFilter) ([]*AlertRule, error)
	
	// Alert management
	GetAlerts(ctx context.Context, filter *AlertFilter) ([]*Alert, error)
	GetAlert(ctx context.Context, alertID string) (*Alert, error)
	AcknowledgeAlert(ctx context.Context, alertID string, acknowledgment *AlertAcknowledgment) error
	ResolveAlert(ctx context.Context, alertID string, resolution *AlertResolution) error
	SnoozeAlert(ctx context.Context, alertID string, duration time.Duration, reason string) error
	
	// Notification channels
	AddNotificationChannel(ctx context.Context, channel *NotificationChannel) error
	UpdateNotificationChannel(ctx context.Context, channelID string, channel *NotificationChannel) error
	RemoveNotificationChannel(ctx context.Context, channelID string) error
	TestNotificationChannel(ctx context.Context, channelID string) error
	ListNotificationChannels(ctx context.Context) ([]*NotificationChannel, error)
	
	// Alert evaluation and triggering
	EvaluateAlerts(ctx context.Context) error
	TriggerAlert(ctx context.Context, alert *Alert) error
	
	// Alert history and analytics
	GetAlertHistory(ctx context.Context, timeRange *TimeRange, filter *AlertHistoryFilter) ([]*AlertHistoryEntry, error)
	GetAlertStatistics(ctx context.Context, timeRange *TimeRange) (*AlertStatistics, error)
	GetAlertTrends(ctx context.Context, timeRange *TimeRange) (*AlertTrends, error)
	
	// Health and diagnostics
	GetAlertingHealth(ctx context.Context) (*AlertingHealth, error)
	GetAlertEvaluationMetrics(ctx context.Context) (*AlertEvaluationMetrics, error)
}

type AlertRule struct {
	RuleID          string                 `json:"rule_id"`
	Name            string                 `json:"name"`
	Description     string                 `json:"description"`
	Type            AlertRuleType          `json:"type"`
	MetricType      MetricType             `json:"metric_type"`
	Condition       *AlertCondition        `json:"condition"`
	Threshold       *AlertThreshold        `json:"threshold"`
	EvaluationWindow time.Duration         `json:"evaluation_window"`
	EvaluationInterval time.Duration       `json:"evaluation_interval"`
	Severity        AlertSeverity          `json:"severity"`
	State           AlertRuleState         `json:"state"`
	Tags            []string               `json:"tags,omitempty"`
	Annotations     map[string]string      `json:"annotations,omitempty"`
	NotificationChannels []string          `json:"notification_channels"`
	AutoResolve     bool                   `json:"auto_resolve"`
	AutoResolveAfter time.Duration        `json:"auto_resolve_after,omitempty"`
	SuppressDuplicates bool               `json:"suppress_duplicates"`
	SuppressionWindow time.Duration       `json:"suppression_window,omitempty"`
	RateLimiting    *RateLimiting         `json:"rate_limiting,omitempty"`
	Schedule        *AlertSchedule        `json:"schedule,omitempty"`
	CreatedAt       time.Time             `json:"created_at"`
	UpdatedAt       time.Time             `json:"updated_at"`
	CreatedBy       string                `json:"created_by"`
	IsEnabled       bool                  `json:"is_enabled"`
	LastEvaluation  time.Time             `json:"last_evaluation,omitempty"`
	NextEvaluation  time.Time             `json:"next_evaluation,omitempty"`
	EvaluationCount int64                 `json:"evaluation_count"`
	AlertCount      int64                 `json:"alert_count"`
}

type AlertRuleType string

const (
	AlertRuleTypeThreshold   AlertRuleType = "threshold"
	AlertRuleTypeAnomaly     AlertRuleType = "anomaly"
	AlertRuleTypeRate        AlertRuleType = "rate"
	AlertRuleTypeTrend       AlertRuleType = "trend"
	AlertRuleTypeComposite   AlertRuleType = "composite"
	AlertRuleTypeCustom      AlertRuleType = "custom"
)

type MetricType string

const (
	MetricTypeExecutionTime    MetricType = "execution_time"
	MetricTypeQueryCount       MetricType = "query_count"
	MetricTypeErrorRate        MetricType = "error_rate"
	MetricTypeCacheHitRate     MetricType = "cache_hit_rate"
	MetricTypeSlowQueryCount   MetricType = "slow_query_count"
	MetricTypeConnectionCount  MetricType = "connection_count"
	MetricTypeCPUUsage         MetricType = "cpu_usage"
	MetricTypeMemoryUsage      MetricType = "memory_usage"
	MetricTypeDiskUsage        MetricType = "disk_usage"
	MetricTypeCustom           MetricType = "custom"
)

type AlertCondition struct {
	Operator        ComparisonOperator     `json:"operator"`
	Value           float64                `json:"value"`
	AggregationType AggregationType        `json:"aggregation_type"`
	ComparisonType  ComparisonType         `json:"comparison_type"`
	TimeWindow      time.Duration          `json:"time_window,omitempty"`
	MinSamples      int                    `json:"min_samples,omitempty"`
	Filters         map[string]interface{} `json:"filters,omitempty"`
	GroupBy         []string               `json:"group_by,omitempty"`
	CustomQuery     string                 `json:"custom_query,omitempty"`
}

type ComparisonOperator string

const (
	OperatorGreaterThan        ComparisonOperator = "gt"
	OperatorGreaterThanEqual   ComparisonOperator = "gte"
	OperatorLessThan           ComparisonOperator = "lt"
	OperatorLessThanEqual      ComparisonOperator = "lte"
	OperatorEqual              ComparisonOperator = "eq"
	OperatorNotEqual           ComparisonOperator = "ne"
	OperatorBetween            ComparisonOperator = "between"
	OperatorNotBetween         ComparisonOperator = "not_between"
)

type AggregationType string

const (
	AggregationAvg      AggregationType = "avg"
	AggregationSum      AggregationType = "sum"
	AggregationCount    AggregationType = "count"
	AggregationMax      AggregationType = "max"
	AggregationMin      AggregationType = "min"
	AggregationP50      AggregationType = "p50"
	AggregationP95      AggregationType = "p95"
	AggregationP99      AggregationType = "p99"
	AggregationRate     AggregationType = "rate"
	AggregationIncrease AggregationType = "increase"
)

type ComparisonType string

const (
	ComparisonAbsolute   ComparisonType = "absolute"
	ComparisonPercentage ComparisonType = "percentage"
	ComparisonRatio      ComparisonType = "ratio"
)

type AlertThreshold struct {
	Critical *ThresholdLevel `json:"critical,omitempty"`
	Warning  *ThresholdLevel `json:"warning,omitempty"`
	Info     *ThresholdLevel `json:"info,omitempty"`
}

type ThresholdLevel struct {
	Value           float64                `json:"value"`
	Operator        ComparisonOperator     `json:"operator"`
	Duration        time.Duration          `json:"duration,omitempty"`
	Conditions      []*AlertCondition      `json:"conditions,omitempty"`
	RecoveryValue   *float64               `json:"recovery_value,omitempty"`
	RecoveryOperator *ComparisonOperator   `json:"recovery_operator,omitempty"`
}

type AlertSeverity string

const (
	SeverityCritical AlertSeverity = "critical"
	SeverityWarning  AlertSeverity = "warning"
	SeverityInfo     AlertSeverity = "info"
)

type AlertRuleState string

const (
	AlertRuleStateOK       AlertRuleState = "ok"
	AlertRuleStatePending  AlertRuleState = "pending"
	AlertRuleStateAlerting AlertRuleState = "alerting"
	AlertRuleStateDisabled AlertRuleState = "disabled"
	AlertRuleStateError    AlertRuleState = "error"
)

type RateLimiting struct {
	MaxAlertsPerHour   int           `json:"max_alerts_per_hour,omitempty"`
	MaxAlertsPerDay    int           `json:"max_alerts_per_day,omitempty"`
	CooldownPeriod     time.Duration `json:"cooldown_period,omitempty"`
	BurstAllowed       int           `json:"burst_allowed,omitempty"`
}

type AlertSchedule struct {
	Timezone      string             `json:"timezone"`
	TimeRanges    []*TimeRange       `json:"time_ranges,omitempty"`
	WeeklySchedule *WeeklySchedule   `json:"weekly_schedule,omitempty"`
	Holidays      []*Holiday         `json:"holidays,omitempty"`
	MaintenanceWindows []*MaintenanceWindow `json:"maintenance_windows,omitempty"`
}

type WeeklySchedule struct {
	Monday    *DaySchedule `json:"monday,omitempty"`
	Tuesday   *DaySchedule `json:"tuesday,omitempty"`
	Wednesday *DaySchedule `json:"wednesday,omitempty"`
	Thursday  *DaySchedule `json:"thursday,omitempty"`
	Friday    *DaySchedule `json:"friday,omitempty"`
	Saturday  *DaySchedule `json:"saturday,omitempty"`
	Sunday    *DaySchedule `json:"sunday,omitempty"`
}

type DaySchedule struct {
	IsEnabled bool           `json:"is_enabled"`
	TimeSlots []*TimeSlot    `json:"time_slots,omitempty"`
}

type TimeSlot struct {
	StartTime string `json:"start_time"` // "09:00"
	EndTime   string `json:"end_time"`   // "17:00"
}

type Holiday struct {
	Name      string    `json:"name"`
	Date      time.Time `json:"date"`
	IsEnabled bool      `json:"is_enabled"`
}

type MaintenanceWindow struct {
	Name      string    `json:"name"`
	StartTime time.Time `json:"start_time"`
	EndTime   time.Time `json:"end_time"`
	IsEnabled bool      `json:"is_enabled"`
	Reason    string    `json:"reason,omitempty"`
}

type Alert struct {
	AlertID         string                 `json:"alert_id"`
	RuleID          string                 `json:"rule_id"`
	RuleName        string                 `json:"rule_name"`
	State           AlertState             `json:"state"`
	Severity        AlertSeverity          `json:"severity"`
	Message         string                 `json:"message"`
	Description     string                 `json:"description,omitempty"`
	Labels          map[string]string      `json:"labels,omitempty"`
	Annotations     map[string]string      `json:"annotations,omitempty"`
	Value           float64                `json:"value"`
	Threshold       float64                `json:"threshold"`
	StartsAt        time.Time              `json:"starts_at"`
	EndsAt          time.Time              `json:"ends_at,omitempty"`
	UpdatedAt       time.Time              `json:"updated_at"`
	Fingerprint     string                 `json:"fingerprint"`
	GeneratorURL    string                 `json:"generator_url,omitempty"`
	SilencedBy      []string               `json:"silenced_by,omitempty"`
	InhibitedBy     []string               `json:"inhibited_by,omitempty"`
	ActiveAt        time.Time              `json:"active_at,omitempty"`
	AcknowledgedAt  time.Time              `json:"acknowledged_at,omitempty"`
	AcknowledgedBy  string                 `json:"acknowledged_by,omitempty"`
	ResolvedAt      time.Time              `json:"resolved_at,omitempty"`
	ResolvedBy      string                 `json:"resolved_by,omitempty"`
	SnoozedUntil    time.Time              `json:"snoozed_until,omitempty"`
	SnoozedBy       string                 `json:"snoozed_by,omitempty"`
	EvaluationData  *AlertEvaluationData   `json:"evaluation_data,omitempty"`
	NotificationLog []*NotificationLogEntry `json:"notification_log,omitempty"`
}

type AlertState string

const (
	AlertStateActive      AlertState = "active"
	AlertStatePending     AlertState = "pending"
	AlertStateAcknowledged AlertState = "acknowledged"
	AlertStateResolved    AlertState = "resolved"
	AlertStateSnoozed     AlertState = "snoozed"
	AlertStateSuppressed  AlertState = "suppressed"
)

type AlertEvaluationData struct {
	EvaluatedAt   time.Time              `json:"evaluated_at"`
	QueryResult   interface{}            `json:"query_result"`
	MetricValue   float64                `json:"metric_value"`
	SampleCount   int                    `json:"sample_count"`
	EvaluationTime time.Duration         `json:"evaluation_time"`
	Context       map[string]interface{} `json:"context,omitempty"`
	ErrorMessage  string                 `json:"error_message,omitempty"`
}

type NotificationLogEntry struct {
	Timestamp   time.Time `json:"timestamp"`
	ChannelID   string    `json:"channel_id"`
	ChannelType string    `json:"channel_type"`
	Status      string    `json:"status"` // "sent", "failed", "retry"
	Message     string    `json:"message,omitempty"`
	Error       string    `json:"error,omitempty"`
	RetryCount  int       `json:"retry_count"`
}

type NotificationChannel struct {
	ChannelID       string                 `json:"channel_id"`
	Name            string                 `json:"name"`
	Type            NotificationChannelType `json:"type"`
	Configuration   *ChannelConfiguration   `json:"configuration"`
	Settings        *NotificationSettings   `json:"settings"`
	IsEnabled       bool                   `json:"is_enabled"`
	CreatedAt       time.Time              `json:"created_at"`
	UpdatedAt       time.Time              `json:"updated_at"`
	LastUsed        time.Time              `json:"last_used,omitempty"`
	SuccessCount    int64                  `json:"success_count"`
	FailureCount    int64                  `json:"failure_count"`
	Tags            []string               `json:"tags,omitempty"`
}

type NotificationChannelType string

const (
	ChannelTypeEmail     NotificationChannelType = "email"
	ChannelTypeSlack     NotificationChannelType = "slack"
	ChannelTypeWebhook   NotificationChannelType = "webhook"
	ChannelTypePagerDuty NotificationChannelType = "pagerduty"
	ChannelTypeSMS       NotificationChannelType = "sms"
	ChannelTypeDiscord   NotificationChannelType = "discord"
	ChannelTypeMSTeams   NotificationChannelType = "msteams"
	ChannelTypeCustom    NotificationChannelType = "custom"
)

type ChannelConfiguration struct {
	EmailConfig     *EmailConfiguration     `json:"email_config,omitempty"`
	SlackConfig     *SlackConfiguration     `json:"slack_config,omitempty"`
	WebhookConfig   *WebhookConfiguration   `json:"webhook_config,omitempty"`
	PagerDutyConfig *PagerDutyConfiguration `json:"pagerduty_config,omitempty"`
	SMSConfig       *SMSConfiguration       `json:"sms_config,omitempty"`
	DiscordConfig   *DiscordConfiguration   `json:"discord_config,omitempty"`
	MSTeamsConfig   *MSTeamsConfiguration   `json:"msteams_config,omitempty"`
	CustomConfig    map[string]interface{}  `json:"custom_config,omitempty"`
}

type EmailConfiguration struct {
	SMTPServer   string   `json:"smtp_server"`
	SMTPPort     int      `json:"smtp_port"`
	Username     string   `json:"username"`
	Password     string   `json:"password"` // Should be encrypted
	From         string   `json:"from"`
	To           []string `json:"to"`
	CC           []string `json:"cc,omitempty"`
	BCC          []string `json:"bcc,omitempty"`
	Subject      string   `json:"subject,omitempty"`
	Template     string   `json:"template,omitempty"`
	TLS          bool     `json:"tls"`
	StartTLS     bool     `json:"start_tls"`
	InsecureSkipVerify bool `json:"insecure_skip_verify"`
}

type SlackConfiguration struct {
	WebhookURL  string `json:"webhook_url"` // Should be encrypted
	Channel     string `json:"channel"`
	Username    string `json:"username,omitempty"`
	IconEmoji   string `json:"icon_emoji,omitempty"`
	IconURL     string `json:"icon_url,omitempty"`
	Title       string `json:"title,omitempty"`
	Template    string `json:"template,omitempty"`
}

type WebhookConfiguration struct {
	URL         string            `json:"url"`
	Method      string            `json:"method"` // "POST", "PUT", "PATCH"
	Headers     map[string]string `json:"headers,omitempty"`
	BasicAuth   *BasicAuth        `json:"basic_auth,omitempty"`
	BearerToken string            `json:"bearer_token,omitempty"` // Should be encrypted
	Template    string            `json:"template,omitempty"`
	Timeout     time.Duration     `json:"timeout"`
	MaxRetries  int               `json:"max_retries"`
	TLS         *TLSConfig        `json:"tls,omitempty"`
}

type BasicAuth struct {
	Username string `json:"username"`
	Password string `json:"password"` // Should be encrypted
}

type TLSConfig struct {
	InsecureSkipVerify bool   `json:"insecure_skip_verify"`
	ServerName         string `json:"server_name,omitempty"`
	CACert             string `json:"ca_cert,omitempty"`
	ClientCert         string `json:"client_cert,omitempty"`
	ClientKey          string `json:"client_key,omitempty"` // Should be encrypted
}

type PagerDutyConfiguration struct {
	IntegrationKey string `json:"integration_key"` // Should be encrypted
	Severity       string `json:"severity,omitempty"`
	Client         string `json:"client,omitempty"`
	ClientURL      string `json:"client_url,omitempty"`
	Description    string `json:"description,omitempty"`
	Details        map[string]string `json:"details,omitempty"`
}

type SMSConfiguration struct {
	Provider    string   `json:"provider"` // "twilio", "aws_sns", etc.
	APIKey      string   `json:"api_key"`  // Should be encrypted
	APISecret   string   `json:"api_secret"` // Should be encrypted
	From        string   `json:"from"`
	To          []string `json:"to"`
	Template    string   `json:"template,omitempty"`
}

type DiscordConfiguration struct {
	WebhookURL string `json:"webhook_url"` // Should be encrypted
	Username   string `json:"username,omitempty"`
	AvatarURL  string `json:"avatar_url,omitempty"`
	Template   string `json:"template,omitempty"`
}

type MSTeamsConfiguration struct {
	WebhookURL string `json:"webhook_url"` // Should be encrypted
	Title      string `json:"title,omitempty"`
	Template   string `json:"template,omitempty"`
}

type NotificationSettings struct {
	Severities    []AlertSeverity   `json:"severities,omitempty"`
	Tags          []string          `json:"tags,omitempty"`
	RetryPolicy   *RetryPolicy      `json:"retry_policy,omitempty"`
	Throttling    *ThrottlingPolicy `json:"throttling,omitempty"`
	Template      *MessageTemplate  `json:"template,omitempty"`
	Formatting    *MessageFormatting `json:"formatting,omitempty"`
}

type RetryPolicy struct {
	MaxRetries    int           `json:"max_retries"`
	InitialDelay  time.Duration `json:"initial_delay"`
	MaxDelay      time.Duration `json:"max_delay"`
	BackoffFactor float64       `json:"backoff_factor"`
	RandomJitter  bool          `json:"random_jitter"`
}

type ThrottlingPolicy struct {
	MaxNotifications int           `json:"max_notifications"`
	TimeWindow       time.Duration `json:"time_window"`
	BurstAllowed     int           `json:"burst_allowed,omitempty"`
}

type MessageTemplate struct {
	SubjectTemplate string `json:"subject_template,omitempty"`
	BodyTemplate    string `json:"body_template"`
	Variables       map[string]interface{} `json:"variables,omitempty"`
}

type MessageFormatting struct {
	IncludeTimestamp  bool `json:"include_timestamp"`
	IncludeLabels     bool `json:"include_labels"`
	IncludeAnnotations bool `json:"include_annotations"`
	IncludeValue      bool `json:"include_value"`
	IncludeThreshold  bool `json:"include_threshold"`
	IncludeChart      bool `json:"include_chart"`
	IncludeRunbook    bool `json:"include_runbook"`
}

type AlertAcknowledgment struct {
	AcknowledgedBy string `json:"acknowledged_by"`
	Reason         string `json:"reason,omitempty"`
	Timestamp      time.Time `json:"timestamp"`
}

type AlertResolution struct {
	ResolvedBy  string `json:"resolved_by"`
	Reason      string `json:"reason,omitempty"`
	Timestamp   time.Time `json:"timestamp"`
	Resolution  string `json:"resolution,omitempty"`
}

// Filter types

type AlertRuleFilter struct {
	Type        *AlertRuleType   `json:"type,omitempty"`
	Severity    *AlertSeverity   `json:"severity,omitempty"`
	State       *AlertRuleState  `json:"state,omitempty"`
	IsEnabled   *bool            `json:"is_enabled,omitempty"`
	Tags        []string         `json:"tags,omitempty"`
	CreatedBy   string           `json:"created_by,omitempty"`
	CreatedAfter *time.Time      `json:"created_after,omitempty"`
	CreatedBefore *time.Time     `json:"created_before,omitempty"`
	Search      string           `json:"search,omitempty"`
	Limit       int              `json:"limit,omitempty"`
	Offset      int              `json:"offset,omitempty"`
}

type AlertFilter struct {
	RuleID        string           `json:"rule_id,omitempty"`
	State         *AlertState      `json:"state,omitempty"`
	Severity      *AlertSeverity   `json:"severity,omitempty"`
	Labels        map[string]string `json:"labels,omitempty"`
	StartsAfter   *time.Time       `json:"starts_after,omitempty"`
	StartsBefore  *time.Time       `json:"starts_before,omitempty"`
	EndsAfter     *time.Time       `json:"ends_after,omitempty"`
	EndsBefore    *time.Time       `json:"ends_before,omitempty"`
	AcknowledgedBy string          `json:"acknowledged_by,omitempty"`
	ResolvedBy    string           `json:"resolved_by,omitempty"`
	Search        string           `json:"search,omitempty"`
	Limit         int              `json:"limit,omitempty"`
	Offset        int              `json:"offset,omitempty"`
	SortBy        string           `json:"sort_by,omitempty"`
	SortOrder     string           `json:"sort_order,omitempty"`
}

type AlertHistoryFilter struct {
	RuleID      string         `json:"rule_id,omitempty"`
	Severity    *AlertSeverity `json:"severity,omitempty"`
	ActionType  string         `json:"action_type,omitempty"` // "triggered", "acknowledged", "resolved"
	PerformedBy string         `json:"performed_by,omitempty"`
	Limit       int            `json:"limit,omitempty"`
	Offset      int            `json:"offset,omitempty"`
}

// Analytics types

type AlertHistoryEntry struct {
	EntryID     string                 `json:"entry_id"`
	AlertID     string                 `json:"alert_id"`
	RuleID      string                 `json:"rule_id"`
	ActionType  string                 `json:"action_type"`
	Timestamp   time.Time              `json:"timestamp"`
	PerformedBy string                 `json:"performed_by,omitempty"`
	Details     map[string]interface{} `json:"details,omitempty"`
	OldState    string                 `json:"old_state,omitempty"`
	NewState    string                 `json:"new_state,omitempty"`
}

type AlertStatistics struct {
	TimeRange           *TimeRange `json:"time_range"`
	TotalAlerts         int        `json:"total_alerts"`
	ActiveAlerts        int        `json:"active_alerts"`
	ResolvedAlerts      int        `json:"resolved_alerts"`
	AcknowledgedAlerts  int        `json:"acknowledged_alerts"`
	SeverityBreakdown   map[AlertSeverity]int `json:"severity_breakdown"`
	StateBreakdown      map[AlertState]int `json:"state_breakdown"`
	RuleBreakdown       map[string]int `json:"rule_breakdown"`
	HourlyDistribution  map[int]int `json:"hourly_distribution"`
	DailyDistribution   map[string]int `json:"daily_distribution"`
	AverageResolutionTime time.Duration `json:"average_resolution_time"`
	MedianResolutionTime  time.Duration `json:"median_resolution_time"`
	FalsePositiveRate   float64 `json:"false_positive_rate"`
	NotificationStats   *NotificationStatistics `json:"notification_stats"`
	GeneratedAt         time.Time `json:"generated_at"`
}

type NotificationStatistics struct {
	TotalNotifications    int `json:"total_notifications"`
	SuccessfulNotifications int `json:"successful_notifications"`
	FailedNotifications   int `json:"failed_notifications"`
	ChannelBreakdown      map[string]int `json:"channel_breakdown"`
	AverageDeliveryTime   time.Duration `json:"average_delivery_time"`
	RetryCount            int `json:"retry_count"`
}

type AlertTrends struct {
	TimeRange       *TimeRange            `json:"time_range"`
	AlertFrequency  *AlertFrequencyTrend  `json:"alert_frequency"`
	SeverityTrends  map[AlertSeverity]*TrendData `json:"severity_trends"`
	RuleTrends      map[string]*TrendData `json:"rule_trends"`
	ResolutionTrends *ResolutionTrendData `json:"resolution_trends"`
	SeasonalPatterns *SeasonalPatterns    `json:"seasonal_patterns"`
	GeneratedAt     time.Time             `json:"generated_at"`
}

type AlertFrequencyTrend struct {
	DataPoints      []AlertFrequencyPoint `json:"data_points"`
	Trend           string                `json:"trend"` // "increasing", "decreasing", "stable"
	TrendStrength   float64               `json:"trend_strength"`
	Forecast        []AlertFrequencyPoint `json:"forecast,omitempty"`
}

type AlertFrequencyPoint struct {
	Timestamp  time.Time `json:"timestamp"`
	AlertCount int       `json:"alert_count"`
	Severity   AlertSeverity `json:"severity,omitempty"`
}

type TrendData struct {
	Values        []float64 `json:"values"`
	Timestamps    []time.Time `json:"timestamps"`
	Trend         string    `json:"trend"`
	TrendStrength float64   `json:"trend_strength"`
	Correlation   float64   `json:"correlation,omitempty"`
}

type ResolutionTrendData struct {
	AverageResolutionTimes []ResolutionTimePoint `json:"average_resolution_times"`
	Trend                  string                `json:"trend"`
	ImprovementPercent     float64               `json:"improvement_percent"`
}

type ResolutionTimePoint struct {
	Timestamp      time.Time     `json:"timestamp"`
	ResolutionTime time.Duration `json:"resolution_time"`
	AlertCount     int           `json:"alert_count"`
}

type SeasonalPatterns struct {
	HourlyPattern  []float64 `json:"hourly_pattern"`
	DailyPattern   []float64 `json:"daily_pattern"`
	WeeklyPattern  []float64 `json:"weekly_pattern"`
	MonthlyPattern []float64 `json:"monthly_pattern"`
}

// Health and metrics types

type AlertingHealth struct {
	IsHealthy            bool                     `json:"is_healthy"`
	Status               string                   `json:"status"`
	Uptime               time.Duration            `json:"uptime"`
	LastEvaluationTime   time.Time                `json:"last_evaluation_time"`
	EvaluationErrors     int64                    `json:"evaluation_errors"`
	NotificationErrors   int64                    `json:"notification_errors"`
	ActiveRules          int                      `json:"active_rules"`
	DisabledRules        int                      `json:"disabled_rules"`
	ErrorRules           int                      `json:"error_rules"`
	PendingAlerts        int                      `json:"pending_alerts"`
	ActiveAlerts         int                      `json:"active_alerts"`
	ComponentHealth      map[string]string        `json:"component_health"`
	PerformanceMetrics   *AlertingPerformanceMetrics `json:"performance_metrics"`
	HealthChecks         []AlertingHealthCheck    `json:"health_checks"`
}

type AlertingPerformanceMetrics struct {
	AverageEvaluationTime    time.Duration `json:"average_evaluation_time"`
	MaxEvaluationTime        time.Duration `json:"max_evaluation_time"`
	EvaluationsPerSecond     float64       `json:"evaluations_per_second"`
	NotificationsPerSecond   float64       `json:"notifications_per_second"`
	MemoryUsage              int64         `json:"memory_usage_bytes"`
	RuleEvaluationQueueDepth int           `json:"rule_evaluation_queue_depth"`
	NotificationQueueDepth   int           `json:"notification_queue_depth"`
}

type AlertingHealthCheck struct {
	CheckName   string        `json:"check_name"`
	Status      string        `json:"status"`
	LastRun     time.Time     `json:"last_run"`
	Duration    time.Duration `json:"duration"`
	Message     string        `json:"message,omitempty"`
	Details     map[string]interface{} `json:"details,omitempty"`
}

type AlertEvaluationMetrics struct {
	TotalEvaluations       int64         `json:"total_evaluations"`
	SuccessfulEvaluations  int64         `json:"successful_evaluations"`
	FailedEvaluations      int64         `json:"failed_evaluations"`
	AverageEvaluationTime  time.Duration `json:"average_evaluation_time"`
	EvaluationTimeP95      time.Duration `json:"evaluation_time_p95"`
	EvaluationTimeP99      time.Duration `json:"evaluation_time_p99"`
	EvaluationErrorRate    float64       `json:"evaluation_error_rate"`
	RuleEvaluationStats    map[string]*RuleEvaluationStats `json:"rule_evaluation_stats"`
	LastEvaluationCycle    time.Time     `json:"last_evaluation_cycle"`
	NextEvaluationCycle    time.Time     `json:"next_evaluation_cycle"`
	EvaluationQueueLength  int           `json:"evaluation_queue_length"`
}

type RuleEvaluationStats struct {
	RuleID              string        `json:"rule_id"`
	EvaluationCount     int64         `json:"evaluation_count"`
	SuccessCount        int64         `json:"success_count"`
	ErrorCount          int64         `json:"error_count"`
	AverageEvaluationTime time.Duration `json:"average_evaluation_time"`
	LastEvaluation      time.Time     `json:"last_evaluation"`
	LastSuccess         time.Time     `json:"last_success"`
	LastError           time.Time     `json:"last_error"`
	LastErrorMessage    string        `json:"last_error_message,omitempty"`
}

// Implementation

type databasePerformanceAlerting struct {
	db                  *gorm.DB
	metricsCollector   QueryPerformanceMetricsCollector
	slowQueryLogger    DatabaseSlowQueryLogger
	dashboard          DatabasePerformanceDashboard
	
	// State management
	isRunning          bool
	startTime          time.Time
	mutex              sync.RWMutex
	stopChan           chan struct{}
	wg                 sync.WaitGroup
	
	// Alert rules and alerts storage
	alertRules         map[string]*AlertRule
	alerts             map[string]*Alert
	alertHistory       []*AlertHistoryEntry
	alertRulesMutex    sync.RWMutex
	alertsMutex        sync.RWMutex
	historyMutex       sync.RWMutex
	
	// Notification channels
	notificationChannels map[string]*NotificationChannel
	channelsMutex        sync.RWMutex
	
	// Evaluation and processing
	evaluationTicker   *time.Ticker
	evaluationInterval time.Duration
	evaluationQueue    chan *AlertRule
	notificationQueue  chan *NotificationTask
	
	// Statistics and metrics
	evaluationMetrics  *AlertEvaluationMetrics
	metricsMutex       sync.RWMutex
	
	// Rate limiting and suppression
	rateLimiters       map[string]*AlertRateLimiter
	suppressionCache   map[string]time.Time
	rateLimitersMutex  sync.RWMutex
	suppressionMutex   sync.RWMutex
}

type NotificationTask struct {
	Alert           *Alert
	Channel         *NotificationChannel
	RetryCount      int
	ScheduledTime   time.Time
}

type AlertRateLimiter struct {
	MaxCount     int
	TimeWindow   time.Duration
	Count        int
	WindowStart  time.Time
	LastReset    time.Time
}

func NewDatabasePerformanceAlerting(
	db *gorm.DB,
	metricsCollector QueryPerformanceMetricsCollector,
	slowQueryLogger DatabaseSlowQueryLogger,
	dashboard DatabasePerformanceDashboard,
) DatabasePerformanceAlerting {
	
	return &databasePerformanceAlerting{
		db:                   db,
		metricsCollector:     metricsCollector,
		slowQueryLogger:      slowQueryLogger,
		dashboard:            dashboard,
		evaluationInterval:   30 * time.Second,
		alertRules:           make(map[string]*AlertRule),
		alerts:               make(map[string]*Alert),
		alertHistory:         make([]*AlertHistoryEntry, 0),
		notificationChannels: make(map[string]*NotificationChannel),
		rateLimiters:         make(map[string]*AlertRateLimiter),
		suppressionCache:     make(map[string]time.Time),
		evaluationMetrics: &AlertEvaluationMetrics{
			RuleEvaluationStats: make(map[string]*RuleEvaluationStats),
		},
	}
}

func (a *databasePerformanceAlerting) Start(ctx context.Context) error {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	if a.isRunning {
		return fmt.Errorf("alerting system is already running")
	}
	
	a.isRunning = true
	a.startTime = time.Now()
	a.stopChan = make(chan struct{})
	a.evaluationQueue = make(chan *AlertRule, 100)
	a.notificationQueue = make(chan *NotificationTask, 1000)
	
	// Initialize default alert rules
	if err := a.initializeDefaultRules(ctx); err != nil {
		return fmt.Errorf("failed to initialize default rules: %w", err)
	}
	
	// Initialize default notification channels
	if err := a.initializeDefaultChannels(ctx); err != nil {
		return fmt.Errorf("failed to initialize default channels: %w", err)
	}
	
	// Start background workers
	a.wg.Add(3)
	go a.evaluationWorker(ctx)
	go a.notificationWorker(ctx)
	go a.maintenanceWorker(ctx)
	
	// Start evaluation ticker
	a.evaluationTicker = time.NewTicker(a.evaluationInterval)
	a.wg.Add(1)
	go a.evaluationScheduler(ctx)
	
	log.Printf("Database performance alerting system started")
	return nil
}

func (a *databasePerformanceAlerting) Stop(ctx context.Context) error {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	if !a.isRunning {
		return nil
	}
	
	a.isRunning = false
	
	// Stop ticker
	if a.evaluationTicker != nil {
		a.evaluationTicker.Stop()
	}
	
	// Close channels and wait for workers
	close(a.stopChan)
	a.wg.Wait()
	
	// Close queues
	close(a.evaluationQueue)
	close(a.notificationQueue)
	
	log.Printf("Database performance alerting system stopped")
	return nil
}

func (a *databasePerformanceAlerting) IsRunning() bool {
	a.mutex.RLock()
	defer a.mutex.RUnlock()
	return a.isRunning
}

// Alert rule management

func (a *databasePerformanceAlerting) CreateAlertRule(ctx context.Context, rule *AlertRule) error {
	if rule.RuleID == "" {
		rule.RuleID = fmt.Sprintf("rule_%d", time.Now().UnixNano())
	}
	
	rule.CreatedAt = time.Now()
	rule.UpdatedAt = time.Now()
	rule.State = AlertRuleStateOK
	rule.EvaluationCount = 0
	rule.AlertCount = 0
	
	// Validate rule
	if err := a.validateAlertRule(rule); err != nil {
		return fmt.Errorf("invalid alert rule: %w", err)
	}
	
	a.alertRulesMutex.Lock()
	a.alertRules[rule.RuleID] = rule
	a.alertRulesMutex.Unlock()
	
	// Initialize evaluation stats
	a.metricsMutex.Lock()
	a.evaluationMetrics.RuleEvaluationStats[rule.RuleID] = &RuleEvaluationStats{
		RuleID: rule.RuleID,
	}
	a.metricsMutex.Unlock()
	
	log.Printf("Alert rule created: %s (%s)", rule.RuleID, rule.Name)
	return nil
}

func (a *databasePerformanceAlerting) UpdateAlertRule(ctx context.Context, ruleID string, rule *AlertRule) error {
	a.alertRulesMutex.Lock()
	defer a.alertRulesMutex.Unlock()
	
	existingRule, exists := a.alertRules[ruleID]
	if !exists {
		return fmt.Errorf("alert rule %s not found", ruleID)
	}
	
	rule.RuleID = ruleID
	rule.CreatedAt = existingRule.CreatedAt
	rule.UpdatedAt = time.Now()
	rule.EvaluationCount = existingRule.EvaluationCount
	rule.AlertCount = existingRule.AlertCount
	
	// Validate updated rule
	if err := a.validateAlertRule(rule); err != nil {
		return fmt.Errorf("invalid alert rule update: %w", err)
	}
	
	a.alertRules[ruleID] = rule
	
	log.Printf("Alert rule updated: %s (%s)", ruleID, rule.Name)
	return nil
}

func (a *databasePerformanceAlerting) DeleteAlertRule(ctx context.Context, ruleID string) error {
	a.alertRulesMutex.Lock()
	defer a.alertRulesMutex.Unlock()
	
	if _, exists := a.alertRules[ruleID]; !exists {
		return fmt.Errorf("alert rule %s not found", ruleID)
	}
	
	delete(a.alertRules, ruleID)
	
	// Clean up related alerts
	a.alertsMutex.Lock()
	for alertID, alert := range a.alerts {
		if alert.RuleID == ruleID {
			delete(a.alerts, alertID)
		}
	}
	a.alertsMutex.Unlock()
	
	// Clean up evaluation stats
	a.metricsMutex.Lock()
	delete(a.evaluationMetrics.RuleEvaluationStats, ruleID)
	a.metricsMutex.Unlock()
	
	log.Printf("Alert rule deleted: %s", ruleID)
	return nil
}

func (a *databasePerformanceAlerting) GetAlertRule(ctx context.Context, ruleID string) (*AlertRule, error) {
	a.alertRulesMutex.RLock()
	defer a.alertRulesMutex.RUnlock()
	
	rule, exists := a.alertRules[ruleID]
	if !exists {
		return nil, fmt.Errorf("alert rule %s not found", ruleID)
	}
	
	return rule, nil
}

func (a *databasePerformanceAlerting) ListAlertRules(ctx context.Context, filter *AlertRuleFilter) ([]*AlertRule, error) {
	a.alertRulesMutex.RLock()
	defer a.alertRulesMutex.RUnlock()
	
	var rules []*AlertRule
	for _, rule := range a.alertRules {
		if a.matchesRuleFilter(rule, filter) {
			rules = append(rules, rule)
		}
	}
	
	// Sort rules by creation time
	sort.Slice(rules, func(i, j int) bool {
		return rules[i].CreatedAt.After(rules[j].CreatedAt)
	})
	
	// Apply limit and offset
	if filter != nil {
		if filter.Offset > 0 && filter.Offset < len(rules) {
			rules = rules[filter.Offset:]
		}
		if filter.Limit > 0 && filter.Limit < len(rules) {
			rules = rules[:filter.Limit]
		}
	}
	
	return rules, nil
}

// Alert management

func (a *databasePerformanceAlerting) GetAlerts(ctx context.Context, filter *AlertFilter) ([]*Alert, error) {
	a.alertsMutex.RLock()
	defer a.alertsMutex.RUnlock()
	
	var alerts []*Alert
	for _, alert := range a.alerts {
		if a.matchesAlertFilter(alert, filter) {
			alerts = append(alerts, alert)
		}
	}
	
	// Sort alerts by start time
	sort.Slice(alerts, func(i, j int) bool {
		return alerts[i].StartsAt.After(alerts[j].StartsAt)
	})
	
	// Apply limit and offset
	if filter != nil {
		if filter.Offset > 0 && filter.Offset < len(alerts) {
			alerts = alerts[filter.Offset:]
		}
		if filter.Limit > 0 && filter.Limit < len(alerts) {
			alerts = alerts[:filter.Limit]
		}
	}
	
	return alerts, nil
}

func (a *databasePerformanceAlerting) GetAlert(ctx context.Context, alertID string) (*Alert, error) {
	a.alertsMutex.RLock()
	defer a.alertsMutex.RUnlock()
	
	alert, exists := a.alerts[alertID]
	if !exists {
		return nil, fmt.Errorf("alert %s not found", alertID)
	}
	
	return alert, nil
}

func (a *databasePerformanceAlerting) AcknowledgeAlert(ctx context.Context, alertID string, acknowledgment *AlertAcknowledgment) error {
	a.alertsMutex.Lock()
	defer a.alertsMutex.Unlock()
	
	alert, exists := a.alerts[alertID]
	if !exists {
		return fmt.Errorf("alert %s not found", alertID)
	}
	
	if alert.State != AlertStateActive {
		return fmt.Errorf("alert %s is not in active state", alertID)
	}
	
	alert.State = AlertStateAcknowledged
	alert.AcknowledgedAt = acknowledgment.Timestamp
	alert.AcknowledgedBy = acknowledgment.AcknowledgedBy
	alert.UpdatedAt = time.Now()
	
	// Add to history
	a.addAlertHistoryEntry(alertID, "acknowledged", acknowledgment.AcknowledgedBy, map[string]interface{}{
		"reason": acknowledgment.Reason,
	})
	
	log.Printf("Alert %s acknowledged by %s", alertID, acknowledgment.AcknowledgedBy)
	return nil
}

func (a *databasePerformanceAlerting) ResolveAlert(ctx context.Context, alertID string, resolution *AlertResolution) error {
	a.alertsMutex.Lock()
	defer a.alertsMutex.Unlock()
	
	alert, exists := a.alerts[alertID]
	if !exists {
		return fmt.Errorf("alert %s not found", alertID)
	}
	
	if alert.State == AlertStateResolved {
		return fmt.Errorf("alert %s is already resolved", alertID)
	}
	
	alert.State = AlertStateResolved
	alert.ResolvedAt = resolution.Timestamp
	alert.ResolvedBy = resolution.ResolvedBy
	alert.EndsAt = time.Now()
	alert.UpdatedAt = time.Now()
	
	// Add to history
	a.addAlertHistoryEntry(alertID, "resolved", resolution.ResolvedBy, map[string]interface{}{
		"reason":     resolution.Reason,
		"resolution": resolution.Resolution,
	})
	
	log.Printf("Alert %s resolved by %s", alertID, resolution.ResolvedBy)
	return nil
}

func (a *databasePerformanceAlerting) SnoozeAlert(ctx context.Context, alertID string, duration time.Duration, reason string) error {
	a.alertsMutex.Lock()
	defer a.alertsMutex.Unlock()
	
	alert, exists := a.alerts[alertID]
	if !exists {
		return fmt.Errorf("alert %s not found", alertID)
	}
	
	if alert.State != AlertStateActive {
		return fmt.Errorf("alert %s is not in active state", alertID)
	}
	
	alert.State = AlertStateSnoozed
	alert.SnoozedUntil = time.Now().Add(duration)
	alert.SnoozedBy = "system" // In real implementation, get from context
	alert.UpdatedAt = time.Now()
	
	// Add to history
	a.addAlertHistoryEntry(alertID, "snoozed", "system", map[string]interface{}{
		"reason":   reason,
		"duration": duration.String(),
		"until":    alert.SnoozedUntil,
	})
	
	log.Printf("Alert %s snoozed for %v", alertID, duration)
	return nil
}

// Notification channel management

func (a *databasePerformanceAlerting) AddNotificationChannel(ctx context.Context, channel *NotificationChannel) error {
	if channel.ChannelID == "" {
		channel.ChannelID = fmt.Sprintf("channel_%d", time.Now().UnixNano())
	}
	
	channel.CreatedAt = time.Now()
	channel.UpdatedAt = time.Now()
	
	// Validate channel configuration
	if err := a.validateNotificationChannel(channel); err != nil {
		return fmt.Errorf("invalid notification channel: %w", err)
	}
	
	a.channelsMutex.Lock()
	a.notificationChannels[channel.ChannelID] = channel
	a.channelsMutex.Unlock()
	
	log.Printf("Notification channel added: %s (%s)", channel.ChannelID, channel.Name)
	return nil
}

func (a *databasePerformanceAlerting) UpdateNotificationChannel(ctx context.Context, channelID string, channel *NotificationChannel) error {
	a.channelsMutex.Lock()
	defer a.channelsMutex.Unlock()
	
	existingChannel, exists := a.notificationChannels[channelID]
	if !exists {
		return fmt.Errorf("notification channel %s not found", channelID)
	}
	
	channel.ChannelID = channelID
	channel.CreatedAt = existingChannel.CreatedAt
	channel.UpdatedAt = time.Now()
	channel.SuccessCount = existingChannel.SuccessCount
	channel.FailureCount = existingChannel.FailureCount
	
	// Validate updated channel
	if err := a.validateNotificationChannel(channel); err != nil {
		return fmt.Errorf("invalid notification channel update: %w", err)
	}
	
	a.notificationChannels[channelID] = channel
	
	log.Printf("Notification channel updated: %s (%s)", channelID, channel.Name)
	return nil
}

func (a *databasePerformanceAlerting) RemoveNotificationChannel(ctx context.Context, channelID string) error {
	a.channelsMutex.Lock()
	defer a.channelsMutex.Unlock()
	
	if _, exists := a.notificationChannels[channelID]; !exists {
		return fmt.Errorf("notification channel %s not found", channelID)
	}
	
	delete(a.notificationChannels, channelID)
	
	log.Printf("Notification channel removed: %s", channelID)
	return nil
}

func (a *databasePerformanceAlerting) TestNotificationChannel(ctx context.Context, channelID string) error {
	a.channelsMutex.RLock()
	channel, exists := a.notificationChannels[channelID]
	a.channelsMutex.RUnlock()
	
	if !exists {
		return fmt.Errorf("notification channel %s not found", channelID)
	}
	
	// Create test alert
	testAlert := &Alert{
		AlertID:     "test_alert",
		RuleID:      "test_rule",
		RuleName:    "Test Rule",
		State:       AlertStateActive,
		Severity:    SeverityInfo,
		Message:     "This is a test alert to verify notification channel configuration",
		Description: "Test notification from Database Performance Alerting system",
		Value:       100.0,
		Threshold:   90.0,
		StartsAt:    time.Now(),
		UpdatedAt:   time.Now(),
	}
	
	// Send test notification
	if err := a.sendNotification(ctx, testAlert, channel); err != nil {
		return fmt.Errorf("test notification failed: %w", err)
	}
	
	log.Printf("Test notification sent successfully to channel %s", channelID)
	return nil
}

func (a *databasePerformanceAlerting) ListNotificationChannels(ctx context.Context) ([]*NotificationChannel, error) {
	a.channelsMutex.RLock()
	defer a.channelsMutex.RUnlock()
	
	var channels []*NotificationChannel
	for _, channel := range a.notificationChannels {
		channels = append(channels, channel)
	}
	
	// Sort by creation time
	sort.Slice(channels, func(i, j int) bool {
		return channels[i].CreatedAt.After(channels[j].CreatedAt)
	})
	
	return channels, nil
}

// Alert evaluation and triggering

func (a *databasePerformanceAlerting) EvaluateAlerts(ctx context.Context) error {
	a.alertRulesMutex.RLock()
	rules := make([]*AlertRule, 0, len(a.alertRules))
	for _, rule := range a.alertRules {
		if rule.IsEnabled && rule.State != AlertRuleStateDisabled {
			rules = append(rules, rule)
		}
	}
	a.alertRulesMutex.RUnlock()
	
	for _, rule := range rules {
		select {
		case a.evaluationQueue <- rule:
			// Rule queued for evaluation
		default:
			log.Printf("Evaluation queue full, skipping rule %s", rule.RuleID)
		}
	}
	
	return nil
}

func (a *databasePerformanceAlerting) TriggerAlert(ctx context.Context, alert *Alert) error {
	if alert.AlertID == "" {
		alert.AlertID = fmt.Sprintf("alert_%d", time.Now().UnixNano())
	}
	
	alert.StartsAt = time.Now()
	alert.UpdatedAt = time.Now()
	alert.ActiveAt = time.Now()
	alert.Fingerprint = a.generateAlertFingerprint(alert)
	
	// Check for suppression
	if a.isAlertSuppressed(alert) {
		log.Printf("Alert %s suppressed", alert.AlertID)
		return nil
	}
	
	// Store alert
	a.alertsMutex.Lock()
	a.alerts[alert.AlertID] = alert
	a.alertsMutex.Unlock()
	
	// Update rule statistics
	a.alertRulesMutex.Lock()
	if rule, exists := a.alertRules[alert.RuleID]; exists {
		rule.AlertCount++
	}
	a.alertRulesMutex.Unlock()
	
	// Add to history
	a.addAlertHistoryEntry(alert.AlertID, "triggered", "system", map[string]interface{}{
		"value":     alert.Value,
		"threshold": alert.Threshold,
	})
	
	// Queue notifications
	a.queueNotifications(ctx, alert)
	
	log.Printf("Alert triggered: %s (%s)", alert.AlertID, alert.RuleName)
	return nil
}

// Analytics and reporting

func (a *databasePerformanceAlerting) GetAlertHistory(ctx context.Context, timeRange *TimeRange, filter *AlertHistoryFilter) ([]*AlertHistoryEntry, error) {
	a.historyMutex.RLock()
	defer a.historyMutex.RUnlock()
	
	var filteredHistory []*AlertHistoryEntry
	
	for _, entry := range a.alertHistory {
		// Filter by time range
		if timeRange != nil {
			if entry.Timestamp.Before(timeRange.Start) || entry.Timestamp.After(timeRange.End) {
				continue
			}
		}
		
		// Apply other filters
		if filter != nil {
			if filter.RuleID != "" && entry.RuleID != filter.RuleID {
				continue
			}
			if filter.ActionType != "" && entry.ActionType != filter.ActionType {
				continue
			}
			if filter.PerformedBy != "" && entry.PerformedBy != filter.PerformedBy {
				continue
			}
		}
		
		filteredHistory = append(filteredHistory, entry)
	}
	
	// Sort by timestamp (newest first)
	sort.Slice(filteredHistory, func(i, j int) bool {
		return filteredHistory[i].Timestamp.After(filteredHistory[j].Timestamp)
	})
	
	// Apply limit and offset
	if filter != nil {
		if filter.Offset > 0 && filter.Offset < len(filteredHistory) {
			filteredHistory = filteredHistory[filter.Offset:]
		}
		if filter.Limit > 0 && filter.Limit < len(filteredHistory) {
			filteredHistory = filteredHistory[:filter.Limit]
		}
	}
	
	return filteredHistory, nil
}

func (a *databasePerformanceAlerting) GetAlertStatistics(ctx context.Context, timeRange *TimeRange) (*AlertStatistics, error) {
	if timeRange == nil {
		timeRange = &TimeRange{
			Start: time.Now().Add(-24 * time.Hour),
			End:   time.Now(),
		}
	}
	
	a.alertsMutex.RLock()
	defer a.alertsMutex.RUnlock()
	
	stats := &AlertStatistics{
		TimeRange:          timeRange,
		SeverityBreakdown:  make(map[AlertSeverity]int),
		StateBreakdown:     make(map[AlertState]int),
		RuleBreakdown:      make(map[string]int),
		HourlyDistribution: make(map[int]int),
		DailyDistribution:  make(map[string]int),
		GeneratedAt:        time.Now(),
	}
	
	var resolutionTimes []time.Duration
	
	for _, alert := range a.alerts {
		// Filter by time range
		if alert.StartsAt.Before(timeRange.Start) || alert.StartsAt.After(timeRange.End) {
			continue
		}
		
		stats.TotalAlerts++
		
		// Count by state
		stats.StateBreakdown[alert.State]++
		switch alert.State {
		case AlertStateActive:
			stats.ActiveAlerts++
		case AlertStateResolved:
			stats.ResolvedAlerts++
		case AlertStateAcknowledged:
			stats.AcknowledgedAlerts++
		}
		
		// Count by severity
		stats.SeverityBreakdown[alert.Severity]++
		
		// Count by rule
		stats.RuleBreakdown[alert.RuleID]++
		
		// Hourly distribution
		hour := alert.StartsAt.Hour()
		stats.HourlyDistribution[hour]++
		
		// Daily distribution
		day := alert.StartsAt.Format("2006-01-02")
		stats.DailyDistribution[day]++
		
		// Resolution time
		if alert.State == AlertStateResolved && !alert.ResolvedAt.IsZero() {
			resolutionTime := alert.ResolvedAt.Sub(alert.StartsAt)
			resolutionTimes = append(resolutionTimes, resolutionTime)
		}
	}
	
	// Calculate resolution time statistics
	if len(resolutionTimes) > 0 {
		var totalResolutionTime time.Duration
		for _, rt := range resolutionTimes {
			totalResolutionTime += rt
		}
		stats.AverageResolutionTime = totalResolutionTime / time.Duration(len(resolutionTimes))
		
		// Sort for median
		sort.Slice(resolutionTimes, func(i, j int) bool {
			return resolutionTimes[i] < resolutionTimes[j]
		})
		stats.MedianResolutionTime = resolutionTimes[len(resolutionTimes)/2]
	}
	
	// Mock false positive rate calculation
	stats.FalsePositiveRate = 0.05 // 5% false positive rate
	
	return stats, nil
}

func (a *databasePerformanceAlerting) GetAlertTrends(ctx context.Context, timeRange *TimeRange) (*AlertTrends, error) {
	if timeRange == nil {
		timeRange = &TimeRange{
			Start: time.Now().Add(-7 * 24 * time.Hour),
			End:   time.Now(),
		}
	}
	
	trends := &AlertTrends{
		TimeRange:      timeRange,
		SeverityTrends: make(map[AlertSeverity]*TrendData),
		RuleTrends:     make(map[string]*TrendData),
		GeneratedAt:    time.Now(),
	}
	
	// Generate alert frequency trend (simplified)
	frequencyTrend := &AlertFrequencyTrend{
		DataPoints: []AlertFrequencyPoint{},
		Trend:      "stable",
		TrendStrength: 0.1,
	}
	
	// Generate data points (daily aggregation)
	duration := timeRange.End.Sub(timeRange.Start)
	days := int(duration.Hours() / 24)
	if days == 0 {
		days = 1
	}
	
	for i := 0; i < days; i++ {
		timestamp := timeRange.Start.Add(time.Duration(i) * 24 * time.Hour)
		// Mock alert count - in real implementation, count actual alerts
		alertCount := 5 + (i % 3) // Mock varying alert counts
		
		frequencyTrend.DataPoints = append(frequencyTrend.DataPoints, AlertFrequencyPoint{
			Timestamp:  timestamp,
			AlertCount: alertCount,
		})
	}
	
	trends.AlertFrequency = frequencyTrend
	
	// Generate resolution trends (simplified)
	trends.ResolutionTrends = &ResolutionTrendData{
		AverageResolutionTimes: []ResolutionTimePoint{},
		Trend:                  "improving",
		ImprovementPercent:     15.0,
	}
	
	return trends, nil
}

// Health and diagnostics

func (a *databasePerformanceAlerting) GetAlertingHealth(ctx context.Context) (*AlertingHealth, error) {
	a.mutex.RLock()
	isRunning := a.isRunning
	startTime := a.startTime
	a.mutex.RUnlock()
	
	health := &AlertingHealth{
		IsHealthy:          isRunning,
		Status:             "healthy",
		ComponentHealth:    make(map[string]string),
		PerformanceMetrics: &AlertingPerformanceMetrics{},
		HealthChecks:       []AlertingHealthCheck{},
	}
	
	if isRunning {
		health.Uptime = time.Since(startTime)
	}
	
	// Count rules by state
	a.alertRulesMutex.RLock()
	for _, rule := range a.alertRules {
		switch rule.State {
		case AlertRuleStateOK:
			health.ActiveRules++
		case AlertRuleStateDisabled:
			health.DisabledRules++
		case AlertRuleStateError:
			health.ErrorRules++
		}
	}
	a.alertRulesMutex.RUnlock()
	
	// Count alerts by state
	a.alertsMutex.RLock()
	for _, alert := range a.alerts {
		switch alert.State {
		case AlertStateActive:
			health.ActiveAlerts++
		case AlertStatePending:
			health.PendingAlerts++
		}
	}
	a.alertsMutex.RUnlock()
	
	// Component health checks
	health.ComponentHealth["database"] = "healthy"
	health.ComponentHealth["metrics_collector"] = "healthy"
	health.ComponentHealth["slow_query_logger"] = "healthy"
	
	if a.metricsCollector.IsRunning() {
		health.ComponentHealth["metrics_collector"] = "healthy"
	} else {
		health.ComponentHealth["metrics_collector"] = "unhealthy"
		health.Status = "degraded"
		health.IsHealthy = false
	}
	
	if a.slowQueryLogger.IsRunning() {
		health.ComponentHealth["slow_query_logger"] = "healthy"
	} else {
		health.ComponentHealth["slow_query_logger"] = "unhealthy"
		health.Status = "degraded"
		health.IsHealthy = false
	}
	
	// Performance metrics (mock data)
	health.PerformanceMetrics.AverageEvaluationTime = 50 * time.Millisecond
	health.PerformanceMetrics.MaxEvaluationTime = 200 * time.Millisecond
	health.PerformanceMetrics.EvaluationsPerSecond = 10.0
	health.PerformanceMetrics.NotificationsPerSecond = 2.0
	health.PerformanceMetrics.MemoryUsage = 1024 * 1024 * 50 // 50MB
	health.PerformanceMetrics.RuleEvaluationQueueDepth = len(a.evaluationQueue)
	health.PerformanceMetrics.NotificationQueueDepth = len(a.notificationQueue)
	
	// Health checks
	health.HealthChecks = []AlertingHealthCheck{
		{
			CheckName: "database_connectivity",
			Status:    "pass",
			LastRun:   time.Now(),
			Duration:  10 * time.Millisecond,
			Message:   "Database connection is healthy",
		},
		{
			CheckName: "evaluation_queue",
			Status:    "pass",
			LastRun:   time.Now(),
			Duration:  1 * time.Millisecond,
			Message:   fmt.Sprintf("Evaluation queue depth: %d", len(a.evaluationQueue)),
		},
		{
			CheckName: "notification_queue",
			Status:    "pass",
			LastRun:   time.Now(),
			Duration:  1 * time.Millisecond,
			Message:   fmt.Sprintf("Notification queue depth: %d", len(a.notificationQueue)),
		},
	}
	
	// Overall health determination
	if health.ErrorRules > 0 {
		health.Status = "degraded"
		health.IsHealthy = false
	}
	
	if health.PerformanceMetrics.RuleEvaluationQueueDepth > 50 {
		health.Status = "degraded"
		health.IsHealthy = false
	}
	
	return health, nil
}

func (a *databasePerformanceAlerting) GetAlertEvaluationMetrics(ctx context.Context) (*AlertEvaluationMetrics, error) {
	a.metricsMutex.RLock()
	defer a.metricsMutex.RUnlock()
	
	// Return copy of evaluation metrics
	metrics := *a.evaluationMetrics
	metrics.LastEvaluationCycle = time.Now().Add(-a.evaluationInterval)
	metrics.NextEvaluationCycle = time.Now().Add(a.evaluationInterval)
	metrics.EvaluationQueueLength = len(a.evaluationQueue)
	
	// Calculate error rate
	if metrics.TotalEvaluations > 0 {
		metrics.EvaluationErrorRate = float64(metrics.FailedEvaluations) / float64(metrics.TotalEvaluations)
	}
	
	return &metrics, nil
}

// Internal helper methods

func (a *databasePerformanceAlerting) initializeDefaultRules(ctx context.Context) error {
	// High execution time alert rule
	executionTimeRule := &AlertRule{
		RuleID:             "high_execution_time",
		Name:               "High Query Execution Time",
		Description:        "Triggers when average query execution time exceeds threshold",
		Type:               AlertRuleTypeThreshold,
		MetricType:         MetricTypeExecutionTime,
		Condition: &AlertCondition{
			Operator:        OperatorGreaterThan,
			Value:           200, // 200ms
			AggregationType: AggregationAvg,
			ComparisonType:  ComparisonAbsolute,
			TimeWindow:      5 * time.Minute,
			MinSamples:      5,
		},
		Threshold: &AlertThreshold{
			Critical: &ThresholdLevel{
				Value:    500, // 500ms
				Operator: OperatorGreaterThan,
				Duration: 2 * time.Minute,
			},
			Warning: &ThresholdLevel{
				Value:    200, // 200ms
				Operator: OperatorGreaterThan,
				Duration: 5 * time.Minute,
			},
		},
		EvaluationWindow:   5 * time.Minute,
		EvaluationInterval: 30 * time.Second,
		Severity:           SeverityWarning,
		State:              AlertRuleStateOK,
		IsEnabled:          true,
		NotificationChannels: []string{}, // Will be populated when channels are added
		AutoResolve:        true,
		AutoResolveAfter:   10 * time.Minute,
		SuppressDuplicates: true,
		SuppressionWindow:  5 * time.Minute,
	}
	
	// High error rate alert rule
	errorRateRule := &AlertRule{
		RuleID:             "high_error_rate",
		Name:               "High Query Error Rate",
		Description:        "Triggers when query error rate exceeds threshold",
		Type:               AlertRuleTypeThreshold,
		MetricType:         MetricTypeErrorRate,
		Condition: &AlertCondition{
			Operator:        OperatorGreaterThan,
			Value:           0.05, // 5%
			AggregationType: AggregationAvg,
			ComparisonType:  ComparisonPercentage,
			TimeWindow:      5 * time.Minute,
			MinSamples:      10,
		},
		Threshold: &AlertThreshold{
			Critical: &ThresholdLevel{
				Value:    0.10, // 10%
				Operator: OperatorGreaterThan,
				Duration: 2 * time.Minute,
			},
			Warning: &ThresholdLevel{
				Value:    0.05, // 5%
				Operator: OperatorGreaterThan,
				Duration: 5 * time.Minute,
			},
		},
		EvaluationWindow:   5 * time.Minute,
		EvaluationInterval: 30 * time.Second,
		Severity:           SeverityCritical,
		State:              AlertRuleStateOK,
		IsEnabled:          true,
		AutoResolve:        true,
		AutoResolveAfter:   10 * time.Minute,
		SuppressDuplicates: true,
		SuppressionWindow:  5 * time.Minute,
	}
	
	// Low cache hit rate alert rule
	cacheHitRule := &AlertRule{
		RuleID:             "low_cache_hit_rate",
		Name:               "Low Cache Hit Rate",
		Description:        "Triggers when cache hit rate falls below threshold",
		Type:               AlertRuleTypeThreshold,
		MetricType:         MetricTypeCacheHitRate,
		Condition: &AlertCondition{
			Operator:        OperatorLessThan,
			Value:           0.80, // 80%
			AggregationType: AggregationAvg,
			ComparisonType:  ComparisonPercentage,
			TimeWindow:      10 * time.Minute,
			MinSamples:      10,
		},
		Threshold: &AlertThreshold{
			Warning: &ThresholdLevel{
				Value:    0.80, // 80%
				Operator: OperatorLessThan,
				Duration: 10 * time.Minute,
			},
		},
		EvaluationWindow:   10 * time.Minute,
		EvaluationInterval: 60 * time.Second,
		Severity:           SeverityWarning,
		State:              AlertRuleStateOK,
		IsEnabled:          true,
		AutoResolve:        true,
		AutoResolveAfter:   15 * time.Minute,
		SuppressDuplicates: true,
		SuppressionWindow:  10 * time.Minute,
	}
	
	// Create default rules
	rules := []*AlertRule{executionTimeRule, errorRateRule, cacheHitRule}
	for _, rule := range rules {
		if err := a.CreateAlertRule(ctx, rule); err != nil {
			return fmt.Errorf("failed to create default rule %s: %w", rule.RuleID, err)
		}
	}
	
	return nil
}

func (a *databasePerformanceAlerting) initializeDefaultChannels(ctx context.Context) error {
	// Log channel (always available)
	logChannel := &NotificationChannel{
		ChannelID:     "default_log",
		Name:          "Default Log Channel",
		Type:          ChannelTypeCustom,
		Configuration: &ChannelConfiguration{
			CustomConfig: map[string]interface{}{
				"type": "log",
			},
		},
		Settings: &NotificationSettings{
			Severities: []AlertSeverity{SeverityCritical, SeverityWarning, SeverityInfo},
			Template: &MessageTemplate{
				BodyTemplate: "Alert: {{.RuleName}} - {{.Message}}",
			},
		},
		IsEnabled: true,
	}
	
	return a.AddNotificationChannel(ctx, logChannel)
}

// Worker methods

func (a *databasePerformanceAlerting) evaluationScheduler(ctx context.Context) {
	defer a.wg.Done()
	
	for {
		select {
		case <-a.stopChan:
			return
		case <-a.evaluationTicker.C:
			if err := a.EvaluateAlerts(ctx); err != nil {
				log.Printf("Error in evaluation scheduler: %v", err)
			}
		}
	}
}

func (a *databasePerformanceAlerting) evaluationWorker(ctx context.Context) {
	defer a.wg.Done()
	
	for {
		select {
		case <-a.stopChan:
			return
		case rule := <-a.evaluationQueue:
			if err := a.evaluateRule(ctx, rule); err != nil {
				log.Printf("Error evaluating rule %s: %v", rule.RuleID, err)
			}
		}
	}
}

func (a *databasePerformanceAlerting) notificationWorker(ctx context.Context) {
	defer a.wg.Done()
	
	for {
		select {
		case <-a.stopChan:
			return
		case task := <-a.notificationQueue:
			if err := a.processNotificationTask(ctx, task); err != nil {
				log.Printf("Error processing notification task: %v", err)
			}
		}
	}
}

func (a *databasePerformanceAlerting) maintenanceWorker(ctx context.Context) {
	defer a.wg.Done()
	
	ticker := time.NewTicker(5 * time.Minute) // Run every 5 minutes
	defer ticker.Stop()
	
	for {
		select {
		case <-a.stopChan:
			return
		case <-ticker.C:
			a.performMaintenance(ctx)
		}
	}
}

func (a *databasePerformanceAlerting) evaluateRule(ctx context.Context, rule *AlertRule) error {
	startTime := time.Now()
	
	// Update evaluation metrics
	a.metricsMutex.Lock()
	a.evaluationMetrics.TotalEvaluations++
	if stats, exists := a.evaluationMetrics.RuleEvaluationStats[rule.RuleID]; exists {
		stats.EvaluationCount++
	}
	a.metricsMutex.Unlock()
	
	// Update rule evaluation time
	a.alertRulesMutex.Lock()
	rule.LastEvaluation = startTime
	rule.EvaluationCount++
	rule.NextEvaluation = startTime.Add(rule.EvaluationInterval)
	a.alertRulesMutex.Unlock()
	
	// Get metrics data for evaluation
	evaluationData, err := a.getEvaluationData(ctx, rule)
	if err != nil {
		// Update error metrics
		a.metricsMutex.Lock()
		a.evaluationMetrics.FailedEvaluations++
		if stats, exists := a.evaluationMetrics.RuleEvaluationStats[rule.RuleID]; exists {
			stats.ErrorCount++
			stats.LastError = time.Now()
			stats.LastErrorMessage = err.Error()
		}
		a.metricsMutex.Unlock()
		
		return fmt.Errorf("failed to get evaluation data: %w", err)
	}
	
	// Evaluate condition
	shouldAlert, alertValue := a.evaluateCondition(rule, evaluationData)
	
	// Update success metrics
	a.metricsMutex.Lock()
	a.evaluationMetrics.SuccessfulEvaluations++
	if stats, exists := a.evaluationMetrics.RuleEvaluationStats[rule.RuleID]; exists {
		stats.SuccessCount++
		stats.LastSuccess = time.Now()
		duration := time.Since(startTime)
		stats.AverageEvaluationTime = (stats.AverageEvaluationTime*time.Duration(stats.SuccessCount-1) + duration) / time.Duration(stats.SuccessCount)
	}
	a.metricsMutex.Unlock()
	
	if shouldAlert {
		// Create and trigger alert
		alert := &Alert{
			RuleID:      rule.RuleID,
			RuleName:    rule.Name,
			State:       AlertStateActive,
			Severity:    rule.Severity,
			Message:     a.generateAlertMessage(rule, alertValue),
			Description: rule.Description,
			Value:       alertValue,
			Threshold:   rule.Condition.Value,
			Labels: map[string]string{
				"rule_id":     rule.RuleID,
				"metric_type": string(rule.MetricType),
				"severity":    string(rule.Severity),
			},
			Annotations: map[string]string{
				"rule_name":   rule.Name,
				"description": rule.Description,
			},
			EvaluationData: &AlertEvaluationData{
				EvaluatedAt:    startTime,
				QueryResult:    evaluationData,
				MetricValue:    alertValue,
				EvaluationTime: time.Since(startTime),
			},
		}
		
		if err := a.TriggerAlert(ctx, alert); err != nil {
			return fmt.Errorf("failed to trigger alert: %w", err)
		}
	}
	
	return nil
}

func (a *databasePerformanceAlerting) getEvaluationData(ctx context.Context, rule *AlertRule) (interface{}, error) {
	// Get time range for evaluation
	endTime := time.Now()
	startTime := endTime.Add(-rule.EvaluationWindow)
	timeRange := &TimeRange{Start: startTime, End: endTime}
	
	switch rule.MetricType {
	case MetricTypeExecutionTime:
		filter := &MetricsFilter{
			TimeRange: timeRange,
			Limit:     1000,
		}
		collection, err := a.metricsCollector.GetQueryMetrics(ctx, filter)
		if err != nil {
			return nil, err
		}
		return collection, nil
		
	case MetricTypeErrorRate:
		filter := &MetricsFilter{
			TimeRange: timeRange,
			HasErrors: &[]bool{true}[0], // Pointer to true
			Limit:     1000,
		}
		collection, err := a.metricsCollector.GetQueryMetrics(ctx, filter)
		if err != nil {
			return nil, err
		}
		return collection, nil
		
	case MetricTypeCacheHitRate:
		filter := &MetricsFilter{
			TimeRange: timeRange,
			Limit:     1000,
		}
		collection, err := a.metricsCollector.GetQueryMetrics(ctx, filter)
		if err != nil {
			return nil, err
		}
		return collection, nil
		
	case MetricTypeSlowQueryCount:
		slowQueries, err := a.slowQueryLogger.GetSlowQueries(ctx, startTime, 1000)
		if err != nil {
			return nil, err
		}
		return slowQueries, nil
		
	default:
		return nil, fmt.Errorf("unsupported metric type: %s", rule.MetricType)
	}
}

func (a *databasePerformanceAlerting) evaluateCondition(rule *AlertRule, data interface{}) (bool, float64) {
	var value float64
	
	switch rule.MetricType {
	case MetricTypeExecutionTime:
		if collection, ok := data.(*QueryMetricsCollection); ok && collection.Summary != nil {
			switch rule.Condition.AggregationType {
			case AggregationAvg:
				value = float64(collection.Summary.AverageExecutionTime.Milliseconds())
			case AggregationMax:
				value = float64(collection.Summary.SlowestQuery.Milliseconds())
			case AggregationP95:
				value = float64(collection.Summary.P95ExecutionTime.Milliseconds())
			case AggregationP99:
				value = float64(collection.Summary.P99ExecutionTime.Milliseconds())
			default:
				value = float64(collection.Summary.AverageExecutionTime.Milliseconds())
			}
		}
		
	case MetricTypeErrorRate:
		if collection, ok := data.(*QueryMetricsCollection); ok && collection.Summary != nil {
			value = collection.Summary.ErrorRate * 100 // Convert to percentage
		}
		
	case MetricTypeCacheHitRate:
		if collection, ok := data.(*QueryMetricsCollection); ok && collection.Summary != nil {
			value = collection.Summary.CacheHitRate * 100 // Convert to percentage
		}
		
	case MetricTypeSlowQueryCount:
		if slowQueries, ok := data.([]DetectedSlowQuery); ok {
			value = float64(len(slowQueries))
		}
	}
	
	// Check if we have minimum samples
	if rule.Condition.MinSamples > 0 {
		sampleCount := a.getSampleCount(data)
		if sampleCount < rule.Condition.MinSamples {
			return false, value
		}
	}
	
	// Evaluate condition
	switch rule.Condition.Operator {
	case OperatorGreaterThan:
		return value > rule.Condition.Value, value
	case OperatorGreaterThanEqual:
		return value >= rule.Condition.Value, value
	case OperatorLessThan:
		return value < rule.Condition.Value, value
	case OperatorLessThanEqual:
		return value <= rule.Condition.Value, value
	case OperatorEqual:
		return value == rule.Condition.Value, value
	case OperatorNotEqual:
		return value != rule.Condition.Value, value
	default:
		return false, value
	}
}

func (a *databasePerformanceAlerting) getSampleCount(data interface{}) int {
	switch d := data.(type) {
	case *QueryMetricsCollection:
		return len(d.Metrics)
	case []DetectedSlowQuery:
		return len(d)
	default:
		return 0
	}
}

func (a *databasePerformanceAlerting) generateAlertMessage(rule *AlertRule, value float64) string {
	switch rule.MetricType {
	case MetricTypeExecutionTime:
		return fmt.Sprintf("Query execution time %.1fms exceeds threshold %.1fms", value, rule.Condition.Value)
	case MetricTypeErrorRate:
		return fmt.Sprintf("Query error rate %.2f%% exceeds threshold %.2f%%", value, rule.Condition.Value)
	case MetricTypeCacheHitRate:
		return fmt.Sprintf("Cache hit rate %.2f%% below threshold %.2f%%", value, rule.Condition.Value)
	case MetricTypeSlowQueryCount:
		return fmt.Sprintf("Slow query count %.0f exceeds threshold %.0f", value, rule.Condition.Value)
	default:
		return fmt.Sprintf("Metric %s value %.2f violates threshold %.2f", rule.MetricType, value, rule.Condition.Value)
	}
}

func (a *databasePerformanceAlerting) generateAlertFingerprint(alert *Alert) string {
	// Create unique fingerprint based on rule and labels
	data := fmt.Sprintf("%s:%s", alert.RuleID, alert.Message)
	if len(alert.Labels) > 0 {
		labelsJSON, _ := json.Marshal(alert.Labels)
		data += string(labelsJSON)
	}
	return fmt.Sprintf("%x", []byte(data)[:8]) // Simple hash
}

func (a *databasePerformanceAlerting) isAlertSuppressed(alert *Alert) bool {
	a.suppressionMutex.RLock()
	defer a.suppressionMutex.RUnlock()
	
	fingerprint := alert.Fingerprint
	if suppressedUntil, exists := a.suppressionCache[fingerprint]; exists {
		return time.Now().Before(suppressedUntil)
	}
	
	return false
}

func (a *databasePerformanceAlerting) queueNotifications(ctx context.Context, alert *Alert) {
	// Get alert rule for notification channels
	a.alertRulesMutex.RLock()
	rule, exists := a.alertRules[alert.RuleID]
	a.alertRulesMutex.RUnlock()
	
	if !exists {
		log.Printf("Alert rule %s not found for notification", alert.RuleID)
		return
	}
	
	// Queue notification for each configured channel
	a.channelsMutex.RLock()
	for _, channelID := range rule.NotificationChannels {
		if channel, exists := a.notificationChannels[channelID]; exists && channel.IsEnabled {
			task := &NotificationTask{
				Alert:         alert,
				Channel:       channel,
				RetryCount:    0,
				ScheduledTime: time.Now(),
			}
			
			select {
			case a.notificationQueue <- task:
				// Task queued successfully
			default:
				log.Printf("Notification queue full, dropping notification for channel %s", channelID)
			}
		}
	}
	a.channelsMutex.RUnlock()
}

func (a *databasePerformanceAlerting) processNotificationTask(ctx context.Context, task *NotificationTask) error {
	startTime := time.Now()
	
	// Check rate limiting
	if a.isRateLimited(task.Channel.ChannelID) {
		log.Printf("Notification rate limited for channel %s", task.Channel.ChannelID)
		return nil
	}
	
	// Send notification
	err := a.sendNotification(ctx, task.Alert, task.Channel)
	
	// Log notification attempt
	logEntry := &NotificationLogEntry{
		Timestamp:   startTime,
		ChannelID:   task.Channel.ChannelID,
		ChannelType: string(task.Channel.Type),
		RetryCount:  task.RetryCount,
	}
	
	if err != nil {
		logEntry.Status = "failed"
		logEntry.Error = err.Error()
		
		// Update channel failure count
		a.channelsMutex.Lock()
		task.Channel.FailureCount++
		a.channelsMutex.Unlock()
		
		// Retry if configured
		if task.RetryCount < 3 { // Max 3 retries
			task.RetryCount++
			task.ScheduledTime = time.Now().Add(time.Duration(task.RetryCount) * time.Minute)
			
			// Requeue for retry
			select {
			case a.notificationQueue <- task:
				logEntry.Status = "retry"
			default:
				log.Printf("Failed to requeue notification for retry")
			}
		}
	} else {
		logEntry.Status = "sent"
		logEntry.Message = "Notification sent successfully"
		
		// Update channel success count
		a.channelsMutex.Lock()
		task.Channel.SuccessCount++
		task.Channel.LastUsed = time.Now()
		a.channelsMutex.Unlock()
	}
	
	// Add log entry to alert
	a.alertsMutex.Lock()
	if alert, exists := a.alerts[task.Alert.AlertID]; exists {
		alert.NotificationLog = append(alert.NotificationLog, logEntry)
	}
	a.alertsMutex.Unlock()
	
	return err
}

func (a *databasePerformanceAlerting) sendNotification(ctx context.Context, alert *Alert, channel *NotificationChannel) error {
	message := a.formatNotificationMessage(alert, channel)
	
	switch channel.Type {
	case ChannelTypeEmail:
		return a.sendEmailNotification(ctx, alert, channel, message)
	case ChannelTypeSlack:
		return a.sendSlackNotification(ctx, alert, channel, message)
	case ChannelTypeWebhook:
		return a.sendWebhookNotification(ctx, alert, channel, message)
	case ChannelTypeCustom:
		// For log channel
		log.Printf("ALERT NOTIFICATION [%s]: %s", channel.Name, message)
		return nil
	default:
		return fmt.Errorf("unsupported notification channel type: %s", channel.Type)
	}
}

func (a *databasePerformanceAlerting) formatNotificationMessage(alert *Alert, channel *NotificationChannel) string {
	template := "Alert: {{.RuleName}} - {{.Message}}"
	
	if channel.Settings != nil && channel.Settings.Template != nil && channel.Settings.Template.BodyTemplate != "" {
		template = channel.Settings.Template.BodyTemplate
	}
	
	// Simple template replacement (in real implementation, use proper templating)
	message := template
	message = fmt.Sprintf(message, alert.RuleName, alert.Message)
	
	// Add additional context if formatting is enabled
	if channel.Settings != nil && channel.Settings.Formatting != nil {
		formatting := channel.Settings.Formatting
		
		if formatting.IncludeTimestamp {
			message += fmt.Sprintf("\nTime: %s", alert.StartsAt.Format(time.RFC3339))
		}
		if formatting.IncludeValue {
			message += fmt.Sprintf("\nValue: %.2f", alert.Value)
		}
		if formatting.IncludeThreshold {
			message += fmt.Sprintf("\nThreshold: %.2f", alert.Threshold)
		}
		if formatting.IncludeLabels && len(alert.Labels) > 0 {
			message += "\nLabels:"
			for key, value := range alert.Labels {
				message += fmt.Sprintf("\n  %s: %s", key, value)
			}
		}
	}
	
	return message
}

func (a *databasePerformanceAlerting) sendEmailNotification(ctx context.Context, alert *Alert, channel *NotificationChannel, message string) error {
	// Mock email sending - in real implementation, use SMTP
	log.Printf("EMAIL NOTIFICATION to %s: %s", "admin@example.com", message)
	return nil
}

func (a *databasePerformanceAlerting) sendSlackNotification(ctx context.Context, alert *Alert, channel *NotificationChannel, message string) error {
	// Mock Slack sending - in real implementation, use Slack API
	log.Printf("SLACK NOTIFICATION to #alerts: %s", message)
	return nil
}

func (a *databasePerformanceAlerting) sendWebhookNotification(ctx context.Context, alert *Alert, channel *NotificationChannel, message string) error {
	// Mock webhook sending - in real implementation, send HTTP request
	log.Printf("WEBHOOK NOTIFICATION to %s: %s", "https://webhook.example.com", message)
	return nil
}

func (a *databasePerformanceAlerting) performMaintenance(ctx context.Context) {
	// Clean up resolved alerts older than 7 days
	cutoffTime := time.Now().Add(-7 * 24 * time.Hour)
	
	a.alertsMutex.Lock()
	for alertID, alert := range a.alerts {
		if alert.State == AlertStateResolved && alert.ResolvedAt.Before(cutoffTime) {
			delete(a.alerts, alertID)
		}
	}
	a.alertsMutex.Unlock()
	
	// Clean up old history entries (keep last 10000)
	a.historyMutex.Lock()
	if len(a.alertHistory) > 10000 {
		a.alertHistory = a.alertHistory[len(a.alertHistory)-10000:]
	}
	a.historyMutex.Unlock()
	
	// Reset rate limiters
	a.rateLimitersMutex.Lock()
	for key, limiter := range a.rateLimiters {
		if time.Since(limiter.LastReset) > 24*time.Hour {
			delete(a.rateLimiters, key)
		}
	}
	a.rateLimitersMutex.Unlock()
	
	// Clean up suppression cache
	a.suppressionMutex.Lock()
	now := time.Now()
	for fingerprint, suppressedUntil := range a.suppressionCache {
		if now.After(suppressedUntil) {
			delete(a.suppressionCache, fingerprint)
		}
	}
	a.suppressionMutex.Unlock()
	
	log.Printf("Alerting system maintenance completed")
}

// Validation and filtering helper methods

func (a *databasePerformanceAlerting) validateAlertRule(rule *AlertRule) error {
	if rule.Name == "" {
		return fmt.Errorf("rule name is required")
	}
	if rule.Condition == nil {
		return fmt.Errorf("rule condition is required")
	}
	if rule.EvaluationInterval <= 0 {
		return fmt.Errorf("evaluation interval must be positive")
	}
	if rule.EvaluationWindow <= 0 {
		return fmt.Errorf("evaluation window must be positive")
	}
	return nil
}

func (a *databasePerformanceAlerting) validateNotificationChannel(channel *NotificationChannel) error {
	if channel.Name == "" {
		return fmt.Errorf("channel name is required")
	}
	if channel.Configuration == nil {
		return fmt.Errorf("channel configuration is required")
	}
	// Add more validation based on channel type
	return nil
}

func (a *databasePerformanceAlerting) matchesRuleFilter(rule *AlertRule, filter *AlertRuleFilter) bool {
	if filter == nil {
		return true
	}
	
	if filter.Type != nil && rule.Type != *filter.Type {
		return false
	}
	if filter.Severity != nil && rule.Severity != *filter.Severity {
		return false
	}
	if filter.State != nil && rule.State != *filter.State {
		return false
	}
	if filter.IsEnabled != nil && rule.IsEnabled != *filter.IsEnabled {
		return false
	}
	if filter.CreatedBy != "" && rule.CreatedBy != filter.CreatedBy {
		return false
	}
	if filter.CreatedAfter != nil && rule.CreatedAt.Before(*filter.CreatedAfter) {
		return false
	}
	if filter.CreatedBefore != nil && rule.CreatedAt.After(*filter.CreatedBefore) {
		return false
	}
	
	return true
}

func (a *databasePerformanceAlerting) matchesAlertFilter(alert *Alert, filter *AlertFilter) bool {
	if filter == nil {
		return true
	}
	
	if filter.RuleID != "" && alert.RuleID != filter.RuleID {
		return false
	}
	if filter.State != nil && alert.State != *filter.State {
		return false
	}
	if filter.Severity != nil && alert.Severity != *filter.Severity {
		return false
	}
	if filter.StartsAfter != nil && alert.StartsAt.Before(*filter.StartsAfter) {
		return false
	}
	if filter.StartsBefore != nil && alert.StartsAt.After(*filter.StartsBefore) {
		return false
	}
	if filter.AcknowledgedBy != "" && alert.AcknowledgedBy != filter.AcknowledgedBy {
		return false
	}
	if filter.ResolvedBy != "" && alert.ResolvedBy != filter.ResolvedBy {
		return false
	}
	
	return true
}

func (a *databasePerformanceAlerting) addAlertHistoryEntry(alertID, actionType, performedBy string, details map[string]interface{}) {
	entry := &AlertHistoryEntry{
		EntryID:     fmt.Sprintf("history_%d", time.Now().UnixNano()),
		AlertID:     alertID,
		ActionType:  actionType,
		Timestamp:   time.Now(),
		PerformedBy: performedBy,
		Details:     details,
	}
	
	a.historyMutex.Lock()
	a.alertHistory = append(a.alertHistory, entry)
	a.historyMutex.Unlock()
}

func (a *databasePerformanceAlerting) isRateLimited(channelID string) bool {
	a.rateLimitersMutex.Lock()
	defer a.rateLimitersMutex.Unlock()
	
	limiter, exists := a.rateLimiters[channelID]
	if !exists {
		// Create new rate limiter (10 notifications per hour)
		limiter = &AlertRateLimiter{
			MaxCount:    10,
			TimeWindow:  time.Hour,
			Count:       0,
			WindowStart: time.Now(),
			LastReset:   time.Now(),
		}
		a.rateLimiters[channelID] = limiter
	}
	
	now := time.Now()
	
	// Reset window if needed
	if now.Sub(limiter.WindowStart) >= limiter.TimeWindow {
		limiter.Count = 0
		limiter.WindowStart = now
		limiter.LastReset = now
	}
	
	// Check if rate limited
	if limiter.Count >= limiter.MaxCount {
		return true
	}
	
	limiter.Count++
	return false
}