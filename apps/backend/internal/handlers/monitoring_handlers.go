package handlers

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/middleware"
	"github.com/imkitchen/backend/internal/services"
)

// MonitoringHandlers manages HTTP endpoints for database monitoring
type MonitoringHandlers struct {
	dashboard           services.DatabasePerformanceDashboard
	metricsCollector    services.QueryPerformanceMetricsCollector
	slowQueryLogger     services.DatabaseSlowQueryLogger
	alerting            services.DatabasePerformanceAlerting
	recommendations     services.QueryOptimizationRecommendations
	rateLimiter         *middleware.MonitoringRateLimiter
}

// NewMonitoringHandlers creates new monitoring handlers
func NewMonitoringHandlers(
	dashboard services.DatabasePerformanceDashboard,
	metricsCollector services.QueryPerformanceMetricsCollector,
	slowQueryLogger services.DatabaseSlowQueryLogger,
	alerting services.DatabasePerformanceAlerting,
	recommendations services.QueryOptimizationRecommendations,
) *MonitoringHandlers {
	return &MonitoringHandlers{
		dashboard:        dashboard,
		metricsCollector: metricsCollector,
		slowQueryLogger:  slowQueryLogger,
		alerting:         alerting,
		recommendations:  recommendations,
		rateLimiter:      middleware.NewMonitoringRateLimiter(),
	}
}

// RegisterRoutes sets up monitoring routes with authentication and rate limiting
func (h *MonitoringHandlers) RegisterRoutes(router *gin.RouterGroup) {
	// Apply authentication and rate limiting to all monitoring routes
	monitoring := router.Group("/monitoring")
	monitoring.Use(middleware.MonitoringAuthMiddleware())
	monitoring.Use(h.rateLimiter.MonitoringRateLimit())

	// Dashboard endpoints
	dashboard := monitoring.Group("/dashboard")
	{
		dashboard.GET("/", h.GetDashboardData)
		dashboard.GET("/:dashboardId", h.GetSpecificDashboard)
		dashboard.GET("/:dashboardId/widgets", h.GetDashboardWidgets)
		dashboard.POST("/:dashboardId/refresh", h.RefreshDashboard)
		dashboard.GET("/:dashboardId/realtime/:widgetId", h.GetRealTimeData)
	}

	// Metrics endpoints
	metrics := monitoring.Group("/metrics")
	{
		metrics.GET("/current", h.GetCurrentMetrics)
		metrics.GET("/trends", h.GetMetricsTrends)
		metrics.GET("/aggregated", h.GetAggregatedMetrics)
		metrics.GET("/export", h.ExportMetrics)
		metrics.GET("/health", h.GetCollectorHealth)
	}

	// Slow query endpoints
	queries := monitoring.Group("/queries")
	{
		queries.GET("/slow", h.GetSlowQueries)
		queries.GET("/slow/:queryId/analysis", h.AnalyzeSlowQuery)
		queries.GET("/slow/statistics", h.GetSlowQueryStatistics)
		queries.GET("/top", h.GetTopSlowQueries)
	}

	// Alerting endpoints
	alerts := monitoring.Group("/alerts")
	{
		alerts.GET("/", h.GetAlerts)
		alerts.GET("/active", h.GetActiveAlerts)
		alerts.GET("/:alertId", h.GetAlert)
		alerts.POST("/:alertId/acknowledge", h.AcknowledgeAlert)
	}

	// Recommendations endpoints
	recommendations := monitoring.Group("/recommendations")
	{
		recommendations.GET("/", h.GetRecommendations)
		recommendations.GET("/queries/:queryId", h.GetQueryRecommendations)
		recommendations.POST("/queries/:queryId/apply", h.ApplyRecommendation)
	}

	// Admin endpoints (require admin authentication)
	admin := monitoring.Group("/admin")
	admin.Use(middleware.AdminMonitoringAuthMiddleware())
	{
		admin.GET("/stats", h.GetMonitoringStats)
		admin.POST("/config", h.UpdateMonitoringConfig)
		admin.POST("/reset", h.ResetMetrics)
	}
}

// Dashboard handlers
func (h *MonitoringHandlers) GetDashboardData(c *gin.Context) {
	timeRange := h.parseTimeRange(c)
	options := h.parseDashboardOptions(c)
	
	data, err := h.dashboard.GetDashboardData(c.Request.Context(), "default", timeRange, options)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "dashboard_error",
			"message": "Failed to retrieve dashboard data",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"data":    data,
	})
}

func (h *MonitoringHandlers) GetSpecificDashboard(c *gin.Context) {
	dashboardID := c.Param("dashboardId")
	timeRange := h.parseTimeRange(c)
	options := h.parseDashboardOptions(c)
	
	data, err := h.dashboard.GetDashboardData(c.Request.Context(), dashboardID, timeRange, options)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "dashboard_error", 
			"message": "Failed to retrieve specific dashboard",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"data":    data,
	})
}

func (h *MonitoringHandlers) GetDashboardWidgets(c *gin.Context) {
	dashboardID := c.Param("dashboardId")
	
	widgets, err := h.dashboard.GetDashboardWidgets(c.Request.Context(), dashboardID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "widgets_error",
			"message": "Failed to retrieve dashboard widgets",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"widgets": widgets,
	})
}

func (h *MonitoringHandlers) RefreshDashboard(c *gin.Context) {
	dashboardID := c.Param("dashboardId")
	
	result, err := h.dashboard.RefreshDashboard(c.Request.Context(), dashboardID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "refresh_error",
			"message": "Failed to refresh dashboard",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"result":  result,
	})
}

func (h *MonitoringHandlers) GetRealTimeData(c *gin.Context) {
	widgetID := c.Param("widgetId")
	
	data, err := h.dashboard.GetRealTimeData(c.Request.Context(), widgetID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "realtime_error",
			"message": "Failed to retrieve real-time data",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"data":    data,
	})
}

// Metrics handlers
func (h *MonitoringHandlers) GetCurrentMetrics(c *gin.Context) {
	metrics, err := h.metricsCollector.GetCurrentMetrics(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "metrics_error",
			"message": "Failed to retrieve current metrics",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"metrics": metrics,
	})
}

func (h *MonitoringHandlers) GetMetricsTrends(c *gin.Context) {
	timeWindow := h.parseTimeWindow(c, time.Hour) // Default 1 hour
	
	trends, err := h.metricsCollector.GetMetricsTrends(c.Request.Context(), timeWindow)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "trends_error",
			"message": "Failed to retrieve metrics trends",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"trends":  trends,
		"window":  timeWindow.String(),
	})
}

func (h *MonitoringHandlers) GetAggregatedMetrics(c *gin.Context) {
	timeWindow := h.parseTimeWindow(c, time.Hour)
	groupBy := h.parseGroupBy(c)
	
	metrics, err := h.metricsCollector.GetAggregatedMetrics(c.Request.Context(), timeWindow, groupBy)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "aggregation_error",
			"message": "Failed to retrieve aggregated metrics",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"metrics": metrics,
		"window":  timeWindow.String(),
		"group_by": groupBy,
	})
}

func (h *MonitoringHandlers) ExportMetrics(c *gin.Context) {
	format := h.parseExportFormat(c)
	timeRange := h.parseExportTimeRange(c)
	
	data, err := h.metricsCollector.ExportMetrics(c.Request.Context(), format, timeRange)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "export_error",
			"message": "Failed to export metrics",
			"details": err.Error(),
		})
		return
	}

	// Set appropriate content type based on format
	switch format {
	case services.ExportFormatCSV:
		c.Header("Content-Type", "text/csv")
		c.Header("Content-Disposition", "attachment; filename=metrics.csv")
	case services.ExportFormatJSON:
		c.Header("Content-Type", "application/json")
		c.Header("Content-Disposition", "attachment; filename=metrics.json")
	default:
		c.Header("Content-Type", "application/octet-stream")
	}

	c.Data(http.StatusOK, c.GetHeader("Content-Type"), data)
}

func (h *MonitoringHandlers) GetCollectorHealth(c *gin.Context) {
	health, err := h.metricsCollector.GetCollectorHealth(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "health_error",
			"message": "Failed to retrieve collector health",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"health":  health,
	})
}

// Slow query handlers
func (h *MonitoringHandlers) GetSlowQueries(c *gin.Context) {
	since := h.parseTimeSince(c, time.Hour*24) // Default last 24 hours
	limit := h.parseLimit(c, 100)              // Default 100 queries
	
	queries, err := h.slowQueryLogger.GetSlowQueries(c.Request.Context(), since, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "slow_queries_error",
			"message": "Failed to retrieve slow queries",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"queries": queries,
		"since":   since.Format(time.RFC3339),
		"limit":   limit,
	})
}

func (h *MonitoringHandlers) GetTopSlowQueries(c *gin.Context) {
	limit := h.parseLimit(c, 20) // Default top 20
	
	queries, err := h.slowQueryLogger.GetTopSlowQueries(c.Request.Context(), limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "top_queries_error",
			"message": "Failed to retrieve top slow queries",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"queries": queries,
		"limit":   limit,
	})
}

func (h *MonitoringHandlers) GetMonitoringStats(c *gin.Context) {
	stats := h.rateLimiter.GetMonitoringStats()
	
	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"stats":   stats,
	})
}

// Helper methods for parsing request parameters
func (h *MonitoringHandlers) parseTimeRange(c *gin.Context) *services.TimeRange {
	// Implementation would parse time range from query parameters
	// For now, return a default range
	return &services.TimeRange{
		Start: time.Now().Add(-time.Hour),
		End:   time.Now(),
	}
}

func (h *MonitoringHandlers) parseDashboardOptions(c *gin.Context) *services.DashboardOptions {
	// Implementation would parse dashboard options from query parameters
	return &services.DashboardOptions{}
}

func (h *MonitoringHandlers) parseTimeWindow(c *gin.Context, defaultWindow time.Duration) time.Duration {
	windowStr := c.Query("window")
	if windowStr == "" {
		return defaultWindow
	}
	
	window, err := time.ParseDuration(windowStr)
	if err != nil {
		return defaultWindow
	}
	
	return window
}

func (h *MonitoringHandlers) parseGroupBy(c *gin.Context) services.MetricsGroupBy {
	groupBy := c.Query("group_by")
	switch groupBy {
	case "query_type":
		return services.GroupByQueryType
	case "database":
		return services.GroupByDatabase
	case "user":
		return services.GroupByUser
	default:
		return services.GroupByTime
	}
}

func (h *MonitoringHandlers) parseExportFormat(c *gin.Context) services.ExportFormat {
	format := c.Query("format")
	switch format {
	case "csv":
		return services.ExportFormatCSV
	case "json":
		return services.ExportFormatJSON
	default:
		return services.ExportFormatJSON
	}
}

func (h *MonitoringHandlers) parseExportTimeRange(c *gin.Context) services.TimeRange {
	// Implementation would parse time range for export
	return services.TimeRange{
		Start: time.Now().Add(-time.Hour * 24),
		End:   time.Now(),
	}
}

func (h *MonitoringHandlers) parseTimeSince(c *gin.Context, defaultDuration time.Duration) time.Time {
	sinceStr := c.Query("since")
	if sinceStr == "" {
		return time.Now().Add(-defaultDuration)
	}
	
	since, err := time.Parse(time.RFC3339, sinceStr)
	if err != nil {
		return time.Now().Add(-defaultDuration)
	}
	
	return since
}

func (h *MonitoringHandlers) parseLimit(c *gin.Context, defaultLimit int) int {
	limitStr := c.Query("limit")
	if limitStr == "" {
		return defaultLimit
	}
	
	limit, err := strconv.Atoi(limitStr)
	if err != nil || limit <= 0 {
		return defaultLimit
	}
	
	// Cap at reasonable maximum
	if limit > 1000 {
		return 1000
	}
	
	return limit
}

// Placeholder handlers for remaining endpoints
func (h *MonitoringHandlers) AnalyzeSlowQuery(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Slow query analysis endpoint"})
}

func (h *MonitoringHandlers) GetSlowQueryStatistics(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Slow query statistics endpoint"})
}

func (h *MonitoringHandlers) GetAlerts(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Alerts endpoint"})
}

func (h *MonitoringHandlers) GetActiveAlerts(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Active alerts endpoint"})
}

func (h *MonitoringHandlers) GetAlert(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Get alert endpoint"})
}

func (h *MonitoringHandlers) AcknowledgeAlert(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Acknowledge alert endpoint"})
}

func (h *MonitoringHandlers) GetRecommendations(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Recommendations endpoint"})
}

func (h *MonitoringHandlers) GetQueryRecommendations(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Query recommendations endpoint"})
}

func (h *MonitoringHandlers) ApplyRecommendation(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Apply recommendation endpoint"})
}

func (h *MonitoringHandlers) UpdateMonitoringConfig(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Update monitoring config endpoint"})
}

func (h *MonitoringHandlers) ResetMetrics(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"message": "Reset metrics endpoint"})
}