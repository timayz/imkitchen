package middleware

import (
	"fmt"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/services"
)

// RateLimit returns a gin.HandlerFunc that implements rate limiting
func RateLimit(cacheService interface{}, requests int, window time.Duration) gin.HandlerFunc {
	return func(c *gin.Context) {
		// Skip rate limiting if cache service is not available
		cache, ok := cacheService.(*services.CacheService)
		if !ok || cache == nil {
			c.Next()
			return
		}

		// Create rate limit key based on client IP and endpoint
		clientIP := c.ClientIP()
		endpoint := c.Request.URL.Path
		key := fmt.Sprintf("rate_limit:%s:%s", clientIP, endpoint)

		// Check rate limit
		allowed, remaining, err := cache.CheckRateLimit(c.Request.Context(), key, requests, window)
		if err != nil {
			// Log error but don't block request if Redis is down
			c.Next()
			return
		}

		// Set rate limit headers
		c.Header("X-RateLimit-Limit", fmt.Sprintf("%d", requests))
		c.Header("X-RateLimit-Remaining", fmt.Sprintf("%d", remaining))
		c.Header("X-RateLimit-Window", window.String())

		// Block request if rate limit exceeded
		if !allowed {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error":     "Rate limit exceeded",
				"status":    "error",
				"limit":     requests,
				"window":    window.String(),
				"retry_after": window.Seconds(),
			})
			c.Abort()
			return
		}

		c.Next()
	}
}

// AuthRateLimit returns a rate limiter specifically for authentication endpoints
func AuthRateLimit(cacheService interface{}) gin.HandlerFunc {
	// 5 attempts per minute for auth endpoints (strict for security)
	return RateLimit(cacheService, 5, time.Minute)
}

// GeneralRateLimit returns a rate limiter for general API endpoints
func GeneralRateLimit(cacheService interface{}) gin.HandlerFunc {
	// 100 requests per minute for general endpoints
	return RateLimit(cacheService, 100, time.Minute)
}