package middleware

import (
	"encoding/json"
	"fmt"
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

// RateLimitConfig holds rate limiting configuration
type RateLimitConfig struct {
	MaxRequests int           `json:"maxRequests"`
	Window      time.Duration `json:"window"`
	Enabled     bool          `json:"enabled"`
}

// UserRateLimit tracks rate limiting for a specific user
type UserRateLimit struct {
	Count      int       `json:"count"`
	WindowStart time.Time `json:"windowStart"`
	mu         sync.RWMutex
}

// RatingRateLimiter manages rate limiting for rating submissions
type RatingRateLimiter struct {
	users  map[string]*UserRateLimit
	config *RateLimitConfig
	mu     sync.RWMutex
}

// NewRatingRateLimiter creates a new rating rate limiter
func NewRatingRateLimiter() *RatingRateLimiter {
	limiter := &RatingRateLimiter{
		users: make(map[string]*UserRateLimit),
		config: &RateLimitConfig{
			MaxRequests: 5,                // 5 ratings per minute
			Window:      time.Minute,      // 1 minute window
			Enabled:     true,
		},
	}
	
	// Start cleanup goroutine
	go limiter.cleanupExpired()
	
	return limiter
}

// RatingRateLimit middleware for rating submission endpoints
func (r *RatingRateLimiter) RatingRateLimit() gin.HandlerFunc {
	return func(c *gin.Context) {
		if !r.config.Enabled {
			c.Next()
			return
		}

		// Get user ID from context (set by auth middleware)
		userIDInterface, exists := c.Get("userID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "authentication_required",
				"message": "Authentication required for rating submission",
			})
			c.Abort()
			return
		}

		userID, ok := userIDInterface.(uuid.UUID)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error": "internal_error",
				"message": "Invalid user context",
			})
			c.Abort()
			return
		}

		// Check rate limit
		if !r.allowRequest(userID.String()) {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error": "rate_limit_exceeded",
				"message": fmt.Sprintf("Too many rating submissions. Maximum %d per %v allowed.", 
					r.config.MaxRequests, r.config.Window),
				"retry_after": int(r.config.Window.Seconds()),
			})
			c.Abort()
			return
		}

		c.Next()
	}
}

// allowRequest checks if a request is allowed for the user
func (r *RatingRateLimiter) allowRequest(userID string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()

	now := time.Now()
	userLimit, exists := r.users[userID]

	if !exists {
		// First request from this user
		r.users[userID] = &UserRateLimit{
			Count:       1,
			WindowStart: now,
		}
		return true
	}

	userLimit.mu.Lock()
	defer userLimit.mu.Unlock()

	// Check if window has expired
	if now.Sub(userLimit.WindowStart) >= r.config.Window {
		// Reset window
		userLimit.Count = 1
		userLimit.WindowStart = now
		return true
	}

	// Check if under limit
	if userLimit.Count < r.config.MaxRequests {
		userLimit.Count++
		return true
	}

	return false
}

// cleanupExpired removes expired rate limit entries
func (r *RatingRateLimiter) cleanupExpired() {
	ticker := time.NewTicker(time.Minute * 5) // Cleanup every 5 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			r.mu.Lock()
			now := time.Now()
			for userID, userLimit := range r.users {
				userLimit.mu.RLock()
				if now.Sub(userLimit.WindowStart) >= r.config.Window*2 {
					delete(r.users, userID)
				}
				userLimit.mu.RUnlock()
			}
			r.mu.Unlock()
		}
	}
}

// UpdateConfig updates the rate limiting configuration
func (r *RatingRateLimiter) UpdateConfig(maxRequests int, window time.Duration, enabled bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	
	r.config.MaxRequests = maxRequests
	r.config.Window = window
	r.config.Enabled = enabled
}

// GetStats returns rate limiting statistics
func (r *RatingRateLimiter) GetStats() map[string]interface{} {
	r.mu.RLock()
	defer r.mu.RUnlock()

	stats := map[string]interface{}{
		"active_users":  len(r.users),
		"max_requests":  r.config.MaxRequests,
		"window_seconds": int(r.config.Window.Seconds()),
		"enabled":       r.config.Enabled,
	}

	return stats
}

// RatingValidationMiddleware validates rating request data
func RatingValidationMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Only validate for POST and PUT requests
		if c.Request.Method != http.MethodPost && c.Request.Method != http.MethodPut {
			c.Next()
			return
		}

		var requestData map[string]interface{}
		if err := c.ShouldBindJSON(&requestData); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error": "invalid_json",
				"message": "Invalid JSON format",
				"details": err.Error(),
			})
			c.Abort()
			return
		}

		// Validate overall rating (required for POST)
		if c.Request.Method == http.MethodPost {
			overallRating, exists := requestData["overallRating"]
			if !exists {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Overall rating is required",
				})
				c.Abort()
				return
			}

			rating, ok := overallRating.(float64)
			if !ok || rating < 1 || rating > 5 || rating != float64(int(rating)) {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Overall rating must be an integer between 1 and 5",
				})
				c.Abort()
				return
			}
		}

		// Validate optional rating fields
		if difficultyRating, exists := requestData["difficultyRating"]; exists && difficultyRating != nil {
			rating, ok := difficultyRating.(float64)
			if !ok || rating < 1 || rating > 5 || rating != float64(int(rating)) {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Difficulty rating must be an integer between 1 and 5",
				})
				c.Abort()
				return
			}
		}

		if tasteRating, exists := requestData["tasteRating"]; exists && tasteRating != nil {
			rating, ok := tasteRating.(float64)
			if !ok || rating < 1 || rating > 5 || rating != float64(int(rating)) {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Taste rating must be an integer between 1 and 5",
				})
				c.Abort()
				return
			}
		}

		// Validate review text length
		if reviewText, exists := requestData["reviewText"]; exists && reviewText != nil {
			text, ok := reviewText.(string)
			if !ok {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Review text must be a string",
				})
				c.Abort()
				return
			}

			if len(text) > 500 {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Review text cannot exceed 500 characters",
				})
				c.Abort()
				return
			}
		}

		// Validate time fields
		timeFields := []string{"actualPrepTime", "actualCookTime"}
		for _, field := range timeFields {
			if value, exists := requestData[field]; exists && value != nil {
				timeValue, ok := value.(float64)
				if !ok || timeValue < 0 || timeValue > 1440 { // Max 24 hours
					c.JSON(http.StatusBadRequest, gin.H{
						"error": "validation_error",
						"message": fmt.Sprintf("%s must be a positive number of minutes (max 1440)", field),
					})
					c.Abort()
					return
				}
			}
		}

		// Validate cooking context
		if cookingContext, exists := requestData["cookingContext"]; exists && cookingContext != nil {
			context, ok := cookingContext.(string)
			if !ok {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Cooking context must be a string",
				})
				c.Abort()
				return
			}

			validContexts := []string{"weeknight", "weekend", "special_occasion"}
			isValid := false
			for _, valid := range validContexts {
				if context == valid {
					isValid = true
					break
				}
			}

			if !isValid {
				c.JSON(http.StatusBadRequest, gin.H{
					"error": "validation_error",
					"message": "Cooking context must be one of: weeknight, weekend, special_occasion",
				})
				c.Abort()
				return
			}
		}

		// Store validated data back in context
		c.Set("validatedRequestData", requestData)
		c.Next()
	}
}

// RecipeExistsMiddleware validates that the recipe exists
func RecipeExistsMiddleware(db interface{}) gin.HandlerFunc {
	// In a real implementation, this would use the actual database connection
	// For now, we'll create a placeholder that can be enhanced later
	return func(c *gin.Context) {
		recipeID := c.Param("id")
		if recipeID == "" {
			c.JSON(http.StatusBadRequest, gin.H{
				"error": "validation_error",
				"message": "Recipe ID is required",
			})
			c.Abort()
			return
		}

		// Validate UUID format
		_, err := uuid.Parse(recipeID)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error": "validation_error",
				"message": "Invalid recipe ID format",
			})
			c.Abort()
			return
		}

		// Store recipe ID in context
		c.Set("recipeID", recipeID)
		c.Next()
	}
}

// AdminOnlyMiddleware restricts access to admin users only
func AdminOnlyMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Check if user has admin role (implementation depends on auth system)
		userRole, exists := c.Get("userRole")
		if !exists || userRole != "admin" {
			c.JSON(http.StatusForbidden, gin.H{
				"error": "access_denied",
				"message": "Admin access required",
			})
			c.Abort()
			return
		}

		c.Next()
	}
}