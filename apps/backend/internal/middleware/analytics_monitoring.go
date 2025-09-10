package middleware

import (
	"context"
	"log"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

// Prometheus metrics for analytics performance monitoring
var (
	// Analytics request duration histogram
	analyticsRequestDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "imkitchen_analytics_request_duration_seconds",
			Help:    "Duration of analytics requests in seconds",
			Buckets: []float64{0.1, 0.25, 0.5, 1, 2, 5, 10},
		},
		[]string{"endpoint", "weeks", "status"},
	)

	// Analytics request counter
	analyticsRequestTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "imkitchen_analytics_requests_total",
			Help: "Total number of analytics requests",
		},
		[]string{"endpoint", "status"},
	)

	// Cache hit/miss ratio for analytics
	analyticsCacheHits = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "imkitchen_analytics_cache_hits_total",
			Help: "Total number of analytics cache hits",
		},
		[]string{"cache_type"},
	)

	analyticsCacheMisses = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "imkitchen_analytics_cache_misses_total",
			Help: "Total number of analytics cache misses",
		},
		[]string{"cache_type"},
	)

	// Analytics calculation performance
	analyticsCalculationDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "imkitchen_analytics_calculation_duration_seconds",
			Help:    "Duration of analytics calculations in seconds",
			Buckets: []float64{0.05, 0.1, 0.25, 0.5, 1, 2},
		},
		[]string{"calculation_type"},
	)

	// Rotation reset operations
	rotationResetTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "imkitchen_rotation_reset_total",
			Help: "Total number of rotation resets",
		},
		[]string{"preserve_patterns", "preserve_favorites"},
	)

	// Analytics export operations
	analyticsExportTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "imkitchen_analytics_export_total",
			Help: "Total number of analytics exports",
		},
		[]string{"format", "includes_debug_logs"},
	)

	// Debug logs request metrics
	debugLogsRequestDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "imkitchen_debug_logs_request_duration_seconds",
			Help:    "Duration of debug logs requests in seconds",
			Buckets: []float64{0.01, 0.05, 0.1, 0.25, 0.5, 1},
		},
		[]string{"limit_range"},
	)
)

// AnalyticsMonitoringMiddleware provides performance monitoring for analytics endpoints
func AnalyticsMonitoringMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Skip monitoring for non-analytics endpoints
		if !isAnalyticsEndpoint(c.Request.URL.Path) {
			c.Next()
			return
		}

		start := time.Now()
		endpoint := getEndpointName(c.Request.URL.Path)
		
		// Extract relevant parameters
		weeks := c.DefaultQuery("weeks", "12")
		limit := c.DefaultQuery("limit", "100")
		format := c.DefaultQuery("format", "json")
		
		// Set up context for tracking cache performance
		ctx := context.WithValue(c.Request.Context(), "monitoring_start", start)
		ctx = context.WithValue(ctx, "endpoint", endpoint)
		c.Request = c.Request.WithContext(ctx)

		// Process request
		c.Next()

		// Record metrics after request completion
		duration := time.Since(start).Seconds()
		status := strconv.Itoa(c.Writer.Status())

		// Record request duration and count
		analyticsRequestDuration.WithLabelValues(endpoint, weeks, status).Observe(duration)
		analyticsRequestTotal.WithLabelValues(endpoint, status).Inc()

		// Log performance warnings
		if duration > 3.0 { // More than 3 seconds
			log.Printf("PERFORMANCE WARNING: Analytics request took %.2fs - endpoint: %s, weeks: %s, status: %s",
				duration, endpoint, weeks, status)
		}

		// Record specific endpoint metrics
		recordSpecificEndpointMetrics(endpoint, c, duration, weeks, limit, format)
	}
}

// isAnalyticsEndpoint checks if the request path is for analytics functionality
func isAnalyticsEndpoint(path string) bool {
	analyticsEndpoints := []string{
		"/api/v1/users/rotation/stats",
		"/api/v1/users/rotation/reset", 
		"/api/v1/users/rotation/export",
		"/api/v1/users/rotation/debug-logs",
	}
	
	for _, endpoint := range analyticsEndpoints {
		if path == endpoint {
			return true
		}
	}
	return false
}

// getEndpointName extracts a clean endpoint name for metrics
func getEndpointName(path string) string {
	switch path {
	case "/api/v1/users/rotation/stats":
		return "rotation_stats"
	case "/api/v1/users/rotation/reset":
		return "rotation_reset"
	case "/api/v1/users/rotation/export":
		return "rotation_export"
	case "/api/v1/users/rotation/debug-logs":
		return "debug_logs"
	default:
		return "unknown"
	}
}

// recordSpecificEndpointMetrics records metrics specific to each endpoint type
func recordSpecificEndpointMetrics(endpoint string, c *gin.Context, duration float64, weeks, limit, format string) {
	switch endpoint {
	case "rotation_reset":
		preservePatterns := c.DefaultPostForm("preservePatterns", "true")
		preserveFavorites := c.DefaultPostForm("preserveFavorites", "true")
		rotationResetTotal.WithLabelValues(preservePatterns, preserveFavorites).Inc()
		
	case "rotation_export":
		includesDebugLogs := c.DefaultQuery("includeDebugLogs", "false")
		analyticsExportTotal.WithLabelValues(format, includesDebugLogs).Inc()
		
	case "debug_logs":
		limitRange := getLimitRange(limit)
		debugLogsRequestDuration.WithLabelValues(limitRange).Observe(duration)
	}
}

// getLimitRange categorizes debug log limits for metrics
func getLimitRange(limitStr string) string {
	limit, err := strconv.Atoi(limitStr)
	if err != nil {
		return "unknown"
	}
	
	switch {
	case limit <= 50:
		return "small"
	case limit <= 100:
		return "medium" 
	case limit <= 500:
		return "large"
	default:
		return "xlarge"
	}
}

// RecordCacheHit records a cache hit for analytics data
func RecordCacheHit(cacheType string) {
	analyticsCacheHits.WithLabelValues(cacheType).Inc()
}

// RecordCacheMiss records a cache miss for analytics data  
func RecordCacheMiss(cacheType string) {
	analyticsCacheMisses.WithLabelValues(cacheType).Inc()
}

// RecordCalculationDuration records the duration of an analytics calculation
func RecordCalculationDuration(calculationType string, duration time.Duration) {
	analyticsCalculationDuration.WithLabelValues(calculationType).Observe(duration.Seconds())
}

// PerformanceTracker provides a context-aware way to track calculation performance
type PerformanceTracker struct {
	calculationType string
	startTime       time.Time
}

// NewPerformanceTracker creates a new performance tracker
func NewPerformanceTracker(calculationType string) *PerformanceTracker {
	return &PerformanceTracker{
		calculationType: calculationType,
		startTime:       time.Now(),
	}
}

// Finish completes the performance tracking and records metrics
func (pt *PerformanceTracker) Finish() {
	duration := time.Since(pt.startTime)
	RecordCalculationDuration(pt.calculationType, duration)
	
	// Log slow calculations
	if duration > 500*time.Millisecond {
		log.Printf("SLOW CALCULATION: %s took %.2fs", pt.calculationType, duration.Seconds())
	}
}

// GetAnalyticsMetricsSnapshot returns current analytics performance metrics
func GetAnalyticsMetricsSnapshot() map[string]interface{} {
	// This would typically gather metrics from Prometheus
	// For now, return a placeholder structure
	return map[string]interface{}{
		"requests_total":        "gathered from prometheus",
		"average_response_time": "gathered from prometheus", 
		"cache_hit_ratio":       "gathered from prometheus",
		"slow_requests_count":   "gathered from prometheus",
	}
}

// Custom context keys for monitoring
type contextKey string

const (
	MonitoringStartKey contextKey = "monitoring_start"
	EndpointKey       contextKey = "endpoint"
	CacheTypeKey      contextKey = "cache_type"
)