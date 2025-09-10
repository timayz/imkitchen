package types

import (
	"time"

	"github.com/google/uuid"
)

// CursorPaginationParams represents parameters for cursor-based pagination
type CursorPaginationParams struct {
	After     string `json:"after,omitempty"`
	Before    string `json:"before,omitempty"`
	Limit     int    `json:"limit"`
	SortBy    string `json:"sort_by"`
	SortOrder string `json:"sort_order"` // "asc" or "desc"
}

// OffsetPaginationParams represents parameters for offset-based pagination
type OffsetPaginationParams struct {
	Page     int    `json:"page"`
	PageSize int    `json:"page_size"`
	SortBy   string `json:"sort_by"`
	SortOrder string `json:"sort_order"`
}

// PaginatedResult represents a paginated result set
type PaginatedResult struct {
	Data       interface{}    `json:"data"`
	Pagination *PaginationInfo `json:"pagination"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// PaginationInfo contains pagination metadata
type PaginationInfo struct {
	Type        string `json:"type"` // "cursor" or "offset"
	HasNext     bool   `json:"has_next"`
	HasPrevious bool   `json:"has_previous"`
	TotalCount  *int64 `json:"total_count,omitempty"`
	
	// Cursor pagination fields
	NextCursor     *string `json:"next_cursor,omitempty"`
	PreviousCursor *string `json:"previous_cursor,omitempty"`
	
	// Offset pagination fields
	CurrentPage *int `json:"current_page,omitempty"`
	TotalPages  *int `json:"total_pages,omitempty"`
	PageSize    *int `json:"page_size,omitempty"`
}

// CursorInfo represents cursor information for a specific record
type CursorInfo struct {
	ID        uuid.UUID   `json:"id"`
	SortValue interface{} `json:"sort_value"`
	SortField string      `json:"sort_field"`
	Timestamp time.Time   `json:"timestamp"`
}

// PaginationService interface for pagination operations
type PaginationService interface {
	CreateCursorFromRecord(record interface{}, sortBy string) (string, error)
	ParseCursor(cursor string) (*CursorInfo, error)
	BuildCursorCondition(cursor *CursorInfo, sortOrder string) (string, []interface{}, error)
	CalculateOffset(page, pageSize int) int
	CalculateTotalPages(totalCount int64, pageSize int) int
}