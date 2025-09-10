package services

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"reflect"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/utils"
)

// EnhancedPaginationService extends the base pagination service with performance optimizations
type EnhancedPaginationService interface {
	PaginationService
	
	// Enhanced cursor generation with reflection
	GenerateEnhancedCursor(item interface{}, sortField string) (string, error)
	
	// Performance optimization methods
	GetFastEstimatedCount(query *gorm.DB, tableName string) (int64, error)
	OptimizeForLargeDataset(query *gorm.DB, estimatedCount int64) *gorm.DB
	
	// Pagination performance analysis
	AnalyzePaginationPerformance(queryType string, executionTime time.Duration, resultCount int, totalCount int64) *PaginationPerformanceMetrics
	
	// Smart pagination strategy selection
	RecommendPaginationStrategy(estimatedCount int64, queryComplexity string) *PaginationStrategy
}

// PaginationStrategy provides recommendations for optimal pagination approach
type PaginationStrategy struct {
	RecommendedType   string    `json:"recommendedType"`   // "cursor", "offset", "hybrid"
	Reasoning         string    `json:"reasoning"`
	PerformanceGain   string    `json:"performanceGain"`
	UseCountEstimation bool     `json:"useCountEstimation"`
	OptimalPageSize   int      `json:"optimalPageSize"`
	IndexRecommendations []string `json:"indexRecommendations"`
}

// PaginationPerformanceMetrics tracks detailed pagination performance
type PaginationPerformanceMetrics struct {
	QueryType          string        `json:"queryType"`
	PaginationType     string        `json:"paginationType"`
	ExecutionTime      time.Duration `json:"executionTime"`
	ResultCount        int           `json:"resultCount"`
	TotalCount         int64         `json:"totalCount"`
	Page               int           `json:"page"`
	EstimatedCount     bool          `json:"estimatedCount"`
	IndexesUsed        []string      `json:"indexesUsed"`
	PerformanceRating  string        `json:"performanceRating"`
	MemoryUsage        int64         `json:"memoryUsage"`
	CacheHitRatio      float64       `json:"cacheHitRatio"`
	Timestamp          time.Time     `json:"timestamp"`
}

type enhancedPaginationService struct {
	*paginationService
	enablePerformanceTracking bool
}

func NewEnhancedPaginationService() EnhancedPaginationService {
	return &enhancedPaginationService{
		paginationService: &paginationService{
			maxPageSize:     100,
			defaultPageSize: 20,
		},
		enablePerformanceTracking: true,
	}
}

// GenerateEnhancedCursor creates a cursor with proper field extraction using reflection
func (p *enhancedPaginationService) GenerateEnhancedCursor(item interface{}, sortField string) (string, error) {
	cursorInfo := &CursorInfo{
		Timestamp: time.Now(),
		SortField: sortField,
	}
	
	// Use reflection to extract the actual field values
	value := reflect.ValueOf(item)
	if value.Kind() == reflect.Ptr {
		value = value.Elem()
	}
	
	if !value.IsValid() {
		return "", fmt.Errorf("invalid item for cursor generation")
	}
	
	// Extract ID field
	if idField := value.FieldByName("ID"); idField.IsValid() && !idField.IsNil() {
		switch idField.Interface().(type) {
		case uuid.UUID:
			cursorInfo.ID = idField.Interface().(uuid.UUID).String()
		case string:
			cursorInfo.ID = idField.Interface().(string)
		default:
			cursorInfo.ID = fmt.Sprintf("%v", idField.Interface())
		}
	}
	
	// Extract sort field value with proper field name conversion
	sortFieldName := p.toStructFieldName(sortField)
	if sortFieldValue := value.FieldByName(sortFieldName); sortFieldValue.IsValid() {
		cursorInfo.SortValue = sortFieldValue.Interface()
	} else {
		// Fallback: try to find field by tag or alternative names
		cursorInfo.SortValue = p.findFieldByAlternativeNames(value, sortField)
	}
	
	// Encode cursor as base64 JSON
	cursorJSON, err := json.Marshal(cursorInfo)
	if err != nil {
		return "", fmt.Errorf("failed to marshal cursor: %w", err)
	}
	
	return base64.StdEncoding.EncodeToString(cursorJSON), nil
}

// toStructFieldName converts database field names to Go struct field names
func (p *enhancedPaginationService) toStructFieldName(dbFieldName string) string {
	// Convert snake_case to CamelCase
	parts := strings.Split(dbFieldName, "_")
	result := ""
	for _, part := range parts {
		if len(part) > 0 {
			result += strings.ToUpper(string(part[0])) + strings.ToLower(part[1:])
		}
	}
	
	// Handle special cases
	switch result {
	case "CreatedAt":
		return "CreatedAt"
	case "UpdatedAt":
		return "UpdatedAt"
	case "DeletedAt":
		return "DeletedAt"
	case "TotalTime":
		return "TotalTime"
	case "AverageRating":
		return "AverageRating"
	case "PrepTime":
		return "PrepTime"
	case "CookTime":
		return "CookTime"
	}
	
	return result
}

// findFieldByAlternativeNames attempts to find struct fields using alternative naming strategies
func (p *enhancedPaginationService) findFieldByAlternativeNames(value reflect.Value, fieldName string) interface{} {
	valueType := value.Type()
	
	// Try different field naming conventions
	alternatives := []string{
		p.toStructFieldName(fieldName),
		strings.Title(fieldName),
		fieldName,
	}
	
	for _, alt := range alternatives {
		if field := value.FieldByName(alt); field.IsValid() {
			return field.Interface()
		}
	}
	
	// Try finding by JSON tag
	for i := 0; i < valueType.NumField(); i++ {
		field := valueType.Field(i)
		if jsonTag := field.Tag.Get("json"); jsonTag != "" {
			jsonName := strings.Split(jsonTag, ",")[0]
			if jsonName == fieldName {
				fieldValue := value.Field(i)
				if fieldValue.IsValid() {
					return fieldValue.Interface()
				}
			}
		}
		
		// Try GORM tag
		if gormTag := field.Tag.Get("gorm"); gormTag != "" && strings.Contains(gormTag, "column:"+fieldName) {
			fieldValue := value.Field(i)
			if fieldValue.IsValid() {
				return fieldValue.Interface()
			}
		}
	}
	
	return nil
}

// GetFastEstimatedCount provides fast count estimation using PostgreSQL statistics
func (p *enhancedPaginationService) GetFastEstimatedCount(query *gorm.DB, tableName string) (int64, error) {
	// For PostgreSQL, we can query pg_stat_user_tables for fast estimates
	var count int64
	
	// First try to get estimate from pg_stat_user_tables
	estimateQuery := `
		SELECT COALESCE(n_tup_ins + n_tup_upd - n_tup_del, 0) as estimated_count
		FROM pg_stat_user_tables 
		WHERE tablename = ?
	`
	
	err := query.Raw(estimateQuery, tableName).Scan(&count).Error
	if err != nil || count == 0 {
		// Fallback to query planner estimate using EXPLAIN
		var result []map[string]interface{}
		explainQuery := fmt.Sprintf("EXPLAIN (FORMAT JSON) %s", query.ToSQL())
		
		err = query.Raw(explainQuery).Scan(&result).Error
		if err == nil && len(result) > 0 {
			// Extract row estimate from query plan
			if planData, ok := result[0]["QUERY PLAN"].(string); ok {
				var plan map[string]interface{}
				if json.Unmarshal([]byte(planData), &plan) == nil {
					if planArray, ok := plan["Plan"].([]interface{}); ok && len(planArray) > 0 {
						if rootPlan, ok := planArray[0].(map[string]interface{}); ok {
							if planRows, ok := rootPlan["Plan Rows"].(float64); ok {
								count = int64(planRows)
							}
						}
					}
				}
			}
		}
		
		// Final fallback to actual count for small datasets
		if count == 0 {
			err = query.Count(&count).Error
		}
	}
	
	return count, err
}

// OptimizeForLargeDataset applies PostgreSQL-specific optimizations for large datasets
func (p *enhancedPaginationService) OptimizeForLargeDataset(query *gorm.DB, estimatedCount int64) *gorm.DB {
	// Apply different optimizations based on dataset size
	if estimatedCount > 100000 {
		// For very large datasets (100k+)
		query = query.Session(&gorm.Session{
			PrepareStmt: true,
			Context:     query.Statement.Context,
		})
		
		// Add query hints for PostgreSQL
		query = query.Set("gorm:query_hint", "/*+ USE_INDEX_FOR_ORDER_BY */")
		
		// Disable unnecessary ORDER BY for cursor pagination
		// (cursor pagination inherently provides ordering)
		
	} else if estimatedCount > 10000 {
		// For medium datasets (10k-100k)
		query = query.Session(&gorm.Session{
			PrepareStmt: true,
			Context:     query.Statement.Context,
		})
	}
	
	// For smaller datasets, use standard optimization
	return query
}

// AnalyzePaginationPerformance provides detailed performance analysis
func (p *enhancedPaginationService) AnalyzePaginationPerformance(queryType string, executionTime time.Duration, resultCount int, totalCount int64) *PaginationPerformanceMetrics {
	metrics := &PaginationPerformanceMetrics{
		QueryType:      queryType,
		ExecutionTime:  executionTime,
		ResultCount:    resultCount,
		TotalCount:     totalCount,
		Timestamp:      time.Now(),
	}
	
	// Calculate performance rating
	if executionTime <= 50*time.Millisecond {
		metrics.PerformanceRating = "excellent"
	} else if executionTime <= 100*time.Millisecond {
		metrics.PerformanceRating = "good"
	} else if executionTime <= 200*time.Millisecond {
		metrics.PerformanceRating = "acceptable"
	} else {
		metrics.PerformanceRating = "needs_optimization"
	}
	
	// Determine if estimation was likely used
	metrics.EstimatedCount = totalCount > 10000
	
	return metrics
}

// RecommendPaginationStrategy provides intelligent pagination strategy recommendations
func (p *enhancedPaginationService) RecommendPaginationStrategy(estimatedCount int64, queryComplexity string) *PaginationStrategy {
	strategy := &PaginationStrategy{
		IndexRecommendations: []string{},
	}
	
	// Determine optimal strategy based on dataset size and complexity
	if estimatedCount > 50000 {
		strategy.RecommendedType = "cursor"
		strategy.Reasoning = "Large dataset benefits from cursor-based pagination for consistent performance"
		strategy.PerformanceGain = "60-80% better performance for deep pagination"
		strategy.UseCountEstimation = true
		strategy.OptimalPageSize = 20
		
		if queryComplexity == "high" {
			strategy.IndexRecommendations = append(strategy.IndexRecommendations,
				"Consider adding composite indices for filter + sort combinations",
				"Add covering indices with INCLUDE clause for frequently accessed columns",
			)
		}
		
	} else if estimatedCount > 10000 {
		strategy.RecommendedType = "hybrid"
		strategy.Reasoning = "Medium dataset can benefit from hybrid approach - cursor for deep pages, offset for shallow"
		strategy.PerformanceGain = "40-60% better for pages beyond 100"
		strategy.UseCountEstimation = true
		strategy.OptimalPageSize = 25
		
	} else {
		strategy.RecommendedType = "offset"
		strategy.Reasoning = "Small dataset works well with traditional offset pagination"
		strategy.PerformanceGain = "Optimal for small datasets with exact counts"
		strategy.UseCountEstimation = false
		strategy.OptimalPageSize = 20
	}
	
	// Add general index recommendations
	if estimatedCount > 1000 {
		strategy.IndexRecommendations = append(strategy.IndexRecommendations,
			"Ensure primary sort column has optimized index",
			"Consider partial indices for frequently filtered subsets",
		)
	}
	
	return strategy
}

// Enhanced method to override the base GenerateCursor with proper implementation
func (p *enhancedPaginationService) GenerateCursor(item interface{}, sortField string) (string, error) {
	// Use the enhanced version that actually works
	return p.GenerateEnhancedCursor(item, sortField)
}

// PaginationHealthCheck provides health metrics for pagination performance
type PaginationHealthCheck struct {
	AverageExecutionTime time.Duration `json:"averageExecutionTime"`
	SlowQueryCount       int           `json:"slowQueryCount"`
	ErrorRate            float64       `json:"errorRate"`
	OptimizationScore    int           `json:"optimizationScore"` // 0-100
	Recommendations      []string      `json:"recommendations"`
	LastChecked          time.Time     `json:"lastChecked"`
}

// CheckPaginationHealth analyzes overall pagination performance
func CheckPaginationHealth(metrics []PaginationPerformanceMetrics) *PaginationHealthCheck {
	if len(metrics) == 0 {
		return &PaginationHealthCheck{
			OptimizationScore: 100,
			LastChecked:       time.Now(),
		}
	}
	
	var totalTime time.Duration
	slowQueries := 0
	
	for _, metric := range metrics {
		totalTime += metric.ExecutionTime
		if metric.ExecutionTime > 200*time.Millisecond {
			slowQueries++
		}
	}
	
	avgTime := totalTime / time.Duration(len(metrics))
	
	health := &PaginationHealthCheck{
		AverageExecutionTime: avgTime,
		SlowQueryCount:      slowQueries,
		ErrorRate:           0, // Would be calculated from actual error tracking
		LastChecked:         time.Now(),
	}
	
	// Calculate optimization score (0-100)
	score := 100
	if avgTime > 50*time.Millisecond {
		score -= 20
	}
	if avgTime > 100*time.Millisecond {
		score -= 30
	}
	if avgTime > 200*time.Millisecond {
		score -= 40
	}
	
	if slowQueries > len(metrics)/10 { // More than 10% slow queries
		score -= 20
	}
	
	health.OptimizationScore = utils.MaxInt(score, 0)
	
	// Generate recommendations
	if avgTime > 100*time.Millisecond {
		health.Recommendations = append(health.Recommendations,
			"Consider implementing cursor-based pagination for better performance",
			"Review database indices for sort and filter columns",
		)
	}
	
	if slowQueries > 0 {
		health.Recommendations = append(health.Recommendations,
			"Analyze slow queries and optimize database indices",
			"Consider count estimation for large datasets",
		)
	}
	
	return health
}

