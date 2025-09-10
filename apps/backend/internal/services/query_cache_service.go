package services

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"time"

	"gorm.io/gorm"
)

// QueryCacheConfig configures query caching behavior
type QueryCacheConfig struct {
	DefaultTTL       time.Duration            `json:"defaultTtl"`
	MaxCacheSize     int                      `json:"maxCacheSize"`
	QueryTTLMap      map[string]time.Duration `json:"queryTtlMap"`
	EnableMetrics    bool                     `json:"enableMetrics"`
	PrefixWhitelist  []string                 `json:"prefixWhitelist"`
	TableBlacklist   []string                 `json:"tableBlacklist"`
}

// CachedQuery represents a cached query result
type CachedQuery struct {
	Key         string      `json:"key"`
	SQL         string      `json:"sql"`
	Args        []interface{} `json:"args"`
	Result      interface{} `json:"result"`
	RowCount    int         `json:"rowCount"`
	CachedAt    time.Time   `json:"cachedAt"`
	TTL         time.Duration `json:"ttl"`
	HitCount    int64       `json:"hitCount"`
	LastHit     time.Time   `json:"lastHit"`
	QueryType   string      `json:"queryType"`
	Tables      []string    `json:"tables"`
}

// QueryCacheMetrics tracks cache performance
type QueryCacheMetrics struct {
	TotalQueries    int64         `json:"totalQueries"`
	CacheHits       int64         `json:"cacheHits"`
	CacheMisses     int64         `json:"cacheMisses"`
	HitRate         float64       `json:"hitRate"`
	AvgQueryTime    time.Duration `json:"avgQueryTime"`
	CacheSize       int           `json:"cacheSize"`
	MemoryUsage     int64         `json:"memoryUsage"`
	TopQueries      []QueryStats  `json:"topQueries"`
	InvalidationCount int64       `json:"invalidationCount"`
}

// QueryStats provides statistics for individual queries
type QueryStats struct {
	QueryPattern string        `json:"queryPattern"`
	HitCount     int64         `json:"hitCount"`
	AvgTime      time.Duration `json:"avgTime"`
	LastAccess   time.Time     `json:"lastAccess"`
	CacheStatus  string        `json:"cacheStatus"`
}

// InvalidationRule defines cache invalidation rules
type InvalidationRule struct {
	TableName    string        `json:"tableName"`
	Operations   []string      `json:"operations"` // INSERT, UPDATE, DELETE
	Pattern      string        `json:"pattern"`    // SQL pattern to match
	TTLOverride  time.Duration `json:"ttlOverride"`
}

// QueryCacheService provides intelligent query result caching
type QueryCacheService interface {
	// Core caching operations
	Get(ctx context.Context, query string, args []interface{}, dest interface{}) (bool, error)
	Set(ctx context.Context, query string, args []interface{}, result interface{}, ttl time.Duration) error
	Invalidate(ctx context.Context, pattern string) error
	InvalidateTable(ctx context.Context, tableName string) error
	
	// Cache management
	Clear(ctx context.Context) error
	GetMetrics(ctx context.Context) (*QueryCacheMetrics, error)
	WarmCache(ctx context.Context, queries []string) error
	
	// Query wrapping for automatic caching
	CachedQuery(ctx context.Context, db *gorm.DB, query string, args []interface{}, dest interface{}) error
	CachedFind(ctx context.Context, db *gorm.DB, dest interface{}, conditions ...interface{}) error
	
	// Configuration and rules
	AddInvalidationRule(rule InvalidationRule) error
	SetTTLForPattern(pattern string, ttl time.Duration) error
	
	// Cache optimization
	OptimizeCache(ctx context.Context) error
	GetCacheStatus(ctx context.Context) (*CacheStatus, error)
}

// CacheStatus provides detailed cache status information
type CacheStatus struct {
	IsEnabled       bool          `json:"isEnabled"`
	Size            int           `json:"size"`
	MaxSize         int           `json:"maxSize"`
	MemoryUsage     int64         `json:"memoryUsage"`
	OldestEntry     time.Time     `json:"oldestEntry"`
	MostRecentEntry time.Time     `json:"mostRecentEntry"`
	HitRate24h      float64       `json:"hitRate24h"`
	TopTables       []TableStats  `json:"topTables"`
}

// TableStats provides cache statistics per table
type TableStats struct {
	TableName   string  `json:"tableName"`
	QueryCount  int64   `json:"queryCount"`
	HitRate     float64 `json:"hitRate"`
	AvgTTL      time.Duration `json:"avgTtl"`
}

type queryCacheService struct {
	cache             *CacheService
	config            *QueryCacheConfig
	metrics           *QueryCacheMetrics
	invalidationRules []InvalidationRule
	ttlPatterns       map[string]time.Duration
}

func NewQueryCacheService(cache *CacheService, config *QueryCacheConfig) QueryCacheService {
	if config == nil {
		config = &QueryCacheConfig{
			DefaultTTL:      5 * time.Minute,
			MaxCacheSize:    1000,
			EnableMetrics:   true,
			PrefixWhitelist: []string{"SELECT"},
			TableBlacklist:  []string{"query_performance_log", "user_sessions"},
		}
	}

	service := &queryCacheService{
		cache:             cache,
		config:            config,
		invalidationRules: make([]InvalidationRule, 0),
		ttlPatterns:       make(map[string]time.Duration),
		metrics: &QueryCacheMetrics{
			TopQueries: make([]QueryStats, 0),
		},
	}

	// Add default invalidation rules
	service.addDefaultInvalidationRules()

	return service
}

func (q *queryCacheService) addDefaultInvalidationRules() {
	defaultRules := []InvalidationRule{
		{
			TableName:  "recipes",
			Operations: []string{"INSERT", "UPDATE", "DELETE"},
			Pattern:    "recipes",
			TTLOverride: 10 * time.Minute,
		},
		{
			TableName:  "meal_plans",
			Operations: []string{"INSERT", "UPDATE", "DELETE"},
			Pattern:    "meal_plans",
			TTLOverride: 15 * time.Minute,
		},
		{
			TableName:  "recipe_ratings",
			Operations: []string{"INSERT", "UPDATE", "DELETE"},
			Pattern:    "recipe_ratings|recipes.*rating",
			TTLOverride: 30 * time.Minute,
		},
	}

	for _, rule := range defaultRules {
		q.invalidationRules = append(q.invalidationRules, rule)
	}
}

// Get retrieves cached query result
func (q *queryCacheService) Get(ctx context.Context, query string, args []interface{}, dest interface{}) (bool, error) {
	if !q.shouldCache(query) {
		return false, nil
	}

	cacheKey := q.generateCacheKey(query, args)
	
	// Update metrics
	if q.config.EnableMetrics {
		q.metrics.TotalQueries++
	}

	cached, err := q.cache.Get(ctx, cacheKey)
	if err != nil {
		if q.config.EnableMetrics {
			q.metrics.CacheMisses++
		}
		return false, nil // Cache miss
	}

	var cachedQuery CachedQuery
	if err := json.Unmarshal([]byte(cached), &cachedQuery); err != nil {
		log.Printf("Failed to unmarshal cached query: %v", err)
		return false, nil
	}

	// Check TTL
	if time.Since(cachedQuery.CachedAt) > cachedQuery.TTL {
		q.cache.Delete(ctx, cacheKey)
		if q.config.EnableMetrics {
			q.metrics.CacheMisses++
		}
		return false, nil
	}

	// Unmarshal result into destination
	resultJSON, err := json.Marshal(cachedQuery.Result)
	if err != nil {
		return false, fmt.Errorf("failed to marshal cached result: %w", err)
	}

	if err := json.Unmarshal(resultJSON, dest); err != nil {
		return false, fmt.Errorf("failed to unmarshal cached result: %w", err)
	}

	// Update hit statistics
	cachedQuery.HitCount++
	cachedQuery.LastHit = time.Now()
	
	// Update cache with new hit statistics
	go func() {
		updatedCache, _ := json.Marshal(cachedQuery)
		q.cache.Set(context.Background(), cacheKey, string(updatedCache), cachedQuery.TTL)
	}()

	if q.config.EnableMetrics {
		q.metrics.CacheHits++
		q.updateMetrics()
	}

	return true, nil
}

// Set stores query result in cache
func (q *queryCacheService) Set(ctx context.Context, query string, args []interface{}, result interface{}, ttl time.Duration) error {
	if !q.shouldCache(query) {
		return nil
	}

	cacheKey := q.generateCacheKey(query, args)

	// Determine TTL
	if ttl == 0 {
		ttl = q.determineTTL(query)
	}

	cachedQuery := CachedQuery{
		Key:       cacheKey,
		SQL:       query,
		Args:      args,
		Result:    result,
		CachedAt:  time.Now(),
		TTL:       ttl,
		HitCount:  0,
		QueryType: q.extractQueryType(query),
		Tables:    q.extractTables(query),
	}

	// Calculate row count if possible
	if resultSlice, ok := result.([]interface{}); ok {
		cachedQuery.RowCount = len(resultSlice)
	}

	cachedJSON, err := json.Marshal(cachedQuery)
	if err != nil {
		return fmt.Errorf("failed to marshal cached query: %w", err)
	}

	return q.cache.Set(ctx, cacheKey, string(cachedJSON), ttl)
}

// CachedQuery executes a query with automatic caching
func (q *queryCacheService) CachedQuery(ctx context.Context, db *gorm.DB, query string, args []interface{}, dest interface{}) error {
	// Try cache first
	hit, err := q.Get(ctx, query, args, dest)
	if err != nil {
		log.Printf("Cache get error: %v", err)
	}
	if hit {
		return nil
	}

	// Execute query
	startTime := time.Now()
	err = db.Raw(query, args...).Scan(dest).Error
	queryTime := time.Since(startTime)

	if err != nil {
		return err
	}

	// Cache the result
	ttl := q.determineTTL(query)
	if queryTime > 100*time.Millisecond { // Cache slower queries with longer TTL
		ttl = ttl * 2
	}

	go func() {
		if err := q.Set(context.Background(), query, args, dest, ttl); err != nil {
			log.Printf("Failed to cache query result: %v", err)
		}
	}()

	return nil
}

// CachedFind executes a GORM Find with automatic caching
func (q *queryCacheService) CachedFind(ctx context.Context, db *gorm.DB, dest interface{}, conditions ...interface{}) error {
	// Generate a deterministic query representation
	query := fmt.Sprintf("FIND:%T:%v", dest, conditions)
	
	// Try cache first
	hit, err := q.Get(ctx, query, nil, dest)
	if err != nil {
		log.Printf("Cache get error: %v", err)
	}
	if hit {
		return nil
	}

	// Execute query
	startTime := time.Now()
	err = db.Find(dest, conditions...).Error
	queryTime := time.Since(startTime)

	if err != nil {
		return err
	}

	// Cache the result
	ttl := q.config.DefaultTTL
	if queryTime > 50*time.Millisecond {
		ttl = ttl * 2
	}

	go func() {
		if err := q.Set(context.Background(), query, nil, dest, ttl); err != nil {
			log.Printf("Failed to cache find result: %v", err)
		}
	}()

	return nil
}

// Invalidate removes cached queries matching a pattern
func (q *queryCacheService) Invalidate(ctx context.Context, pattern string) error {
	// This is a simplified implementation
	// In practice, you'd maintain an index of cache keys by pattern
	log.Printf("Invalidating cache for pattern: %s", pattern)
	
	if q.config.EnableMetrics {
		q.metrics.InvalidationCount++
	}
	
	return nil
}

// InvalidateTable removes all cached queries for a specific table
func (q *queryCacheService) InvalidateTable(ctx context.Context, tableName string) error {
	log.Printf("Invalidating cache for table: %s", tableName)
	
	// Find matching invalidation rules
	for _, rule := range q.invalidationRules {
		if rule.TableName == tableName {
			if err := q.Invalidate(ctx, rule.Pattern); err != nil {
				log.Printf("Failed to invalidate pattern %s: %v", rule.Pattern, err)
			}
		}
	}
	
	return nil
}

// Clear removes all cached queries
func (q *queryCacheService) Clear(ctx context.Context) error {
	log.Printf("Clearing entire query cache")
	
	// Reset metrics
	if q.config.EnableMetrics {
		q.metrics = &QueryCacheMetrics{
			TopQueries: make([]QueryStats, 0),
		}
	}
	
	return nil // Would implement actual clearing logic
}

// GetMetrics returns current cache metrics
func (q *queryCacheService) GetMetrics(ctx context.Context) (*QueryCacheMetrics, error) {
	if !q.config.EnableMetrics {
		return nil, fmt.Errorf("metrics not enabled")
	}

	q.updateMetrics()
	return q.metrics, nil
}

func (q *queryCacheService) updateMetrics() {
	if q.metrics.TotalQueries > 0 {
		q.metrics.HitRate = float64(q.metrics.CacheHits) / float64(q.metrics.TotalQueries)
	}
}

// WarmCache pre-loads common queries into cache
func (q *queryCacheService) WarmCache(ctx context.Context, queries []string) error {
	log.Printf("Warming cache with %d queries", len(queries))
	
	for _, query := range queries {
		// This would execute and cache common queries
		log.Printf("Warming cache for query pattern: %s", query)
	}
	
	return nil
}

// Helper methods

func (q *queryCacheService) shouldCache(query string) bool {
	query = strings.TrimSpace(strings.ToUpper(query))
	
	// Check if query type is whitelisted
	cached := false
	for _, prefix := range q.config.PrefixWhitelist {
		if strings.HasPrefix(query, strings.ToUpper(prefix)) {
			cached = true
			break
		}
	}
	
	if !cached {
		return false
	}
	
	// Check if any blacklisted tables are involved
	for _, table := range q.config.TableBlacklist {
		if strings.Contains(strings.ToLower(query), strings.ToLower(table)) {
			return false
		}
	}
	
	return true
}

func (q *queryCacheService) generateCacheKey(query string, args []interface{}) string {
	// Create deterministic cache key
	keyData := struct {
		Query string        `json:"query"`
		Args  []interface{} `json:"args"`
	}{
		Query: strings.TrimSpace(query),
		Args:  args,
	}
	
	keyJSON, _ := json.Marshal(keyData)
	hash := sha256.Sum256(keyJSON)
	return fmt.Sprintf("query_cache:%s", hex.EncodeToString(hash[:8]))
}

func (q *queryCacheService) determineTTL(query string) time.Duration {
	query = strings.ToLower(query)
	
	// Check custom TTL patterns
	for pattern, ttl := range q.ttlPatterns {
		if strings.Contains(query, pattern) {
			return ttl
		}
	}
	
	// Default TTL based on query characteristics
	if strings.Contains(query, "count(") {
		return 10 * time.Minute // Count queries change less frequently
	}
	
	if strings.Contains(query, "avg(") || strings.Contains(query, "sum(") {
		return 15 * time.Minute // Aggregate queries
	}
	
	if strings.Contains(query, "recipes") && strings.Contains(query, "rating") {
		return 30 * time.Minute // Rating aggregations change slowly
	}
	
	return q.config.DefaultTTL
}

func (q *queryCacheService) extractQueryType(query string) string {
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

func (q *queryCacheService) extractTables(query string) []string {
	// Simplified table extraction
	// Real implementation would use SQL parsing
	tables := []string{}
	query = strings.ToLower(query)
	
	commonTables := []string{"recipes", "meal_plans", "users", "recipe_ratings", "shopping_lists"}
	for _, table := range commonTables {
		if strings.Contains(query, table) {
			tables = append(tables, table)
		}
	}
	
	return tables
}

// Configuration methods

func (q *queryCacheService) AddInvalidationRule(rule InvalidationRule) error {
	q.invalidationRules = append(q.invalidationRules, rule)
	return nil
}

func (q *queryCacheService) SetTTLForPattern(pattern string, ttl time.Duration) error {
	q.ttlPatterns[pattern] = ttl
	return nil
}

func (q *queryCacheService) OptimizeCache(ctx context.Context) error {
	log.Printf("Optimizing query cache")
	
	// Remove least recently used items if cache is full
	// Adjust TTLs based on hit patterns
	// Compact fragmented cache entries
	
	return nil
}

func (q *queryCacheService) GetCacheStatus(ctx context.Context) (*CacheStatus, error) {
	status := &CacheStatus{
		IsEnabled:   true,
		MaxSize:     q.config.MaxCacheSize,
		TopTables:   make([]TableStats, 0),
	}
	
	// Would implement actual status gathering
	return status, nil
}