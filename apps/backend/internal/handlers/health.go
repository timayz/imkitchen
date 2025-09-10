package handlers

import (
	"context"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

// HealthResponse represents the structure of health check response
type HealthResponse struct {
	Status    string            `json:"status"`
	Timestamp string            `json:"timestamp"`
	Version   string            `json:"version"`
	Services  map[string]string `json:"services,omitempty"`
}

// HealthCheck returns a basic health check endpoint
func HealthCheck(deps interface{}) gin.HandlerFunc {
	return func(c *gin.Context) {
		response := HealthResponse{
			Status:    "healthy",
			Timestamp: time.Now().UTC().Format(time.RFC3339),
			Version:   "1.0.0",
		}

		c.JSON(http.StatusOK, response)
	}
}

// ReadinessCheck returns a readiness check endpoint that verifies dependencies
func ReadinessCheck(deps interface{}) gin.HandlerFunc {
	return func(c *gin.Context) {
		services := make(map[string]string)
		status := "ready"
		httpStatus := http.StatusOK
		
		// Check Redis connectivity if dependencies are available
		if d, ok := deps.(*Dependencies); ok && d.CacheService != nil {
			ctx, cancel := context.WithTimeout(c.Request.Context(), 2*time.Second)
			defer cancel()
			
			if err := d.CacheService.Ping(ctx); err != nil {
				services["redis"] = "unhealthy"
				status = "degraded"
				httpStatus = http.StatusServiceUnavailable
			} else {
				services["redis"] = "healthy"
			}
		} else {
			services["redis"] = "not_configured"
		}
		
		// TODO: Add database connectivity check
		services["database"] = "not_configured"
		
		response := HealthResponse{
			Status:    status,
			Timestamp: time.Now().UTC().Format(time.RFC3339),
			Version:   "1.0.0",
			Services:  services,
		}

		c.JSON(httpStatus, response)
	}
}

// CacheServiceInterface defines the interface for cache operations
type CacheServiceInterface interface {
	Ping(ctx context.Context) error
}

// Dependencies type for handler access - using interface instead of concrete type
type Dependencies struct {
	CacheService CacheServiceInterface
}