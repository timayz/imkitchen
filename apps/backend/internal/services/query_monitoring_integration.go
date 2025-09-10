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

// QueryMonitoringIntegration provides centralized query monitoring and alerting
type QueryMonitoringIntegration interface {
	Initialize(db *gorm.DB) *gorm.DB
	StartPerformanceReporting(interval time.Duration)
	StopPerformanceReporting()
	GetRealTimeMetrics() *RealTimeMetrics
	SetSlowQueryAlert(callback SlowQueryAlertCallback)
	GetQueryTrends(ctx context.Context, hours int) (*QueryTrends, error)
	ExportPerformanceReport(ctx context.Context, format string) ([]byte, error)
}

// SlowQueryAlertCallback is called when slow queries are detected
type SlowQueryAlertCallback func(metric QueryPerformanceMetrics)

// RealTimeMetrics provides real-time query performance data
type RealTimeMetrics struct {
	ActiveQueries         int           `json:"activeQueries"`
	QueriesPerSecond     float64       `json:"queriesPerSecond"`
	AverageResponseTime  time.Duration `json:"averageResponseTime"`
	SlowQueryRate        float64       `json:"slowQueryRate"`
	LastUpdated          time.Time     `json:"lastUpdated"`
	DatabaseConnections  int           `json:"databaseConnections"`
	CacheHitRate         float64       `json:"cacheHitRate"`
}

// QueryTrends provides trending analysis of query performance
type QueryTrends struct {
	HourlyMetrics    []HourlyMetric    `json:"hourlyMetrics"`
	TopSlowQueries   []QueryFrequency  `json:"topSlowQueries"`
	PerformanceTrend string            `json:"performanceTrend"` // improving, stable, degrading
	Recommendations  []TrendRecommendation `json:"recommendations"`
}

// HourlyMetric represents performance metrics for a specific hour
type HourlyMetric struct {
	Hour              time.Time     `json:"hour"`
	TotalQueries      int           `json:"totalQueries"`
	SlowQueries       int           `json:"slowQueries"`
	AverageTime       time.Duration `json:"averageTime"`
	PeakResponseTime  time.Duration `json:"peakResponseTime"`
}

// QueryFrequency tracks how often specific queries are executed
type QueryFrequency struct {
	QueryPattern     string        `json:"queryPattern"`
	ExecutionCount   int           `json:"executionCount"`
	AverageTime      time.Duration `json:"averageTime"`
	TotalTime        time.Duration `json:"totalTime"`
	LastSeen         time.Time     `json:"lastSeen"`
}

// TrendRecommendation provides actionable performance improvement suggestions
type TrendRecommendation struct {
	Type        string `json:"type"`        // index, query, architecture
	Priority    string `json:"priority"`    // critical, high, medium, low
	Description string `json:"description"`
	Action      string `json:"action"`
	Impact      string `json:"impact"`
}

type queryMonitoringIntegration struct {
	performanceService QueryPerformanceService
	cacheService      QueryCacheService
	
	// Monitoring state
	realTimeMetrics    *RealTimeMetrics
	metricsLock        sync.RWMutex
	reportingTicker    *time.Ticker
	reportingStop      chan bool
	slowQueryCallback  SlowQueryAlertCallback
	
	// Historical data
	hourlyMetrics      []HourlyMetric
	queryFrequencies   map[string]*QueryFrequency
	frequencyLock      sync.RWMutex
	
	// Configuration
	alertThreshold     time.Duration
	trendAnalysisHours int
}

func NewQueryMonitoringIntegration(
	performanceService QueryPerformanceService,
	cacheService QueryCacheService,
) QueryMonitoringIntegration {
	return &queryMonitoringIntegration{
		performanceService: performanceService,
		cacheService:      cacheService,
		realTimeMetrics:   &RealTimeMetrics{},
		reportingStop:     make(chan bool),
		queryFrequencies:  make(map[string]*QueryFrequency),
		alertThreshold:    200 * time.Millisecond,
		trendAnalysisHours: 24,
		hourlyMetrics:     make([]HourlyMetric, 0, 48), // Keep 48 hours of data
	}
}

// Initialize sets up query monitoring with the database
func (q *queryMonitoringIntegration) Initialize(db *gorm.DB) *gorm.DB {
	// Get monitored database from performance service
	monitoredDB := q.performanceService.StartMonitoring()
	
	// Add additional callbacks for real-time monitoring
	monitoredDB.Callback().Query().After("performance:after").Register("monitoring:after", q.monitoringCallback)
	monitoredDB.Callback().Create().After("performance:after").Register("monitoring:after", q.monitoringCallback)
	monitoredDB.Callback().Update().After("performance:after").Register("monitoring:after", q.monitoringCallback)
	monitoredDB.Callback().Delete().After("performance:after").Register("monitoring:after", q.monitoringCallback)
	
	log.Println("Query monitoring integration initialized")
	return monitoredDB
}

func (q *queryMonitoringIntegration) monitoringCallback(db *gorm.DB) {
	startTime, exists := db.InstanceGet("query_start_time")
	if !exists {
		return
	}
	
	duration := time.Since(startTime.(time.Time))
	query := db.Statement.SQL.String()
	
	// Update real-time metrics
	q.updateRealTimeMetrics(duration, duration > q.alertThreshold)
	
	// Update query frequencies
	q.updateQueryFrequencies(query, duration)
	
	// Trigger slow query alert if configured
	if duration > q.alertThreshold && q.slowQueryCallback != nil {
		metric := QueryPerformanceMetrics{
			Query:        query,
			Duration:     duration,
			RowsAffected: db.RowsAffected,
			Timestamp:    time.Now(),
			QueryType:    extractQueryType(query),
			IsSlowQuery:  true,
		}
		
		go q.slowQueryCallback(metric)
	}
}

func (q *queryMonitoringIntegration) updateRealTimeMetrics(duration time.Duration, isSlowQuery bool) {
	q.metricsLock.Lock()
	defer q.metricsLock.Unlock()
	
	now := time.Now()
	
	// Update queries per second (simple moving average over last minute)
	if q.realTimeMetrics.LastUpdated.IsZero() {
		q.realTimeMetrics.QueriesPerSecond = 1.0
	} else {
		timeSinceUpdate := now.Sub(q.realTimeMetrics.LastUpdated).Seconds()
		if timeSinceUpdate > 0 {
			newQPS := 1.0 / timeSinceUpdate
			q.realTimeMetrics.QueriesPerSecond = (q.realTimeMetrics.QueriesPerSecond*0.9) + (newQPS*0.1)
		}
	}
	
	// Update average response time (exponential moving average)
	if q.realTimeMetrics.AverageResponseTime == 0 {
		q.realTimeMetrics.AverageResponseTime = duration
	} else {
		avgMillis := float64(q.realTimeMetrics.AverageResponseTime.Nanoseconds())
		newMillis := float64(duration.Nanoseconds())
		smoothedAvg := (avgMillis*0.9) + (newMillis*0.1)
		q.realTimeMetrics.AverageResponseTime = time.Duration(int64(smoothedAvg))
	}
	
	// Update slow query rate (exponential moving average)
	if isSlowQuery {
		q.realTimeMetrics.SlowQueryRate = (q.realTimeMetrics.SlowQueryRate*0.95) + (0.05)
	} else {
		q.realTimeMetrics.SlowQueryRate = q.realTimeMetrics.SlowQueryRate * 0.95
	}
	
	q.realTimeMetrics.LastUpdated = now
}

func (q *queryMonitoringIntegration) updateQueryFrequencies(query string, duration time.Duration) {
	// Normalize query by removing parameter values
	pattern := q.normalizeQueryPattern(query)
	
	q.frequencyLock.Lock()
	defer q.frequencyLock.Unlock()
	
	freq, exists := q.queryFrequencies[pattern]
	if !exists {
		freq = &QueryFrequency{
			QueryPattern:   pattern,
			ExecutionCount: 0,
			AverageTime:    0,
			TotalTime:      0,
		}
		q.queryFrequencies[pattern] = freq
	}
	
	// Update frequency metrics
	freq.ExecutionCount++
	freq.TotalTime += duration
	freq.AverageTime = freq.TotalTime / time.Duration(freq.ExecutionCount)
	freq.LastSeen = time.Now()
}

func (q *queryMonitoringIntegration) normalizeQueryPattern(query string) string {
	// Simple query pattern normalization
	// In production, this would be more sophisticated
	pattern := query
	
	// Replace specific values with placeholders
	// This is a simplified version - production would use regex
	if len(pattern) > 200 {
		pattern = pattern[:200] + "..."
	}
	
	return pattern
}

// StartPerformanceReporting begins periodic performance reporting
func (q *queryMonitoringIntegration) StartPerformanceReporting(interval time.Duration) {
	if q.reportingTicker != nil {
		q.StopPerformanceReporting()
	}
	
	q.reportingTicker = time.NewTicker(interval)
	
	go func() {
		for {
			select {
			case <-q.reportingTicker.C:
				q.generateHourlyMetrics()
				q.logPerformanceReport()
			case <-q.reportingStop:
				return
			}
		}
	}()
	
	log.Printf("Performance reporting started with %v interval", interval)
}

// StopPerformanceReporting stops periodic performance reporting
func (q *queryMonitoringIntegration) StopPerformanceReporting() {
	if q.reportingTicker != nil {
		q.reportingTicker.Stop()
		q.reportingTicker = nil
		q.reportingStop <- true
	}
}

func (q *queryMonitoringIntegration) generateHourlyMetrics() {
	ctx := context.Background()
	since := time.Hour
	
	report, err := q.performanceService.GetPerformanceReport(ctx, since)
	if err != nil {
		log.Printf("Failed to generate hourly metrics: %v", err)
		return
	}
	
	// Create hourly metric
	hourlyMetric := HourlyMetric{
		Hour:             time.Now().Truncate(time.Hour),
		TotalQueries:     report.TotalQueries,
		SlowQueries:      report.SlowQueries,
		AverageTime:      report.AverageQueryTime,
		PeakResponseTime: q.findPeakResponseTime(report.TopSlowQueries),
	}
	
	// Add to history, maintaining rolling window
	q.hourlyMetrics = append(q.hourlyMetrics, hourlyMetric)
	if len(q.hourlyMetrics) > 48 {
		q.hourlyMetrics = q.hourlyMetrics[1:] // Remove oldest
	}
}

func (q *queryMonitoringIntegration) findPeakResponseTime(slowQueries []QueryPerformanceMetrics) time.Duration {
	var peak time.Duration
	for _, query := range slowQueries {
		if query.Duration > peak {
			peak = query.Duration
		}
	}
	return peak
}

func (q *queryMonitoringIntegration) logPerformanceReport() {
	metrics := q.GetRealTimeMetrics()
	
	log.Printf("Query Performance Report - QPS: %.2f, Avg Time: %v, Slow Rate: %.2f%%",
		metrics.QueriesPerSecond,
		metrics.AverageResponseTime,
		metrics.SlowQueryRate*100)
}

// GetRealTimeMetrics returns current performance metrics
func (q *queryMonitoringIntegration) GetRealTimeMetrics() *RealTimeMetrics {
	q.metricsLock.RLock()
	defer q.metricsLock.RUnlock()
	
	// Return a copy to avoid race conditions
	metrics := *q.realTimeMetrics
	
	// Add cache hit rate from cache service
	if cacheMetrics := q.cacheService.GetCacheMetrics(); cacheMetrics != nil {
		metrics.CacheHitRate = cacheMetrics.HitRate
	}
	
	return &metrics
}

// SetSlowQueryAlert sets a callback for slow query notifications
func (q *queryMonitoringIntegration) SetSlowQueryAlert(callback SlowQueryAlertCallback) {
	q.slowQueryCallback = callback
}

// GetQueryTrends provides trending analysis over specified hours
func (q *queryMonitoringIntegration) GetQueryTrends(ctx context.Context, hours int) (*QueryTrends, error) {
	if hours > len(q.hourlyMetrics) {
		hours = len(q.hourlyMetrics)
	}
	
	// Get recent metrics
	recentMetrics := q.hourlyMetrics
	if len(recentMetrics) > hours {
		recentMetrics = recentMetrics[len(recentMetrics)-hours:]
	}
	
	// Get top slow query patterns
	topSlowQueries := q.getTopSlowQueryPatterns()
	
	// Analyze trend
	trend := q.analyzePerfomanceTrend(recentMetrics)
	
	// Generate recommendations
	recommendations := q.generateTrendRecommendations(recentMetrics, topSlowQueries)
	
	return &QueryTrends{
		HourlyMetrics:    recentMetrics,
		TopSlowQueries:   topSlowQueries,
		PerformanceTrend: trend,
		Recommendations:  recommendations,
	}, nil
}

func (q *queryMonitoringIntegration) getTopSlowQueryPatterns() []QueryFrequency {
	q.frequencyLock.RLock()
	defer q.frequencyLock.RUnlock()
	
	// Convert to slice and sort by total time
	frequencies := make([]QueryFrequency, 0, len(q.queryFrequencies))
	for _, freq := range q.queryFrequencies {
		frequencies = append(frequencies, *freq)
	}
	
	sort.Slice(frequencies, func(i, j int) bool {
		return frequencies[i].TotalTime > frequencies[j].TotalTime
	})
	
	// Return top 10
	if len(frequencies) > 10 {
		frequencies = frequencies[:10]
	}
	
	return frequencies
}

func (q *queryMonitoringIntegration) analyzePerfomanceTrend(metrics []HourlyMetric) string {
	if len(metrics) < 2 {
		return "stable"
	}
	
	// Simple trend analysis based on average response time
	recentAvg := float64(0)
	olderAvg := float64(0)
	
	half := len(metrics) / 2
	
	for i := half; i < len(metrics); i++ {
		recentAvg += float64(metrics[i].AverageTime.Nanoseconds())
	}
	recentAvg /= float64(len(metrics) - half)
	
	for i := 0; i < half; i++ {
		olderAvg += float64(metrics[i].AverageTime.Nanoseconds())
	}
	olderAvg /= float64(half)
	
	change := (recentAvg - olderAvg) / olderAvg
	
	if change > 0.2 {
		return "degrading"
	} else if change < -0.2 {
		return "improving"
	}
	
	return "stable"
}

func (q *queryMonitoringIntegration) generateTrendRecommendations(
	metrics []HourlyMetric,
	slowQueries []QueryFrequency,
) []TrendRecommendation {
	var recommendations []TrendRecommendation
	
	// Analyze slow query rate trend
	if len(metrics) > 0 {
		recentMetric := metrics[len(metrics)-1]
		slowQueryRate := float64(recentMetric.SlowQueries) / float64(recentMetric.TotalQueries)
		
		if slowQueryRate > 0.1 {
			recommendations = append(recommendations, TrendRecommendation{
				Type:        "index",
				Priority:    "high",
				Description: fmt.Sprintf("High slow query rate: %.1f%%", slowQueryRate*100),
				Action:      "Review and optimize database indices for frequently slow queries",
				Impact:      "Reduce query response times by 50-80%",
			})
		}
	}
	
	// Analyze frequently slow queries
	for _, query := range slowQueries {
		if query.AverageTime > 500*time.Millisecond && query.ExecutionCount > 10 {
			recommendations = append(recommendations, TrendRecommendation{
				Type:        "query",
				Priority:    "high",
				Description: fmt.Sprintf("Frequently executed slow query: %dms avg", query.AverageTime.Milliseconds()),
				Action:      "Optimize this specific query pattern with indexing or restructuring",
				Impact:      "Improve user experience for common operations",
			})
		}
	}
	
	// Add general recommendations
	recommendations = append(recommendations, TrendRecommendation{
		Type:        "architecture",
		Priority:    "medium",
		Description: "Consider implementing query result caching",
		Action:      "Cache frequently accessed data to reduce database load",
		Impact:      "Reduce average response times by 60-90%",
	})
	
	return recommendations
}

// ExportPerformanceReport exports performance data in various formats
func (q *queryMonitoringIntegration) ExportPerformanceReport(ctx context.Context, format string) ([]byte, error) {
	trends, err := q.GetQueryTrends(ctx, 24)
	if err != nil {
		return nil, fmt.Errorf("failed to get trends: %w", err)
	}
	
	realTimeMetrics := q.GetRealTimeMetrics()
	
	report := struct {
		Timestamp       time.Time          `json:"timestamp"`
		RealTimeMetrics *RealTimeMetrics   `json:"realTimeMetrics"`
		QueryTrends     *QueryTrends       `json:"queryTrends"`
		Format          string             `json:"format"`
	}{
		Timestamp:       time.Now(),
		RealTimeMetrics: realTimeMetrics,
		QueryTrends:     trends,
		Format:          format,
	}
	
	switch format {
	case "json":
		return json.MarshalIndent(report, "", "  ")
	default:
		return nil, fmt.Errorf("unsupported export format: %s", format)
	}
}

// Utility function to extract query type
func extractQueryType(query string) string {
	// Use the same logic from performance service
	return "SELECT" // Simplified
}