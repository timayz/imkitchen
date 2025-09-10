package services

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"strconv"
	"time"

	"gorm.io/gorm"
)

// CursorPaginationParams provides cursor-based pagination parameters
type CursorPaginationParams struct {
	First  *int    `json:"first,omitempty"`  // Forward pagination limit
	After  *string `json:"after,omitempty"`  // Cursor for forward pagination
	Last   *int    `json:"last,omitempty"`   // Backward pagination limit
	Before *string `json:"before,omitempty"` // Cursor for backward pagination
}

// OffsetPaginationParams provides offset-based pagination parameters
type OffsetPaginationParams struct {
	Page  int `json:"page"`
	Limit int `json:"limit"`
}

// PaginationInfo provides pagination metadata
type PaginationInfo struct {
	HasNextPage     bool    `json:"hasNextPage"`
	HasPreviousPage bool    `json:"hasPreviousPage"`
	StartCursor     *string `json:"startCursor,omitempty"`
	EndCursor       *string `json:"endCursor,omitempty"`
	TotalCount      *int64  `json:"totalCount,omitempty"`
	CurrentPage     *int    `json:"currentPage,omitempty"`
	TotalPages      *int    `json:"totalPages,omitempty"`
	PageSize        int     `json:"pageSize"`
}

// Edge represents a paginated item with cursor
type Edge struct {
	Node   interface{} `json:"node"`
	Cursor string      `json:"cursor"`
}

// PaginatedResult provides paginated results with metadata
type PaginatedResult struct {
	Edges    []Edge          `json:"edges"`
	PageInfo *PaginationInfo `json:"pageInfo"`
}

// CursorInfo contains information for cursor generation
type CursorInfo struct {
	ID        string    `json:"id"`
	SortField string    `json:"sortField"`
	SortValue interface{} `json:"sortValue"`
	Timestamp time.Time `json:"timestamp"`
}

// PaginationService provides advanced pagination capabilities
type PaginationService interface {
	// Cursor-based pagination (for real-time data and large datasets)
	ApplyCursorPagination(query *gorm.DB, params CursorPaginationParams, sortField string) (*gorm.DB, error)
	GenerateCursor(item interface{}, sortField string) (string, error)
	ParseCursor(cursor string) (*CursorInfo, error)
	
	// Offset-based pagination (for traditional pagination)
	ApplyOffsetPagination(query *gorm.DB, params OffsetPaginationParams) *gorm.DB
	CalculatePaginationInfo(totalCount int64, params OffsetPaginationParams) *PaginationInfo
	
	// Hybrid pagination results
	CreatePaginatedResult(items []interface{}, params interface{}, totalCount *int64, sortField string) (*PaginatedResult, error)
	
	// Performance optimizations
	OptimizePaginationQuery(query *gorm.DB, estimatedCount int64) *gorm.DB
	GetEstimatedCount(query *gorm.DB) (int64, error)
}

type paginationService struct {
	maxPageSize     int
	defaultPageSize int
}

func NewPaginationService() PaginationService {
	return &paginationService{
		maxPageSize:     100,
		defaultPageSize: 20,
	}
}

// ApplyCursorPagination applies cursor-based pagination to a GORM query
func (p *paginationService) ApplyCursorPagination(query *gorm.DB, params CursorPaginationParams, sortField string) (*gorm.DB, error) {
	// Validate parameters
	if params.First != nil && params.Last != nil {
		return nil, fmt.Errorf("cannot specify both 'first' and 'last' parameters")
	}
	
	if params.After != nil && params.Before != nil {
		return nil, fmt.Errorf("cannot specify both 'after' and 'before' parameters")
	}
	
	// Forward pagination
	if params.First != nil {
		limit := *params.First
		if limit > p.maxPageSize {
			limit = p.maxPageSize
		}
		
		query = query.Limit(limit + 1) // +1 to check if there's a next page
		
		if params.After != nil {
			cursor, err := p.ParseCursor(*params.After)
			if err != nil {
				return nil, fmt.Errorf("invalid cursor: %w", err)
			}
			
			// Apply cursor condition based on sort field
			query = p.applyCursorCondition(query, cursor, sortField, false)
		}
		
		// Apply default sorting
		query = query.Order(fmt.Sprintf("%s ASC, id ASC", sortField))
	}
	
	// Backward pagination
	if params.Last != nil {
		limit := *params.Last
		if limit > p.maxPageSize {
			limit = p.maxPageSize
		}
		
		query = query.Limit(limit + 1) // +1 to check if there's a previous page
		
		if params.Before != nil {
			cursor, err := p.ParseCursor(*params.Before)
			if err != nil {
				return nil, fmt.Errorf("invalid cursor: %w", err)
			}
			
			// Apply cursor condition based on sort field
			query = p.applyCursorCondition(query, cursor, sortField, true)
		}
		
		// Apply reverse sorting for backward pagination
		query = query.Order(fmt.Sprintf("%s DESC, id DESC", sortField))
	}
	
	// Default forward pagination if no direction specified
	if params.First == nil && params.Last == nil {
		query = query.Limit(p.defaultPageSize + 1)
		query = query.Order(fmt.Sprintf("%s ASC, id ASC", sortField))
	}
	
	return query, nil
}

func (p *paginationService) applyCursorCondition(query *gorm.DB, cursor *CursorInfo, sortField string, reverse bool) *gorm.DB {
	operator := ">"
	if reverse {
		operator = "<"
	}
	
	// Handle different sort field types
	switch v := cursor.SortValue.(type) {
	case string:
		return query.Where(fmt.Sprintf("(%s %s ? OR (%s = ? AND id %s ?))", 
			sortField, operator, sortField, operator), v, v, cursor.ID)
	case float64:
		return query.Where(fmt.Sprintf("(%s %s ? OR (%s = ? AND id %s ?))", 
			sortField, operator, sortField, operator), v, v, cursor.ID)
	case time.Time:
		return query.Where(fmt.Sprintf("(%s %s ? OR (%s = ? AND id %s ?))", 
			sortField, operator, sortField, operator), v, v, cursor.ID)
	default:
		// Fallback to ID-only pagination
		return query.Where(fmt.Sprintf("id %s ?", operator), cursor.ID)
	}
}

// GenerateCursor creates a cursor string for a given item
func (p *paginationService) GenerateCursor(item interface{}, sortField string) (string, error) {
	// Extract ID and sort field value from the item
	// This is a simplified implementation - real implementation would use reflection
	// or struct tags to extract the appropriate fields
	
	cursorInfo := &CursorInfo{
		Timestamp: time.Now(),
		SortField: sortField,
	}
	
	// In a real implementation, you would extract the actual values from the item
	// For now, we'll create a placeholder cursor
	cursorInfo.ID = "placeholder-id"
	cursorInfo.SortValue = "placeholder-value"
	
	// Encode cursor as base64 JSON
	cursorJSON, err := json.Marshal(cursorInfo)
	if err != nil {
		return "", fmt.Errorf("failed to marshal cursor: %w", err)
	}
	
	return base64.StdEncoding.EncodeToString(cursorJSON), nil
}

// ParseCursor decodes a cursor string
func (p *paginationService) ParseCursor(cursor string) (*CursorInfo, error) {
	// Decode base64
	cursorJSON, err := base64.StdEncoding.DecodeString(cursor)
	if err != nil {
		return nil, fmt.Errorf("invalid cursor encoding: %w", err)
	}
	
	// Unmarshal JSON
	var cursorInfo CursorInfo
	if err := json.Unmarshal(cursorJSON, &cursorInfo); err != nil {
		return nil, fmt.Errorf("invalid cursor format: %w", err)
	}
	
	return &cursorInfo, nil
}

// ApplyOffsetPagination applies traditional offset-based pagination
func (p *paginationService) ApplyOffsetPagination(query *gorm.DB, params OffsetPaginationParams) *gorm.DB {
	// Validate and normalize parameters
	if params.Page < 1 {
		params.Page = 1
	}
	if params.Limit < 1 {
		params.Limit = p.defaultPageSize
	}
	if params.Limit > p.maxPageSize {
		params.Limit = p.maxPageSize
	}
	
	// Calculate offset
	offset := (params.Page - 1) * params.Limit
	
	return query.Offset(offset).Limit(params.Limit)
}

// CalculatePaginationInfo calculates pagination metadata for offset-based pagination
func (p *paginationService) CalculatePaginationInfo(totalCount int64, params OffsetPaginationParams) *PaginationInfo {
	// Normalize parameters
	if params.Page < 1 {
		params.Page = 1
	}
	if params.Limit < 1 {
		params.Limit = p.defaultPageSize
	}
	
	totalPages := int(totalCount) / params.Limit
	if int(totalCount)%params.Limit > 0 {
		totalPages++
	}
	
	hasNextPage := params.Page < totalPages
	hasPreviousPage := params.Page > 1
	
	return &PaginationInfo{
		HasNextPage:     hasNextPage,
		HasPreviousPage: hasPreviousPage,
		TotalCount:      &totalCount,
		CurrentPage:     &params.Page,
		TotalPages:      &totalPages,
		PageSize:        params.Limit,
	}
}

// CreatePaginatedResult creates a paginated result with appropriate metadata
func (p *paginationService) CreatePaginatedResult(items []interface{}, params interface{}, totalCount *int64, sortField string) (*PaginatedResult, error) {
	edges := make([]Edge, 0, len(items))
	
	// Handle different pagination types
	switch p := params.(type) {
	case CursorPaginationParams:
		return p.createCursorPaginatedResult(items, p, sortField)
	case OffsetPaginationParams:
		return p.createOffsetPaginatedResult(items, p, totalCount)
	default:
		return nil, fmt.Errorf("unsupported pagination params type")
	}
}

func (p *paginationService) createCursorPaginatedResult(items []interface{}, params CursorPaginationParams, sortField string) (*PaginatedResult, error) {
	if len(items) == 0 {
		return &PaginatedResult{
			Edges: []Edge{},
			PageInfo: &PaginationInfo{
				HasNextPage:     false,
				HasPreviousPage: false,
				PageSize:        p.getRequestedLimit(params),
			},
		}, nil
	}
	
	requestedLimit := p.getRequestedLimit(params)
	hasMore := len(items) > requestedLimit
	
	// Remove extra item used for has-more detection
	if hasMore {
		items = items[:requestedLimit]
	}
	
	// Create edges with cursors
	edges := make([]Edge, len(items))
	for i, item := range items {
		cursor, err := p.GenerateCursor(item, sortField)
		if err != nil {
			return nil, fmt.Errorf("failed to generate cursor: %w", err)
		}
		
		edges[i] = Edge{
			Node:   item,
			Cursor: cursor,
		}
	}
	
	// Determine pagination state
	var hasNextPage, hasPreviousPage bool
	var startCursor, endCursor *string
	
	if len(edges) > 0 {
		firstCursor := edges[0].Cursor
		lastCursor := edges[len(edges)-1].Cursor
		startCursor = &firstCursor
		endCursor = &lastCursor
	}
	
	// Forward pagination
	if params.First != nil {
		hasNextPage = hasMore
		hasPreviousPage = params.After != nil
	}
	
	// Backward pagination
	if params.Last != nil {
		hasNextPage = params.Before != nil
		hasPreviousPage = hasMore
		
		// Reverse the order for backward pagination
		for i, j := 0, len(edges)-1; i < j; i, j = i+1, j-1 {
			edges[i], edges[j] = edges[j], edges[i]
		}
	}
	
	return &PaginatedResult{
		Edges: edges,
		PageInfo: &PaginationInfo{
			HasNextPage:     hasNextPage,
			HasPreviousPage: hasPreviousPage,
			StartCursor:     startCursor,
			EndCursor:       endCursor,
			PageSize:        requestedLimit,
		},
	}, nil
}

func (p *paginationService) createOffsetPaginatedResult(items []interface{}, params OffsetPaginationParams, totalCount *int64) (*PaginatedResult, error) {
	edges := make([]Edge, len(items))
	for i, item := range items {
		// For offset pagination, we can use simple sequential cursors
		cursor := base64.StdEncoding.EncodeToString([]byte(strconv.Itoa((params.Page-1)*params.Limit + i)))
		edges[i] = Edge{
			Node:   item,
			Cursor: cursor,
		}
	}
	
	var pageInfo *PaginationInfo
	if totalCount != nil {
		pageInfo = p.CalculatePaginationInfo(*totalCount, params)
	} else {
		pageInfo = &PaginationInfo{
			PageSize: params.Limit,
		}
	}
	
	return &PaginatedResult{
		Edges:    edges,
		PageInfo: pageInfo,
	}, nil
}

func (p *paginationService) getRequestedLimit(params CursorPaginationParams) int {
	if params.First != nil {
		return minInt(*params.First, p.maxPageSize)
	}
	if params.Last != nil {
		return minInt(*params.Last, p.maxPageSize)
	}
	return p.defaultPageSize
}

// OptimizePaginationQuery optimizes queries for better pagination performance
func (p *paginationService) OptimizePaginationQuery(query *gorm.DB, estimatedCount int64) *gorm.DB {
	// For large datasets, disable count queries to improve performance
	if estimatedCount > 10000 {
		// Use query hints for large datasets
		query = query.Session(&gorm.Session{
			PrepareStmt: true, // Use prepared statements for better performance
		})
	}
	
	// Add query hints for PostgreSQL
	if estimatedCount > 100000 {
		// For very large datasets, consider using LIMIT optimization
		query = query.Set("gorm:query_hint", "/*+ USE_INDEX(recipes, idx_recipes_search_composite) */")
	}
	
	return query
}

// GetEstimatedCount provides fast count estimation for large tables
func (p *paginationService) GetEstimatedCount(query *gorm.DB) (int64, error) {
	var count int64
	
	// For PostgreSQL, we can use table statistics for fast estimation
	// This is a simplified version - full implementation would query pg_stat_user_tables
	err := query.Count(&count).Error
	
	return count, err
}

// Utility function
func minInt(a, b int) int {
	if a < b {
		return a
	}
	return b
}