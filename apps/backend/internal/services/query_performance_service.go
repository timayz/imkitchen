package services

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// QueryPerformanceMetrics tracks database query performance
type ServiceQueryPerformanceMetrics struct {
	Query            string        `json:"query"`
	Duration         time.Duration `json:"duration"`
	RowsAffected     int64         `json:"rowsAffected"`
	Timestamp        time.Time     `json:"timestamp"`
	QueryType        string        `json:"queryType"` // SELECT, INSERT, UPDATE, DELETE
	IsSlowQuery      bool          `json:"isSlowQuery"`
	OptimizationHint string        `json:"optimizationHint,omitempty"`
}

// QueryPerformanceService provides database query performance monitoring and optimization
type QueryPerformanceService interface {
	StartMonitoring() *gorm.DB
	GetSlowQueries(ctx context.Context, since time.Duration) ([]QueryPerformanceMetrics, error)
	AnalyzeQuery(ctx context.Context, query string) (*QueryAnalysis, error)
	GetPerformanceReport(ctx context.Context, since time.Duration) (*PerformanceReport, error)
	OptimizeCommonQueries(ctx context.Context) error
}

// QueryAnalysis provides detailed analysis of a specific query
type QueryAnalysis struct {
	Query               string                 `json:"query"`
	EstimatedCost       float64                `json:"estimatedCost"`
	ActualCost          *float64               `json:"actualCost,omitempty"`
	ExecutionPlan       []ExecutionPlanStep    `json:"executionPlan"`
	IndexRecommendations []IndexRecommendation  `json:"indexRecommendations"`
	OptimizationTips    []string               `json:"optimizationTips"`
	IsOptimal           bool                   `json:"isOptimal"`
}

// ExecutionPlanStep represents a step in the query execution plan
type ExecutionPlanStep struct {
	NodeType    string  `json:"nodeType"`
	Relation    string  `json:"relation,omitempty"`
	Cost        float64 `json:"cost"`
	Rows        int64   `json:"rows"`
	Width       int     `json:"width"`
	Description string  `json:"description"`
}

// IndexRecommendation suggests database indices for query optimization
type ServiceIndexRecommendation struct {
	TableName   string   `json:"tableName"`
	Columns     []string `json:"columns"`
	IndexType   string   `json:"indexType"` // btree, gin, gist, hash
	Reason      string   `json:"reason"`
	Priority    string   `json:"priority"` // high, medium, low
	CreateSQL   string   `json:"createSQL"`
}

// PerformanceReport provides overall database performance insights
type ServicePerformanceReport struct {
	TotalQueries       int                     `json:"totalQueries"`
	SlowQueries        int                     `json:"slowQueries"`
	AverageQueryTime   time.Duration           `json:"averageQueryTime"`
	SlowQueryThreshold time.Duration           `json:"slowQueryThreshold"`
	TopSlowQueries     []QueryPerformanceMetrics `json:"topSlowQueries"`
	IndexUsage         []IndexUsageStats       `json:"indexUsage"`
	Recommendations    []string                `json:"recommendations"`
}

// IndexUsageStats provides statistics about index usage
type ServiceIndexUsageStats struct {
	TableName    string `json:"tableName"`
	IndexName    string `json:"indexName"`
	ScanCount    int64  `json:"scanCount"`
	TupleReads   int64  `json:"tupleReads"`
	TupleFetches int64  `json:"tupleFetches"`
	IsUnused     bool   `json:"isUnused"`
}

type queryPerformanceService struct {
	db                 *gorm.DB
	slowQueryThreshold time.Duration
	metrics            []QueryPerformanceMetrics
	maxMetricsSize     int
}

func NewQueryPerformanceService(db *gorm.DB) QueryPerformanceService {
	return &queryPerformanceService{
		db:                 db,
		slowQueryThreshold: 100 * time.Millisecond, // Queries slower than 100ms are considered slow
		metrics:            make([]QueryPerformanceMetrics, 0),
		maxMetricsSize:     1000, // Keep last 1000 queries in memory
	}
}

// StartMonitoring enables query performance monitoring
func (q *queryPerformanceService) StartMonitoring() *gorm.DB {
	// Create a custom logger that captures query performance
	customLogger := logger.New(
		log.New(log.Writer(), "\r\n", log.LstdFlags),
		logger.Config{
			SlowThreshold:             q.slowQueryThreshold,
			LogLevel:                  logger.Warn,
			IgnoreRecordNotFoundError: true,
			Colorful:                  false,
		},
	)

	// Clone the database instance with the custom logger
	monitoredDB := q.db.Session(&gorm.Session{
		Logger: customLogger,
	})

	// Add callback to capture metrics
	monitoredDB.Callback().Query().Before("gorm:query").Register("performance:before", q.beforeQueryCallback)
	monitoredDB.Callback().Query().After("gorm:query").Register("performance:after", q.afterQueryCallback)
	monitoredDB.Callback().Create().Before("gorm:create").Register("performance:before", q.beforeQueryCallback)
	monitoredDB.Callback().Create().After("gorm:create").Register("performance:after", q.afterQueryCallback)
	monitoredDB.Callback().Update().Before("gorm:update").Register("performance:before", q.beforeQueryCallback)
	monitoredDB.Callback().Update().After("gorm:update").Register("performance:after", q.afterQueryCallback)
	monitoredDB.Callback().Delete().Before("gorm:delete").Register("performance:before", q.beforeQueryCallback)
	monitoredDB.Callback().Delete().After("gorm:delete").Register("performance:after", q.afterQueryCallback)

	return monitoredDB
}

func (q *queryPerformanceService) beforeQueryCallback(db *gorm.DB) {
	db.InstanceSet("query_start_time", time.Now())
}

func (q *queryPerformanceService) afterQueryCallback(db *gorm.DB) {
	startTime, exists := db.InstanceGet("query_start_time")
	if !exists {
		return
	}

	duration := time.Since(startTime.(time.Time))
	query := db.Statement.SQL.String()
	queryType := q.extractQueryType(query)
	
	metric := QueryPerformanceMetrics{
		Query:        query,
		Duration:     duration,
		RowsAffected: db.RowsAffected,
		Timestamp:    time.Now(),
		QueryType:    queryType,
		IsSlowQuery:  duration > q.slowQueryThreshold,
	}

	// Add optimization hint for slow queries
	if metric.IsSlowQuery {
		metric.OptimizationHint = q.generateOptimizationHint(query)
	}

	q.addMetric(metric)

	// Log slow queries
	if metric.IsSlowQuery {
		log.Printf("SLOW QUERY [%v]: %s", duration, query)
	}
}

func (q *queryPerformanceService) extractQueryType(query string) string {
	query = strings.TrimSpace(strings.ToUpper(query))
	if strings.HasPrefix(query, "SELECT") {
		return "SELECT"
	} else if strings.HasPrefix(query, "INSERT") {
		return "INSERT"
	} else if strings.HasPrefix(query, "UPDATE") {
		return "UPDATE"
	} else if strings.HasPrefix(query, "DELETE") {
		return "DELETE"
	}
	return "OTHER"
}

func (q *queryPerformanceService) generateOptimizationHint(query string) string {
	query = strings.ToLower(query)
	
	if strings.Contains(query, "where") && !strings.Contains(query, "index") {
		return "Consider adding indices on WHERE clause columns"
	}
	if strings.Contains(query, "order by") && !strings.Contains(query, "limit") {
		return "Consider adding LIMIT to ORDER BY queries"
	}
	if strings.Contains(query, "group by") {
		return "Ensure GROUP BY columns are indexed"
	}
	if strings.Contains(query, "join") && strings.Contains(query, "on") {
		return "Ensure JOIN columns are indexed"
	}
	if strings.Contains(query, "like '%") {
		return "Avoid leading wildcards in LIKE queries; consider full-text search"
	}
	
	return "Review query structure and indexing strategy"
}

func (q *queryPerformanceService) addMetric(metric QueryPerformanceMetrics) {
	q.metrics = append(q.metrics, metric)
	
	// Keep only the most recent metrics
	if len(q.metrics) > q.maxMetricsSize {
		q.metrics = q.metrics[len(q.metrics)-q.maxMetricsSize:]
	}
}

// GetSlowQueries returns slow queries from the specified time period
func (q *queryPerformanceService) GetSlowQueries(ctx context.Context, since time.Duration) ([]QueryPerformanceMetrics, error) {
	cutoff := time.Now().Add(-since)
	var slowQueries []QueryPerformanceMetrics
	
	for _, metric := range q.metrics {
		if metric.Timestamp.After(cutoff) && metric.IsSlowQuery {
			slowQueries = append(slowQueries, metric)
		}
	}
	
	return slowQueries, nil
}

// AnalyzeQuery provides detailed analysis of a specific query
func (q *queryPerformanceService) AnalyzeQuery(ctx context.Context, query string) (*QueryAnalysis, error) {
	// Get execution plan using EXPLAIN
	var results []map[string]interface{}
	explainQuery := "EXPLAIN (FORMAT JSON, ANALYZE false, BUFFERS false) " + query
	
	if err := q.db.Raw(explainQuery).Scan(&results).Error; err != nil {
		return nil, fmt.Errorf("failed to get execution plan: %w", err)
	}

	analysis := &QueryAnalysis{
		Query:               query,
		ExecutionPlan:       make([]ExecutionPlanStep, 0),
		IndexRecommendations: make([]IndexRecommendation, 0),
		OptimizationTips:    make([]string, 0),
	}

	// Parse execution plan (simplified - full implementation would parse the JSON plan)
	if len(results) > 0 {
		analysis.EstimatedCost = 100.0 // Placeholder - would extract from actual plan
		analysis.IsOptimal = analysis.EstimatedCost < 1000 // Simple heuristic
	}

	// Generate recommendations based on query patterns
	recommendations := q.generateQueryRecommendations(query)
	analysis.IndexRecommendations = recommendations
	analysis.OptimizationTips = q.generateOptimizationTips(query)

	return analysis, nil
}

func (q *queryPerformanceService) generateQueryRecommendations(query string) []IndexRecommendation {
	var recommendations []IndexRecommendation
	query = strings.ToLower(query)

	// Analyze recipe search patterns
	if strings.Contains(query, "recipes") {
		if strings.Contains(query, "meal_type") {
			recommendations = append(recommendations, IndexRecommendation{
				TableName: "recipes",
				Columns:   []string{"meal_type"},
				IndexType: "gin",
				Reason:    "Improve filtering by meal_type array",
				Priority:  "high",
				CreateSQL: "CREATE INDEX IF NOT EXISTS idx_recipes_meal_type_opt ON recipes USING GIN(meal_type);",
			})
		}
		
		if strings.Contains(query, "dietary_labels") {
			recommendations = append(recommendations, IndexRecommendation{
				TableName: "recipes",
				Columns:   []string{"dietary_labels"},
				IndexType: "gin",
				Reason:    "Improve filtering by dietary restrictions",
				Priority:  "high",
				CreateSQL: "CREATE INDEX IF NOT EXISTS idx_recipes_dietary_labels_opt ON recipes USING GIN(dietary_labels);",
			})
		}
		
		if strings.Contains(query, "total_time") && strings.Contains(query, "complexity") {
			recommendations = append(recommendations, IndexRecommendation{
				TableName: "recipes",
				Columns:   []string{"total_time", "complexity"},
				IndexType: "btree",
				Reason:    "Improve filtering by time and complexity",
				Priority:  "medium",
				CreateSQL: "CREATE INDEX IF NOT EXISTS idx_recipes_time_complexity ON recipes(total_time, complexity);",
			})
		}
	}

	// Analyze meal plan patterns
	if strings.Contains(query, "meal_plans") {
		if strings.Contains(query, "user_id") && strings.Contains(query, "week_start") {
			recommendations = append(recommendations, IndexRecommendation{
				TableName: "meal_plans",
				Columns:   []string{"user_id", "week_start", "status"},
				IndexType: "btree",
				Reason:    "Improve user meal plan queries with status filtering",
				Priority:  "high",
				CreateSQL: "CREATE INDEX IF NOT EXISTS idx_meal_plans_user_week_status ON meal_plans(user_id, week_start DESC, status);",
			})
		}
	}

	return recommendations
}

func (q *queryPerformanceService) generateOptimizationTips(query string) []string {
	var tips []string
	query = strings.ToLower(query)

	if strings.Contains(query, "select *") {
		tips = append(tips, "Avoid SELECT *; specify only needed columns")
	}
	
	if strings.Contains(query, "order by") && !strings.Contains(query, "limit") {
		tips = append(tips, "Consider adding LIMIT to ORDER BY queries")
	}
	
	if strings.Contains(query, "where") && strings.Count(query, "and") > 3 {
		tips = append(tips, "Complex WHERE clauses may benefit from composite indices")
	}
	
	if strings.Contains(query, "jsonb") && strings.Contains(query, "->") {
		tips = append(tips, "Consider using GIN indices for JSONB queries")
	}
	
	if strings.Contains(query, "ilike") {
		tips = append(tips, "Consider using full-text search (to_tsvector) instead of ILIKE for better performance")
	}

	return tips
}

// GetPerformanceReport provides overall database performance insights
func (q *queryPerformanceService) GetPerformanceReport(ctx context.Context, since time.Duration) (*PerformanceReport, error) {
	cutoff := time.Now().Add(-since)
	
	var totalQueries, slowQueries int
	var totalDuration time.Duration
	topSlowQueries := make([]QueryPerformanceMetrics, 0)
	
	for _, metric := range q.metrics {
		if metric.Timestamp.After(cutoff) {
			totalQueries++
			totalDuration += metric.Duration
			
			if metric.IsSlowQuery {
				slowQueries++
				topSlowQueries = append(topSlowQueries, metric)
			}
		}
	}
	
	// Sort and limit top slow queries
	if len(topSlowQueries) > 10 {
		// Simple sorting by duration (in real implementation, use sort.Slice)
		topSlowQueries = topSlowQueries[:10]
	}
	
	var averageQueryTime time.Duration
	if totalQueries > 0 {
		averageQueryTime = totalDuration / time.Duration(totalQueries)
	}
	
	// Get index usage stats
	indexUsage, err := q.getIndexUsageStats(ctx)
	if err != nil {
		log.Printf("Failed to get index usage stats: %v", err)
		indexUsage = []IndexUsageStats{}
	}
	
	report := &PerformanceReport{
		TotalQueries:       totalQueries,
		SlowQueries:        slowQueries,
		AverageQueryTime:   averageQueryTime,
		SlowQueryThreshold: q.slowQueryThreshold,
		TopSlowQueries:     topSlowQueries,
		IndexUsage:         indexUsage,
		Recommendations:    q.generatePerformanceRecommendations(slowQueries, totalQueries),
	}
	
	return report, nil
}

func (q *queryPerformanceService) getIndexUsageStats(ctx context.Context) ([]IndexUsageStats, error) {
	var stats []IndexUsageStats
	
	// Query PostgreSQL statistics for index usage
	query := `
		SELECT 
			schemaname as schema_name,
			tablename as table_name,
			indexname as index_name,
			idx_scan as scan_count,
			idx_tup_read as tuple_reads,
			idx_tup_fetch as tuple_fetches
		FROM pg_stat_user_indexes 
		WHERE schemaname = 'public'
		ORDER BY idx_scan DESC;
	`
	
	rows, err := q.db.Raw(query).Rows()
	if err != nil {
		return stats, err
	}
	defer rows.Close()
	
	for rows.Next() {
		var schemaName, tableName, indexName string
		var scanCount, tupleReads, tupleFetches int64
		
		if err := rows.Scan(&schemaName, &tableName, &indexName, &scanCount, &tupleReads, &tupleFetches); err != nil {
			continue
		}
		
		stats = append(stats, IndexUsageStats{
			TableName:    tableName,
			IndexName:    indexName,
			ScanCount:    scanCount,
			TupleReads:   tupleReads,
			TupleFetches: tupleFetches,
			IsUnused:     scanCount == 0,
		})
	}
	
	return stats, nil
}

func (q *queryPerformanceService) generatePerformanceRecommendations(slowQueries, totalQueries int) []string {
	var recommendations []string
	
	slowQueryRatio := float64(slowQueries) / float64(totalQueries)
	
	if slowQueryRatio > 0.1 {
		recommendations = append(recommendations, "High ratio of slow queries detected - review indexing strategy")
	}
	
	if slowQueries > 0 {
		recommendations = append(recommendations, "Consider implementing query result caching for frequently accessed data")
		recommendations = append(recommendations, "Review and optimize database indices based on query patterns")
	}
	
	recommendations = append(recommendations, "Consider connection pooling optimization for better throughput")
	recommendations = append(recommendations, "Implement query result pagination for large datasets")
	
	return recommendations
}

// OptimizeCommonQueries creates indices for commonly used query patterns
func (q *queryPerformanceService) OptimizeCommonQueries(ctx context.Context) error {
	optimizations := []string{
		// Recipe search optimizations
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_search_composite ON recipes(deleted_at, total_time, complexity, average_rating) WHERE deleted_at IS NULL;",
		
		// Meal plan optimizations
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_active_user ON meal_plans(user_id, status, week_start DESC) WHERE status = 'active';",
		
		// Recipe rating optimizations
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_ratings_recipe_user ON recipe_ratings(recipe_id, user_id, overall_rating);",
		
		// Full-text search optimization
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_fulltext_gin ON recipes USING GIN(to_tsvector('english', title || ' ' || COALESCE(description, ''))) WHERE deleted_at IS NULL;",
		
		// JSONB optimizations
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_ingredients_gin ON recipes USING GIN(ingredients) WHERE deleted_at IS NULL;",
		"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_meals_gin ON meal_plans USING GIN(meals);",
	}
	
	for _, sql := range optimizations {
		if err := q.db.Exec(sql).Error; err != nil {
			log.Printf("Failed to create optimization index: %s, error: %v", sql, err)
			// Continue with other optimizations even if one fails
		} else {
			log.Printf("Successfully created optimization index: %s", sql)
		}
	}
	
	return nil
}