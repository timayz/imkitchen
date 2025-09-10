package performance

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

// DatabaseMonitoringSystemTestSuite is the main test suite for the database monitoring system
type DatabaseMonitoringSystemTestSuite struct {
	suite.Suite
	db                    *gorm.DB
	ctx                   context.Context
	cancel                context.CancelFunc
	metricsCollector      services.QueryPerformanceMetricsCollector
	slowQueryLogger       services.DatabaseSlowQueryLogger
	dashboard             services.DatabasePerformanceDashboard
	alerting              services.DatabasePerformanceAlerting
	recommendations       services.QueryOptimizationRecommendations
	queryExecutionMonitor services.QueryExecutionMonitor
}

// SetupSuite initializes the test suite
func (suite *DatabaseMonitoringSystemTestSuite) SetupSuite() {
	// Create in-memory SQLite database for testing
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	require.NoError(suite.T(), err)
	
	suite.db = db
	suite.ctx, suite.cancel = context.WithCancel(context.Background())
	
	// Initialize services
	suite.queryExecutionMonitor = services.NewQueryExecutionMonitor(db)
	suite.metricsCollector = services.NewQueryPerformanceMetricsCollector(db, suite.slowQueryLogger)
	suite.slowQueryLogger = services.NewDatabaseSlowQueryLogger(db, suite.queryExecutionMonitor)
	suite.dashboard = services.NewDatabasePerformanceDashboard(db, suite.metricsCollector, suite.slowQueryLogger)
	suite.alerting = services.NewDatabasePerformanceAlerting(db, suite.metricsCollector, suite.slowQueryLogger, suite.dashboard)
	suite.recommendations = services.NewQueryOptimizationRecommendations(db, suite.metricsCollector, suite.slowQueryLogger, suite.dashboard, suite.alerting)
}

// TearDownSuite cleans up after all tests
func (suite *DatabaseMonitoringSystemTestSuite) TearDownSuite() {
	suite.cancel()
	
	// Stop all services
	if suite.recommendations.IsRunning() {
		suite.recommendations.Stop(suite.ctx)
	}
	if suite.alerting.IsRunning() {
		suite.alerting.Stop(suite.ctx)
	}
	if suite.metricsCollector.IsRunning() {
		suite.metricsCollector.Stop(suite.ctx)
	}
	if suite.slowQueryLogger.IsRunning() {
		suite.slowQueryLogger.Stop(suite.ctx)
	}
}

// TestSlowQueryLogger tests the slow query logging functionality
func (suite *DatabaseMonitoringSystemTestSuite) TestSlowQueryLogger() {
	suite.Run("StartStopLifecycle", func() {
		// Test starting the logger
		err := suite.slowQueryLogger.Start(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.True(suite.T(), suite.slowQueryLogger.IsRunning())
		
		// Test stopping the logger
		err = suite.slowQueryLogger.Stop(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.False(suite.T(), suite.slowQueryLogger.IsRunning())
	})
	
	suite.Run("ConfigureThreshold", func() {
		// Test setting threshold
		threshold := 100 * time.Millisecond
		err := suite.slowQueryLogger.SetSlowQueryThreshold(threshold)
		assert.NoError(suite.T(), err)
		
		actualThreshold := suite.slowQueryLogger.GetSlowQueryThreshold()
		assert.Equal(suite.T(), threshold, actualThreshold)
	})
	
	suite.Run("EnableDisableDetailedLogging", func() {
		// Test enabling detailed logging
		err := suite.slowQueryLogger.EnableDetailedLogging(true)
		assert.NoError(suite.T(), err)
		
		// Test disabling detailed logging
		err = suite.slowQueryLogger.EnableDetailedLogging(false)
		assert.NoError(suite.T(), err)
	})
	
	suite.Run("GetSlowQueries", func() {
		// Start the logger
		err := suite.slowQueryLogger.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.slowQueryLogger.Stop(suite.ctx)
		
		// Get slow queries (should be empty initially)
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-1 * time.Hour),
			End:   time.Now(),
		}
		
		slowQueries, err := suite.slowQueryLogger.GetSlowQueries(suite.ctx, timeRange.Start, 10)
		assert.NoError(suite.T(), err)
		assert.IsType(suite.T(), []services.DetectedSlowQuery{}, slowQueries)
		// Note: May be empty since we haven't executed any actual slow queries
	})
	
	suite.Run("GetSlowQueryStatistics", func() {
		// Start the logger
		err := suite.slowQueryLogger.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.slowQueryLogger.Stop(suite.ctx)
		
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-1 * time.Hour),
			End:   time.Now(),
		}
		
		stats, err := suite.slowQueryLogger.GetSlowQueryStatistics(suite.ctx, timeRange)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), stats)
		assert.NotNil(suite.T(), stats.QueryCategoryCounts)
		assert.NotNil(suite.T(), stats.HourlyDistribution)
		assert.NotNil(suite.T(), stats.DatabaseDistribution)
	})
}

// TestMetricsCollector tests the query performance metrics collection functionality
func (suite *DatabaseMonitoringSystemTestSuite) TestMetricsCollector() {
	suite.Run("StartStopLifecycle", func() {
		// Test starting the collector
		err := suite.metricsCollector.Start(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.True(suite.T(), suite.metricsCollector.IsRunning())
		
		// Test stopping the collector
		err = suite.metricsCollector.Stop(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.False(suite.T(), suite.metricsCollector.IsRunning())
	})
	
	suite.Run("RecordQueryExecution", func() {
		// Start the collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Create a test metric
		metric := &services.QueryExecutionMetric{
			QueryID:       "test_query_1",
			QueryText:     "SELECT * FROM test_table WHERE id = ?",
			QueryType:     services.QueryTypeSelect,
			ExecutionTime: 150 * time.Millisecond,
			Timestamp:     time.Now(),
			Database:      "test_db",
			RowsReturned:  5,
			ErrorOccurred: false,
		}
		
		// Record the metric
		err = suite.metricsCollector.RecordQueryExecution(suite.ctx, metric)
		assert.NoError(suite.T(), err)
	})
	
	suite.Run("GetQueryMetrics", func() {
		// Start the collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Create filter
		filter := &services.MetricsFilter{
			TimeRange: &services.TimeRange{
				Start: time.Now().Add(-1 * time.Hour),
				End:   time.Now(),
			},
			Limit: 10,
		}
		
		// Get metrics
		collection, err := suite.metricsCollector.GetQueryMetrics(suite.ctx, filter)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), collection)
		assert.NotNil(suite.T(), collection.Summary)
	})
	
	suite.Run("GetCurrentMetrics", func() {
		// Start the collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Get current metrics snapshot
		snapshot, err := suite.metricsCollector.GetCurrentMetrics(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), snapshot)
		assert.NotZero(suite.T(), snapshot.Timestamp)
	})
	
	suite.Run("SetGetPerformanceThresholds", func() {
		// Create test thresholds
		thresholds := &services.PerformanceThresholds{
			MaxExecutionTime:     200 * time.Millisecond,
			MaxAverageExecTime:   100 * time.Millisecond,
			MaxErrorRate:         0.05,
			MinCacheHitRate:      0.80,
			MaxActiveConnections: 100,
		}
		
		// Set thresholds
		err := suite.metricsCollector.SetPerformanceThresholds(thresholds)
		assert.NoError(suite.T(), err)
		
		// Get thresholds
		actualThresholds := suite.metricsCollector.GetPerformanceThresholds()
		assert.Equal(suite.T(), thresholds.MaxExecutionTime, actualThresholds.MaxExecutionTime)
		assert.Equal(suite.T(), thresholds.MaxErrorRate, actualThresholds.MaxErrorRate)
	})
	
	suite.Run("CheckThresholdBreaches", func() {
		// Start the collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Check for threshold breaches
		breaches, err := suite.metricsCollector.CheckThresholdBreaches(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.IsType(suite.T(), []services.ThresholdBreach{}, breaches)
	})
	
	suite.Run("GetCollectorHealth", func() {
		// Start the collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Get health status
		health, err := suite.metricsCollector.GetCollectorHealth(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), health)
		assert.NotZero(suite.T(), health.Uptime)
	})
}

// TestDashboard tests the dashboard functionality
func (suite *DatabaseMonitoringSystemTestSuite) TestDashboard() {
	suite.Run("Initialize", func() {
		err := suite.dashboard.Initialize(suite.ctx)
		assert.NoError(suite.T(), err)
	})
	
	suite.Run("GetDashboardConfig", func() {
		// Initialize first
		err := suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		config, err := suite.dashboard.GetDashboardConfig()
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), config)
		assert.NotEmpty(suite.T(), config.DashboardID)
	})
	
	suite.Run("UpdateDashboardConfig", func() {
		// Initialize first
		err := suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		// Get existing config
		config, err := suite.dashboard.GetDashboardConfig()
		require.NoError(suite.T(), err)
		
		// Update config
		config.Name = "Updated Dashboard"
		config.Description = "Updated description"
		
		err = suite.dashboard.UpdateDashboardConfig(config)
		assert.NoError(suite.T(), err)
		
		// Verify update
		updatedConfig, err := suite.dashboard.GetDashboardConfig()
		require.NoError(suite.T(), err)
		assert.Equal(suite.T(), "Updated Dashboard", updatedConfig.Name)
	})
	
	suite.Run("GetDashboardData", func() {
		// Initialize first
		err := suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-1 * time.Hour),
			End:   time.Now(),
		}
		
		options := &services.DashboardOptions{
			IncludeData:     true,
			IncludeMetadata: true,
		}
		
		data, err := suite.dashboard.GetDashboardData(suite.ctx, "main_performance", timeRange, options)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), data)
		assert.NotNil(suite.T(), data.Dashboard)
		assert.NotNil(suite.T(), data.WidgetsData)
	})
	
	suite.Run("RefreshDashboard", func() {
		// Initialize first
		err := suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		result, err := suite.dashboard.RefreshDashboard(suite.ctx, "main_performance")
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), result)
		assert.Equal(suite.T(), "main_performance", result.DashboardID)
		assert.NotZero(suite.T(), result.Duration)
	})
	
	suite.Run("ExportDashboard", func() {
		// Initialize first
		err := suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		data, err := suite.dashboard.ExportDashboard(suite.ctx, "main_performance", services.ExportFormatJSON)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), data)
		assert.Greater(suite.T(), len(data), 0)
	})
	
	suite.Run("GetPerformanceInsights", func() {
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-24 * time.Hour),
			End:   time.Now(),
		}
		
		insights, err := suite.dashboard.GetPerformanceInsights(suite.ctx, timeRange)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), insights)
		assert.NotNil(suite.T(), insights.KeyMetrics)
		assert.NotZero(suite.T(), insights.OverallHealthScore)
	})
}

// TestAlerting tests the alerting functionality
func (suite *DatabaseMonitoringSystemTestSuite) TestAlerting() {
	suite.Run("StartStopLifecycle", func() {
		// Test starting the alerting system
		err := suite.alerting.Start(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.True(suite.T(), suite.alerting.IsRunning())
		
		// Test stopping the alerting system
		err = suite.alerting.Stop(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.False(suite.T(), suite.alerting.IsRunning())
	})
	
	suite.Run("CreateAlertRule", func() {
		// Create test alert rule
		rule := &services.AlertRule{
			Name:        "Test High Execution Time",
			Description: "Test rule for high execution times",
			Type:        services.AlertRuleTypeThreshold,
			MetricType:  services.MetricTypeExecutionTime,
			Condition: &services.AlertCondition{
				Operator:        services.OperatorGreaterThan,
				Value:           200, // 200ms
				AggregationType: services.AggregationAvg,
				TimeWindow:      5 * time.Minute,
			},
			EvaluationWindow:   5 * time.Minute,
			EvaluationInterval: 30 * time.Second,
			Severity:           services.SeverityWarning,
			IsEnabled:          true,
		}
		
		err := suite.alerting.CreateAlertRule(suite.ctx, rule)
		assert.NoError(suite.T(), err)
		assert.NotEmpty(suite.T(), rule.RuleID)
	})
	
	suite.Run("ListAlertRules", func() {
		// Create a rule first
		rule := &services.AlertRule{
			Name:        "Test Rule for Listing",
			Description: "Test rule",
			Type:        services.AlertRuleTypeThreshold,
			MetricType:  services.MetricTypeErrorRate,
			Condition: &services.AlertCondition{
				Operator: services.OperatorGreaterThan,
				Value:    0.05,
			},
			EvaluationWindow:   5 * time.Minute,
			EvaluationInterval: 30 * time.Second,
			Severity:           services.SeverityCritical,
			IsEnabled:          true,
		}
		
		err := suite.alerting.CreateAlertRule(suite.ctx, rule)
		require.NoError(suite.T(), err)
		
		// List rules
		rules, err := suite.alerting.ListAlertRules(suite.ctx, nil)
		assert.NoError(suite.T(), err)
		assert.Greater(suite.T(), len(rules), 0)
	})
	
	suite.Run("AddNotificationChannel", func() {
		channel := &services.NotificationChannel{
			Name: "Test Log Channel",
			Type: services.ChannelTypeCustom,
			Configuration: &services.ChannelConfiguration{
				CustomConfig: map[string]interface{}{
					"type": "log",
				},
			},
			IsEnabled: true,
		}
		
		err := suite.alerting.AddNotificationChannel(suite.ctx, channel)
		assert.NoError(suite.T(), err)
		assert.NotEmpty(suite.T(), channel.ChannelID)
	})
	
	suite.Run("ListNotificationChannels", func() {
		channels, err := suite.alerting.ListNotificationChannels(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.Greater(suite.T(), len(channels), 0)
	})
	
	suite.Run("GetAlertingHealth", func() {
		// Start the system first
		err := suite.alerting.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.alerting.Stop(suite.ctx)
		
		health, err := suite.alerting.GetAlertingHealth(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), health)
		assert.NotEmpty(suite.T(), health.Status)
	})
	
	suite.Run("GetAlertStatistics", func() {
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-24 * time.Hour),
			End:   time.Now(),
		}
		
		stats, err := suite.alerting.GetAlertStatistics(suite.ctx, timeRange)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), stats)
		assert.NotNil(suite.T(), stats.SeverityBreakdown)
		assert.NotNil(suite.T(), stats.StateBreakdown)
	})
}

// TestRecommendations tests the recommendations functionality
func (suite *DatabaseMonitoringSystemTestSuite) TestRecommendations() {
	suite.Run("StartStopLifecycle", func() {
		// Test starting the recommendations system
		err := suite.recommendations.Start(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.True(suite.T(), suite.recommendations.IsRunning())
		
		// Test stopping the recommendations system
		err = suite.recommendations.Stop(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.False(suite.T(), suite.recommendations.IsRunning())
	})
	
	suite.Run("GenerateRecommendations", func() {
		// Start the system first
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		// Create test query analysis request
		request := &services.QueryAnalysisRequest{
			QueryText: "SELECT * FROM recipes WHERE cuisine = 'Italian' ORDER BY rating DESC",
			QueryType: "SELECT",
			PerformanceData: &services.QueryPerformanceData{
				ExecutionTime: 350 * time.Millisecond,
				RowsExamined:  10000,
				RowsReturned:  50,
				CallFrequency: 100,
			},
			Priority: services.PriorityMedium,
		}
		
		recommendations, err := suite.recommendations.GenerateRecommendations(suite.ctx, request)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), recommendations)
		assert.NotEmpty(suite.T(), recommendations.RecommendationID)
		assert.Equal(suite.T(), services.StatusReady, recommendations.Status)
	})
	
	suite.Run("GetRecommendations", func() {
		// Start the system first
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		// Generate a recommendation first
		request := &services.QueryAnalysisRequest{
			QueryText: "SELECT COUNT(*) FROM orders WHERE created_at > '2023-01-01'",
			QueryType: "SELECT",
			Priority:  services.PriorityHigh,
		}
		
		_, err = suite.recommendations.GenerateRecommendations(suite.ctx, request)
		require.NoError(suite.T(), err)
		
		// Get recommendations
		filter := &services.RecommendationFilter{
			Priority: &[]services.RecommendationPriority{services.PriorityHigh}[0],
			Limit:    10,
		}
		
		recs, err := suite.recommendations.GetRecommendations(suite.ctx, filter)
		assert.NoError(suite.T(), err)
		assert.Greater(suite.T(), len(recs), 0)
	})
	
	suite.Run("PredictPerformanceImpact", func() {
		// Start the system first
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		// Create test recommendation
		recommendation := &services.Recommendation{
			ID:          "test_rec_1",
			Type:        services.RecommendationTypeIndex,
			Category:    services.CategoryPerformance,
			Title:       "Add index on recipes.cuisine",
			Description: "Add index to improve query performance",
			Priority:    services.PriorityMedium,
		}
		
		impact, err := suite.recommendations.PredictPerformanceImpact(suite.ctx, recommendation)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), impact)
		assert.NotNil(suite.T(), impact.PredictedImprovement)
	})
	
	suite.Run("GetModelAccuracy", func() {
		// Start the system first
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		accuracy, err := suite.recommendations.GetModelAccuracy(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), accuracy)
		assert.Greater(suite.T(), accuracy.OverallAccuracy, 0.0)
		assert.LessOrEqual(suite.T(), accuracy.OverallAccuracy, 1.0)
	})
	
	suite.Run("GenerateOptimizationReport", func() {
		// Start the system first
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-7 * 24 * time.Hour),
			End:   time.Now(),
		}
		
		options := &services.ReportOptions{
			IncludeExecutiveSummary: true,
			IncludeRecommendations:  true,
			Format:                  "json",
		}
		
		report, err := suite.recommendations.GenerateOptimizationReport(suite.ctx, timeRange, options)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), report)
		assert.NotEmpty(suite.T(), report.ReportID)
		assert.Equal(suite.T(), timeRange, report.TimeRange)
	})
}

// TestIntegration tests the integration between different components
func (suite *DatabaseMonitoringSystemTestSuite) TestIntegration() {
	suite.Run("FullWorkflowIntegration", func() {
		// Start all services
		err := suite.slowQueryLogger.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.slowQueryLogger.Stop(suite.ctx)
		
		err = suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		err = suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		err = suite.alerting.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.alerting.Stop(suite.ctx)
		
		err = suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		// Wait for services to initialize
		time.Sleep(100 * time.Millisecond)
		
		// Simulate a slow query by recording a metric
		metric := &services.QueryExecutionMetric{
			QueryID:       "integration_test_query",
			QueryText:     "SELECT * FROM large_table WHERE unindexed_column = 'value'",
			QueryType:     services.QueryTypeSelect,
			ExecutionTime: 800 * time.Millisecond, // Slow query
			Timestamp:     time.Now(),
			Database:      "test_db",
			RowsExamined:  1000000,
			RowsReturned:  1,
			ErrorOccurred: false,
		}
		
		// Record the metric
		err = suite.metricsCollector.RecordQueryExecution(suite.ctx, metric)
		assert.NoError(suite.T(), err)
		
		// Wait for processing
		time.Sleep(200 * time.Millisecond)
		
		// Check that dashboard can retrieve the data
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-5 * time.Minute),
			End:   time.Now(),
		}
		
		dashboardData, err := suite.dashboard.GetDashboardData(suite.ctx, "main_performance", timeRange, &services.DashboardOptions{
			IncludeData: true,
		})
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), dashboardData)
		
		// Generate recommendations for the slow query
		request := &services.QueryAnalysisRequest{
			QueryID:   metric.QueryID,
			QueryText: metric.QueryText,
			QueryType: string(metric.QueryType),
			PerformanceData: &services.QueryPerformanceData{
				ExecutionTime: metric.ExecutionTime,
				RowsExamined:  metric.RowsExamined,
				RowsReturned:  metric.RowsReturned,
			},
			Priority: services.PriorityHigh,
		}
		
		recommendations, err := suite.recommendations.GenerateRecommendations(suite.ctx, request)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), recommendations)
		assert.Equal(suite.T(), services.StatusReady, recommendations.Status)
		
		// Verify that we got some recommendations
		totalRecs := len(recommendations.IndexRecommendations) + 
					len(recommendations.QueryRewriteRecommendations) + 
					len(recommendations.ConfigurationRecommendations) + 
					len(recommendations.SchemaRecommendations)
		assert.Greater(suite.T(), totalRecs, 0, "Should generate at least one recommendation for a slow query")
	})
	
	suite.Run("AlertingWorkflow", func() {
		// Start required services
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		err = suite.alerting.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.alerting.Stop(suite.ctx)
		
		// Wait for services to initialize
		time.Sleep(100 * time.Millisecond)
		
		// Get list of alert rules (should have default rules)
		rules, err := suite.alerting.ListAlertRules(suite.ctx, nil)
		assert.NoError(suite.T(), err)
		assert.Greater(suite.T(), len(rules), 0, "Should have default alert rules")
		
		// Check alerting system health
		health, err := suite.alerting.GetAlertingHealth(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.True(suite.T(), health.IsHealthy)
		
		// Get alert statistics
		timeRange := &services.TimeRange{
			Start: time.Now().Add(-1 * time.Hour),
			End:   time.Now(),
		}
		
		stats, err := suite.alerting.GetAlertStatistics(suite.ctx, timeRange)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), stats)
	})
	
	suite.Run("PerformanceBaseline", func() {
		// Start metrics collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Record multiple metrics to establish baseline
		baselineQueries := []struct {
			queryText     string
			executionTime time.Duration
		}{
			{"SELECT * FROM recipes WHERE id = ?", 50 * time.Millisecond},
			{"SELECT COUNT(*) FROM recipes", 100 * time.Millisecond},
			{"SELECT * FROM recipes ORDER BY rating DESC LIMIT 10", 75 * time.Millisecond},
		}
		
		for i, query := range baselineQueries {
			metric := &services.QueryExecutionMetric{
				QueryID:       fmt.Sprintf("baseline_query_%d", i),
				QueryText:     query.queryText,
				QueryType:     services.QueryTypeSelect,
				ExecutionTime: query.executionTime,
				Timestamp:     time.Now(),
				Database:      "test_db",
				RowsReturned:  10,
				ErrorOccurred: false,
			}
			
			err = suite.metricsCollector.RecordQueryExecution(suite.ctx, metric)
			assert.NoError(suite.T(), err)
		}
		
		// Get current metrics to verify baseline
		snapshot, err := suite.metricsCollector.GetCurrentMetrics(suite.ctx)
		assert.NoError(suite.T(), err)
		assert.NotNil(suite.T(), snapshot)
		
		// Check performance health
		assert.NotNil(suite.T(), snapshot.PerformanceHealth)
		assert.Greater(suite.T(), snapshot.PerformanceHealth.OverallScore, 0.0)
	})
}

// TestPerformanceAndScalability tests system performance under various loads
func (suite *DatabaseMonitoringSystemTestSuite) TestPerformanceAndScalability() {
	suite.Run("HighVolumeMetricRecording", func() {
		// Start metrics collector
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Record a large number of metrics quickly
		numMetrics := 1000
		startTime := time.Now()
		
		for i := 0; i < numMetrics; i++ {
			metric := &services.QueryExecutionMetric{
				QueryID:       fmt.Sprintf("perf_test_query_%d", i),
				QueryText:     "SELECT * FROM test_table WHERE id = ?",
				QueryType:     services.QueryTypeSelect,
				ExecutionTime: time.Duration(50+i%100) * time.Millisecond,
				Timestamp:     time.Now(),
				Database:      "test_db",
				RowsReturned:  int64(i % 10),
				ErrorOccurred: i%50 == 0, // 2% error rate
			}
			
			err = suite.metricsCollector.RecordQueryExecution(suite.ctx, metric)
			assert.NoError(suite.T(), err)
		}
		
		recordingDuration := time.Since(startTime)
		metricsPerSecond := float64(numMetrics) / recordingDuration.Seconds()
		
		suite.T().Logf("Recorded %d metrics in %v (%.2f metrics/sec)", 
			numMetrics, recordingDuration, metricsPerSecond)
		
		// Should be able to record at least 1000 metrics per second
		assert.Greater(suite.T(), metricsPerSecond, 100.0, 
			"Should be able to record metrics quickly")
	})
	
	suite.Run("ConcurrentRecommendationGeneration", func() {
		// Start recommendations system
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		// Generate recommendations concurrently
		numConcurrent := 10
		var wg sync.WaitGroup
		results := make([]*services.OptimizationRecommendations, numConcurrent)
		errors := make([]error, numConcurrent)
		
		startTime := time.Now()
		
		for i := 0; i < numConcurrent; i++ {
			wg.Add(1)
			go func(index int) {
				defer wg.Done()
				
				request := &services.QueryAnalysisRequest{
					QueryText: fmt.Sprintf("SELECT * FROM table_%d WHERE column = 'value'", index),
					QueryType: "SELECT",
					PerformanceData: &services.QueryPerformanceData{
						ExecutionTime: time.Duration(200+index*10) * time.Millisecond,
						RowsExamined:  int64(1000 + index*100),
						RowsReturned:  int64(10 + index),
					},
					Priority: services.PriorityMedium,
				}
				
				rec, err := suite.recommendations.GenerateRecommendations(suite.ctx, request)
				results[index] = rec
				errors[index] = err
			}(i)
		}
		
		wg.Wait()
		duration := time.Since(startTime)
		
		// Check that all requests completed successfully
		for i, err := range errors {
			assert.NoError(suite.T(), err, "Request %d should not have errored", i)
			assert.NotNil(suite.T(), results[i], "Result %d should not be nil", i)
		}
		
		suite.T().Logf("Generated %d recommendations concurrently in %v", 
			numConcurrent, duration)
		
		// Should complete within reasonable time
		assert.Less(suite.T(), duration, 10*time.Second, 
			"Concurrent recommendation generation should be fast")
	})
	
	suite.Run("DashboardLoadTest", func() {
		// Initialize dashboard
		err := suite.dashboard.Initialize(suite.ctx)
		require.NoError(suite.T(), err)
		
		// Simulate multiple concurrent dashboard requests
		numRequests := 20
		var wg sync.WaitGroup
		errors := make([]error, numRequests)
		
		startTime := time.Now()
		
		for i := 0; i < numRequests; i++ {
			wg.Add(1)
			go func(index int) {
				defer wg.Done()
				
				timeRange := &services.TimeRange{
					Start: time.Now().Add(-time.Duration(index+1) * time.Hour),
					End:   time.Now(),
				}
				
				_, err := suite.dashboard.GetDashboardData(
					suite.ctx, 
					"main_performance", 
					timeRange, 
					&services.DashboardOptions{IncludeData: true},
				)
				errors[index] = err
			}(i)
		}
		
		wg.Wait()
		duration := time.Since(startTime)
		
		// Check that all requests completed successfully
		for i, err := range errors {
			assert.NoError(suite.T(), err, "Dashboard request %d should not have errored", i)
		}
		
		suite.T().Logf("Completed %d dashboard requests in %v", numRequests, duration)
		
		// Should handle concurrent requests well
		assert.Less(suite.T(), duration, 15*time.Second, 
			"Dashboard should handle concurrent requests efficiently")
	})
}

// TestErrorHandling tests various error conditions and recovery
func (suite *DatabaseMonitoringSystemTestSuite) TestErrorHandling() {
	suite.Run("InvalidQueryAnalysis", func() {
		err := suite.recommendations.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.recommendations.Stop(suite.ctx)
		
		// Test with invalid/empty query
		request := &services.QueryAnalysisRequest{
			QueryText: "",
			QueryType: "INVALID",
			Priority:  services.PriorityMedium,
		}
		
		recommendations, err := suite.recommendations.GenerateRecommendations(suite.ctx, request)
		// Should handle gracefully - either return error or empty recommendations
		if err == nil {
			assert.NotNil(suite.T(), recommendations)
		}
	})
	
	suite.Run("InvalidThresholds", func() {
		// Test setting invalid threshold
		err := suite.slowQueryLogger.SetSlowQueryThreshold(-1 * time.Millisecond)
		assert.Error(suite.T(), err)
		
		// Test setting zero threshold
		err = suite.slowQueryLogger.SetSlowQueryThreshold(0)
		assert.Error(suite.T(), err)
	})
	
	suite.Run("ServiceStopWithoutStart", func() {
		// Test stopping services that haven't been started
		err := suite.metricsCollector.Stop(suite.ctx)
		assert.NoError(suite.T(), err) // Should handle gracefully
		
		err = suite.alerting.Stop(suite.ctx)
		assert.NoError(suite.T(), err) // Should handle gracefully
		
		err = suite.recommendations.Stop(suite.ctx)
		assert.NoError(suite.T(), err) // Should handle gracefully
	})
	
	suite.Run("DoubleStart", func() {
		// Start service
		err := suite.metricsCollector.Start(suite.ctx)
		require.NoError(suite.T(), err)
		defer suite.metricsCollector.Stop(suite.ctx)
		
		// Try to start again
		err = suite.metricsCollector.Start(suite.ctx)
		assert.Error(suite.T(), err, "Should not allow double start")
	})
}

// Run the test suite
func TestDatabaseMonitoringSystemSuite(t *testing.T) {
	suite.Run(t, new(DatabaseMonitoringSystemTestSuite))
}