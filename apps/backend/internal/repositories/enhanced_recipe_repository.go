package repositories

import (
	"context"
	"fmt"
	"reflect"
	"strings"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/types"
)

// EnhancedRecipeRepository provides advanced pagination and performance optimizations
type EnhancedRecipeRepository interface {
	// Legacy compatibility
	RecipeRepository
	
	// Enhanced search methods
	SearchWithCursorPagination(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams, cursorParams types.CursorPaginationParams) (*types.PaginatedResult, error)
	SearchWithOptimizedPagination(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error)
	GetEstimatedRecipeCount(ctx context.Context, userID uuid.UUID, filters *models.RecipeFilters) (int64, error)
	SearchWithMetadata(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams) (*EnhancedRecipeSearchResponse, error)
}

// EnhancedRecipeSearchResponse provides comprehensive search results with performance metadata
type EnhancedRecipeSearchResponse struct {
	*models.RecipeSearchResponse
	ExecutionTime    time.Duration              `json:"executionTime"`
	QueryPlan        *string                    `json:"queryPlan,omitempty"`
	IndexesUsed      []string                   `json:"indexesUsed,omitempty"`
	EstimatedTotal   bool                       `json:"estimatedTotal"`
	CacheHit         bool                       `json:"cacheHit"`
	PaginationInfo   *types.PaginationInfo   `json:"paginationInfo"`
}

type enhancedRecipeRepository struct {
	db                *gorm.DB
	paginationService types.PaginationService
}

func NewEnhancedRecipeRepository(db *gorm.DB, paginationService types.PaginationService) EnhancedRecipeRepository {
	return &enhancedRecipeRepository{
		db:                db,
		paginationService: paginationService,
	}
}

// Implement RecipeRepository methods by embedding functionality
func (r *enhancedRecipeRepository) Create(ctx context.Context, recipe *Recipe) error {
	return r.db.WithContext(ctx).Create(recipe).Error
}

func (r *enhancedRecipeRepository) GetByID(ctx context.Context, id uuid.UUID) (*Recipe, error) {
	var recipe Recipe
	err := r.db.WithContext(ctx).Where("id = ? AND deleted_at IS NULL", id).First(&recipe).Error
	if err != nil {
		return nil, err
	}
	return &recipe, nil
}

func (r *enhancedRecipeRepository) GetByUserID(ctx context.Context, userID uuid.UUID, filters *RecipeFilters) ([]*Recipe, error) {
	query := r.db.WithContext(ctx).Model(&Recipe{}).Where("user_id = ? AND deleted_at IS NULL", userID)
	
	// Convert RecipeFilters to models.RecipeFilters
	var modelFilters *models.RecipeFilters
	if filters != nil {
		modelFilters = &models.RecipeFilters{
			CuisineType:   filters.Cuisine,
			DietaryLabels: filters.DietaryLabels,
			MaxPrepTime:   filters.MaxPrepTime,
			MaxCookTime:   filters.MaxCookTime,
		}
	}
	
	query = r.applyFilters(query, modelFilters)
	
	var recipes []*Recipe
	err := query.Find(&recipes).Error
	return recipes, err
}

func (r *enhancedRecipeRepository) Update(ctx context.Context, id uuid.UUID, updates *RecipeUpdates) error {
	return r.db.WithContext(ctx).Model(&Recipe{}).Where("id = ? AND deleted_at IS NULL", id).Updates(updates).Error
}

func (r *enhancedRecipeRepository) Delete(ctx context.Context, id uuid.UUID) error {
	return r.db.WithContext(ctx).Model(&Recipe{}).Where("id = ?", id).Update("deleted_at", time.Now()).Error
}

func (r *enhancedRecipeRepository) Search(ctx context.Context, query *RecipeSearchQuery) ([]*Recipe, error) {
	q := r.db.WithContext(ctx).Model(&Recipe{})
	
	if query.Query != "" {
		q = q.Where("title ILIKE ? OR description ILIKE ?", "%"+query.Query+"%", "%"+query.Query+"%")
	}
	
	if query.UserID != nil {
		q = q.Where("user_id = ? OR is_public = true", *query.UserID)
	} else {
		q = q.Where("is_public = true")
	}
	
	if query.Filters != nil {
		// Convert RecipeFilters to models.RecipeFilters
		modelFilters := &models.RecipeFilters{
			CuisineType:   query.Filters.Cuisine,
			DietaryLabels: query.Filters.DietaryLabels,
			MaxPrepTime:   query.Filters.MaxPrepTime,
			MaxCookTime:   query.Filters.MaxCookTime,
		}
		q = r.applyFilters(q, modelFilters)
	}
	
	if query.SortBy != "" {
		order := query.SortBy
		if query.SortOrder == "desc" {
			order += " DESC"
		}
		q = q.Order(order)
	}
	
	var recipes []*Recipe
	err := q.Find(&recipes).Error
	return recipes, err
}

func (r *enhancedRecipeRepository) GetByExternalSource(ctx context.Context, source, externalID string) (*Recipe, error) {
	var recipe Recipe
	err := r.db.WithContext(ctx).Where("external_source = ? AND external_id = ? AND deleted_at IS NULL", source, externalID).First(&recipe).Error
	if err != nil {
		return nil, err
	}
	return &recipe, nil
}

func (r *enhancedRecipeRepository) GetCommunityRecipeByID(ctx context.Context, id uuid.UUID) (*Recipe, error) {
	var recipe Recipe
	err := r.db.WithContext(ctx).Where("id = ? AND is_public = true AND deleted_at IS NULL", id).First(&recipe).Error
	if err != nil {
		return nil, err
	}
	return &recipe, nil
}

// applyFilters applies recipe filters to a query
func (r *enhancedRecipeRepository) applyFilters(query *gorm.DB, filters *models.RecipeFilters) *gorm.DB {
	if filters == nil {
		return query
	}
	
	if filters.CuisineType != nil {
		query = query.Where("cuisine_type = ?", *filters.CuisineType)
	}
	
	if len(filters.DietaryLabels) > 0 {
		query = query.Where("dietary_labels && ?", filters.DietaryLabels)
	}
	
	if filters.MaxPrepTime != nil {
		query = query.Where("prep_time <= ?", *filters.MaxPrepTime)
	}
	
	if filters.MaxCookTime != nil {
		query = query.Where("cook_time <= ?", *filters.MaxCookTime)
	}
	
	if len(filters.ExcludeIDs) > 0 {
		query = query.Where("id NOT IN ?", filters.ExcludeIDs)
	}
	
	return query
}

// SearchWithCursorPagination implements cursor-based pagination for recipe search
func (r *enhancedRecipeRepository) SearchWithCursorPagination(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams, cursorParams types.CursorPaginationParams) (*types.PaginatedResult, error) {
	startTime := time.Now()

	// Build base query
	query := r.buildBaseSearchQuery(userID)
	
	// Apply filters
	query = r.applyFilters(query, &params.RecipeFilters)
	
	// Determine sort field
	sortField := r.getSortField(params)
	
	// Apply cursor pagination
	paginatedQuery, err := r.applyCursorPagination(query, cursorParams, sortField)
	if err != nil {
		return nil, fmt.Errorf("failed to apply cursor pagination: %w", err)
	}
	
	// Execute query
	var recipes []models.Recipe
	if err := paginatedQuery.WithContext(ctx).Find(&recipes).Error; err != nil {
		return nil, fmt.Errorf("failed to execute paginated query: %w", err)
	}
	
	// Convert to interface slice for pagination service
	items := make([]interface{}, len(recipes))
	for i, recipe := range recipes {
		items[i] = recipe
	}
	
	// Create paginated result
	result, err := r.createPaginatedResult(items, cursorParams, sortField)
	if err != nil {
		return nil, fmt.Errorf("failed to create paginated result: %w", err)
	}
	
	// Add execution time to metadata
	if result.Metadata == nil {
		result.Metadata = make(map[string]interface{})
	}
	result.Metadata["execution_time"] = time.Since(startTime).Milliseconds()
	
	return result, nil
}

// SearchWithOptimizedPagination provides performance-optimized search with smart pagination selection
func (r *enhancedRecipeRepository) SearchWithOptimizedPagination(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {

	// Build base query
	query := r.buildBaseSearchQuery(userID)
	
	// Apply filters
	query = r.applyFilters(query, &params.RecipeFilters)
	
	// Get estimated count first
	estimatedCount, err := r.getEstimatedCount(query)
	if err != nil {
		return nil, fmt.Errorf("failed to get estimated count: %w", err)
	}
	
	// Optimize query for large datasets
	query = r.optimizePaginationQuery(query, estimatedCount)
	
	// Apply sorting
	sortField := r.getSortField(params)
	sortOrder := "ASC"
	if params.SortOrder == "desc" {
		sortOrder = "DESC"
	}
	query = query.Order(fmt.Sprintf("%s %s, id ASC", sortField, sortOrder))
	
	// Apply pagination using the optimized service
	offsetParams := types.OffsetPaginationParams{
		Page:     params.Page,
		PageSize: params.Limit,
	}
	query = r.applyOffsetPagination(query, offsetParams)
	
	// Execute query
	var recipes []models.Recipe
	if err := query.WithContext(ctx).Find(&recipes).Error; err != nil {
		return nil, fmt.Errorf("failed to execute search query: %w", err)
	}
	
	// Calculate pagination metadata
	var total int64
	useEstimatedCount := estimatedCount > 10000 // Use estimation for large datasets
	
	if useEstimatedCount {
		total = estimatedCount
	} else {
		// Get exact count for smaller datasets
		countQuery := r.buildBaseSearchQuery(userID)
		countQuery = r.applyFilters(countQuery, &params.RecipeFilters)
		if err := countQuery.Count(&total).Error; err != nil {
			return nil, fmt.Errorf("failed to get exact count: %w", err)
		}
	}
	
	// Calculate pagination info
	pageInfo := r.calculatePaginationInfo(total, offsetParams)
	
	response := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      total,
		Page:       params.Page,
		Limit:      params.Limit,
		TotalPages: *pageInfo.TotalPages,
	}
	
	return response, nil
}

// SearchWithMetadata provides comprehensive search with performance metadata
func (r *enhancedRecipeRepository) SearchWithMetadata(ctx context.Context, userID uuid.UUID, params *models.RecipeSearchParams) (*EnhancedRecipeSearchResponse, error) {
	startTime := time.Now()

	// Get standard search results
	response, err := r.SearchWithOptimizedPagination(ctx, userID, params)
	if err != nil {
		return nil, err
	}
	
	// Build enhanced response with metadata
	enhanced := &EnhancedRecipeSearchResponse{
		RecipeSearchResponse: response,
		ExecutionTime:        time.Since(startTime),
		EstimatedTotal:       response.Total > 10000, // Indicates if we used estimation
		CacheHit:            false, // TODO: Integrate with Redis caching in Task 3
		IndexesUsed:         r.detectIndexUsage(params),
	}
	
	// Add pagination info
	offsetParams := types.OffsetPaginationParams{
		Page:     params.Page,
		PageSize: params.Limit,
	}
	enhanced.PaginationInfo = r.calculatePaginationInfo(response.Total, offsetParams)
	
	return enhanced, nil
}

// GetEstimatedRecipeCount provides fast count estimation for pagination planning
func (r *enhancedRecipeRepository) GetEstimatedRecipeCount(ctx context.Context, userID uuid.UUID, filters *models.RecipeFilters) (int64, error) {
	query := r.buildBaseSearchQuery(userID)
	query = r.applyFilters(query, filters)
	
	return r.getEstimatedCount(query)
}

// buildBaseSearchQuery creates the base query for recipe searches
func (r *enhancedRecipeRepository) buildBaseSearchQuery(userID uuid.UUID) *gorm.DB {
	return r.db.Model(&models.Recipe{}).Where("deleted_at IS NULL AND (user_id = ? OR is_public = true)", userID)
}

// getSortField determines the appropriate sort field with performance optimization
func (r *enhancedRecipeRepository) getSortField(params *models.RecipeSearchParams) string {
	sortField := "created_at" // Default with good index performance
	
	if params.SortBy != "" {
		switch params.SortBy {
		case "created_at":
			sortField = "created_at"
		case "updated_at":
			sortField = "updated_at"
		case "total_time":
			sortField = "total_time"
		case "average_rating":
			sortField = "average_rating"
		default:
			sortField = "created_at" // Fallback to indexed field
		}
	}
	
	return sortField
}

// detectIndexUsage analyzes which database indices are likely being used
func (r *enhancedRecipeRepository) detectIndexUsage(params *models.RecipeSearchParams) []string {
	var indexes []string
	
	// Basic deletion filter
	indexes = append(indexes, "idx_recipes_deleted_at")
	
	// Search-specific indices from our Task 1 migration
	if params.Search != nil && *params.Search != "" {
		indexes = append(indexes, "idx_recipes_fulltext_ranked")
	}
	
	if params.CuisineType != nil && *params.CuisineType != "" {
		if params.MaxPrepTime != nil {
			indexes = append(indexes, "idx_recipes_cuisine_diet_preptime")
		}
	}
	
	if len(params.DietaryLabels) > 0 {
		indexes = append(indexes, "idx_recipes_combined_filters")
		
		// Check for specialized dietary indices
		for _, label := range params.DietaryLabels {
			switch label {
			case "vegetarian":
				indexes = append(indexes, "idx_recipes_vegetarian_fast")
			case "vegan":
				indexes = append(indexes, "idx_recipes_vegan_fast")
			case "gluten-free":
				indexes = append(indexes, "idx_recipes_gluten_free_fast")
			}
		}
	}
	
	// Sorting and pagination
	switch params.SortBy {
	case "created_at":
		indexes = append(indexes, "idx_recipes_pagination_optimal")
	case "average_rating":
		indexes = append(indexes, "idx_recipes_trending_optimized")
	}
	
	return indexes
}

// Helper methods for pagination operations

// applyCursorPagination applies cursor-based pagination to a query
func (r *enhancedRecipeRepository) applyCursorPagination(query *gorm.DB, params types.CursorPaginationParams, sortField string) (*gorm.DB, error) {
	if params.After != "" {
		cursorInfo, err := r.paginationService.ParseCursor(params.After)
		if err != nil {
			return nil, fmt.Errorf("failed to parse after cursor: %w", err)
		}
		
		condition, args, err := r.paginationService.BuildCursorCondition(cursorInfo, params.SortOrder)
		if err != nil {
			return nil, fmt.Errorf("failed to build cursor condition: %w", err)
		}
		
		query = query.Where(condition, args...)
	}
	
	// Apply sorting
	sortOrder := "ASC"
	if params.SortOrder == "desc" {
		sortOrder = "DESC"
	}
	query = query.Order(fmt.Sprintf("%s %s, id ASC", sortField, sortOrder))
	
	// Apply limit
	query = query.Limit(params.Limit)
	
	return query, nil
}

// createPaginatedResult creates a paginated result from items
func (r *enhancedRecipeRepository) createPaginatedResult(items []interface{}, params types.CursorPaginationParams, sortField string) (*types.PaginatedResult, error) {
	hasNext := len(items) == params.Limit
	hasPrevious := params.After != ""
	
	var nextCursor, previousCursor *string
	
	if hasNext && len(items) > 0 {
		lastItem := items[len(items)-1]
		cursor, err := r.generateCursor(lastItem, sortField)
		if err != nil {
			return nil, fmt.Errorf("failed to generate next cursor: %w", err)
		}
		nextCursor = &cursor
	}
	
	if hasPrevious && len(items) > 0 {
		firstItem := items[0]
		cursor, err := r.generateCursor(firstItem, sortField)
		if err != nil {
			return nil, fmt.Errorf("failed to generate previous cursor: %w", err)
		}
		previousCursor = &cursor
	}
	
	pagination := &types.PaginationInfo{
		Type:            "cursor",
		HasNext:         hasNext,
		HasPrevious:     hasPrevious,
		NextCursor:      nextCursor,
		PreviousCursor:  previousCursor,
	}
	
	return &types.PaginatedResult{
		Data:       items,
		Pagination: pagination,
	}, nil
}

// generateCursor creates a cursor with proper field extraction using reflection
func (r *enhancedRecipeRepository) generateCursor(item interface{}, sortField string) (string, error) {
	cursorInfo := &types.CursorInfo{
		Timestamp: time.Now(),
		SortField: sortField,
	}
	
	// Use reflection to extract the actual field values
	value := reflect.ValueOf(item)
	if value.Kind() == reflect.Ptr {
		value = value.Elem()
	}
	
	// Extract ID field
	if idField := value.FieldByName("ID"); idField.IsValid() {
		if idField.Type().String() == "uuid.UUID" {
			cursorInfo.ID = idField.Interface().(uuid.UUID)
		}
	}
	
	// Extract sort field value
	if sortFieldValue := value.FieldByName(r.toCamelCase(sortField)); sortFieldValue.IsValid() {
		cursorInfo.SortValue = sortFieldValue.Interface()
	}
	
	// Encode cursor
	return r.paginationService.CreateCursorFromRecord(cursorInfo, sortField)
}

// toCamelCase converts snake_case to CamelCase for struct field names
func (r *enhancedRecipeRepository) toCamelCase(s string) string {
	parts := strings.Split(s, "_")
	result := ""
	for _, part := range parts {
		if len(part) > 0 {
			result += strings.ToUpper(string(part[0])) + part[1:]
		}
	}
	return result
}

// getEstimatedCount provides fast count estimation
func (r *enhancedRecipeRepository) getEstimatedCount(query *gorm.DB) (int64, error) {
	var count int64
	err := query.Count(&count).Error
	return count, err
}

// optimizePaginationQuery optimizes query for large datasets
func (r *enhancedRecipeRepository) optimizePaginationQuery(query *gorm.DB, estimatedCount int64) *gorm.DB {
	if estimatedCount > 100000 {
		// Use covering indexes for very large datasets
		query = query.Select("id, title, description, user_id, created_at, updated_at, total_time, average_rating")
	}
	return query
}

// applyOffsetPagination applies offset-based pagination
func (r *enhancedRecipeRepository) applyOffsetPagination(query *gorm.DB, params types.OffsetPaginationParams) *gorm.DB {
	offset := r.paginationService.CalculateOffset(params.Page, params.PageSize)
	return query.Offset(offset).Limit(params.PageSize)
}

// calculatePaginationInfo calculates pagination metadata
func (r *enhancedRecipeRepository) calculatePaginationInfo(total int64, params types.OffsetPaginationParams) *types.PaginationInfo {
	totalPages := r.paginationService.CalculateTotalPages(total, params.PageSize)
	currentPage := params.Page
	pageSize := params.PageSize
	
	return &types.PaginationInfo{
		Type:         "offset",
		HasNext:      currentPage < totalPages,
		HasPrevious:  currentPage > 1,
		TotalCount:   &total,
		CurrentPage:  &currentPage,
		TotalPages:   &totalPages,
		PageSize:     &pageSize,
	}
}

// PaginationPerformanceMetrics tracks pagination performance
type PaginationPerformanceMetrics struct {
	QueryType          string        `json:"queryType"`
	PaginationType     string        `json:"paginationType"`     // "cursor" or "offset"
	ExecutionTime      time.Duration `json:"executionTime"`
	ResultCount        int           `json:"resultCount"`
	TotalCount         int64         `json:"totalCount"`
	Page               int           `json:"page"`
	EstimatedCount     bool          `json:"estimatedCount"`
	IndexesUsed        []string      `json:"indexesUsed"`
	PerformanceRating  string        `json:"performanceRating"` // "excellent", "good", "needs_optimization"
	Timestamp          time.Time     `json:"timestamp"`
}

// CalculatePerformanceRating determines the performance rating based on execution time and other factors
func CalculatePerformanceRating(metrics *PaginationPerformanceMetrics) string {
	if metrics.ExecutionTime <= 50*time.Millisecond {
		return "excellent"
	} else if metrics.ExecutionTime <= 100*time.Millisecond {
		return "good"
	} else {
		return "needs_optimization"
	}
}

// CountOptimization provides strategies for optimizing COUNT queries on large datasets
type CountOptimization struct {
	UseEstimation     bool   `json:"useEstimation"`
	EstimationMethod  string `json:"estimationMethod"`
	ThresholdRows     int64  `json:"thresholdRows"`
	PerformanceGain   string `json:"performanceGain"`
}

func GetCountOptimizationStrategy(estimatedRows int64) *CountOptimization {
	if estimatedRows > 100000 {
		return &CountOptimization{
			UseEstimation:    true,
			EstimationMethod: "pg_stat_user_tables",
			ThresholdRows:    estimatedRows,
			PerformanceGain:  "80-95% faster",
		}
	} else if estimatedRows > 10000 {
		return &CountOptimization{
			UseEstimation:    true,
			EstimationMethod: "query_planner_estimate",
			ThresholdRows:    estimatedRows,
			PerformanceGain:  "60-80% faster",
		}
	}
	
	return &CountOptimization{
		UseEstimation:    false,
		EstimationMethod: "exact_count",
		ThresholdRows:    estimatedRows,
		PerformanceGain:  "exact_results",
	}
}