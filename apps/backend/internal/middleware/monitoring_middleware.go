package middleware

import (
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
)

// MonitoringAuthMiddleware validates authentication for monitoring endpoints
func MonitoringAuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Check for authentication token in header
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "authentication_required",
				"message": "Authentication required for monitoring endpoints",
			})
			c.Abort()
			return
		}

		// Validate Bearer token format
		if len(authHeader) < 7 || authHeader[:7] != "Bearer " {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "invalid_auth_format",
				"message": "Authorization header must use Bearer token format",
			})
			c.Abort()
			return
		}

		token := authHeader[7:]
		if token == "" {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "missing_token",
				"message": "Bearer token is required",
			})
			c.Abort()
			return
		}

		// In production, validate token against auth service
		// For now, check for basic token presence and format
		if len(token) < 16 {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "invalid_token",
				"message": "Invalid authentication token",
			})
			c.Abort()
			return
		}

		// Store authenticated context
		c.Set("authenticated", true)
		c.Set("auth_token", token)
		c.Next()
	}
}

// MonitoringRateLimitConfig holds monitoring API rate limiting configuration
type MonitoringRateLimitConfig struct {
	MaxRequests int           `json:"maxRequests"`
	Window      time.Duration `json:"window"`
	Enabled     bool          `json:"enabled"`
}

// MonitoringUserLimit tracks rate limiting for monitoring API access
type MonitoringUserLimit struct {
	Count       int       `json:"count"`
	WindowStart time.Time `json:"windowStart"`
	mu          sync.RWMutex
}

// MonitoringRateLimiter manages rate limiting for monitoring APIs
type MonitoringRateLimiter struct {
	clients map[string]*MonitoringUserLimit
	config  *MonitoringRateLimitConfig
	mu      sync.RWMutex
}

// NewMonitoringRateLimiter creates a new monitoring API rate limiter
func NewMonitoringRateLimiter() *MonitoringRateLimiter {
	limiter := &MonitoringRateLimiter{
		clients: make(map[string]*MonitoringUserLimit),
		config: &MonitoringRateLimitConfig{
			MaxRequests: 100,               // 100 requests per minute for monitoring
			Window:      time.Minute,       // 1 minute window
			Enabled:     true,
		},
	}
	
	// Start cleanup goroutine
	go limiter.cleanupExpired()
	
	return limiter
}

// MonitoringRateLimit middleware for monitoring API endpoints
func (m *MonitoringRateLimiter) MonitoringRateLimit() gin.HandlerFunc {
	return func(c *gin.Context) {
		if !m.config.Enabled {
			c.Next()
			return
		}

		// Get client identifier (IP + User Agent for monitoring APIs)
		clientIP := c.ClientIP()
		userAgent := c.GetHeader("User-Agent")
		clientID := clientIP + ":" + userAgent
		
		// Use hash for consistent client identification
		if len(clientID) > 100 {
			clientID = clientID[:100] // Truncate to prevent excessive memory usage
		}

		// Check rate limit
		if !m.allowRequest(clientID) {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error": "monitoring_rate_limit_exceeded",
				"message": "Too many monitoring API requests. Please reduce request frequency.",
				"max_requests": m.config.MaxRequests,
				"window_seconds": int(m.config.Window.Seconds()),
				"retry_after": int(m.config.Window.Seconds()),
			})
			c.Abort()
			return
		}

		c.Next()
	}
}

// allowRequest checks if a monitoring API request is allowed for the client
func (m *MonitoringRateLimiter) allowRequest(clientID string) bool {
	m.mu.Lock()
	defer m.mu.Unlock()

	now := time.Now()
	clientLimit, exists := m.clients[clientID]

	if !exists {
		// First request from this client
		m.clients[clientID] = &MonitoringUserLimit{
			Count:       1,
			WindowStart: now,
		}
		return true
	}

	clientLimit.mu.Lock()
	defer clientLimit.mu.Unlock()

	// Check if window has expired
	if now.Sub(clientLimit.WindowStart) >= m.config.Window {
		// Reset window
		clientLimit.Count = 1
		clientLimit.WindowStart = now
		return true
	}

	// Check if under limit
	if clientLimit.Count < m.config.MaxRequests {
		clientLimit.Count++
		return true
	}

	return false
}

// cleanupExpired removes expired rate limit entries for monitoring
func (m *MonitoringRateLimiter) cleanupExpired() {
	ticker := time.NewTicker(time.Minute * 5) // Cleanup every 5 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			m.mu.Lock()
			now := time.Now()
			for clientID, clientLimit := range m.clients {
				clientLimit.mu.RLock()
				if now.Sub(clientLimit.WindowStart) >= m.config.Window*2 {
					delete(m.clients, clientID)
				}
				clientLimit.mu.RUnlock()
			}
			m.mu.Unlock()
		}
	}
}

// UpdateMonitoringConfig updates the monitoring API rate limiting configuration
func (m *MonitoringRateLimiter) UpdateMonitoringConfig(maxRequests int, window time.Duration, enabled bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	
	m.config.MaxRequests = maxRequests
	m.config.Window = window
	m.config.Enabled = enabled
}

// GetMonitoringStats returns monitoring rate limiting statistics
func (m *MonitoringRateLimiter) GetMonitoringStats() map[string]interface{} {
	m.mu.RLock()
	defer m.mu.RUnlock()

	stats := map[string]interface{}{
		"active_clients": len(m.clients),
		"max_requests":   m.config.MaxRequests,
		"window_seconds": int(m.config.Window.Seconds()),
		"enabled":        m.config.Enabled,
		"endpoint_type":  "monitoring",
	}

	return stats
}

// AdminMonitoringAuthMiddleware validates admin authentication for monitoring management
func AdminMonitoringAuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// First check basic authentication
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "admin_authentication_required", 
				"message": "Admin authentication required for monitoring management",
			})
			c.Abort()
			return
		}

		// Validate Bearer token format
		if len(authHeader) < 7 || authHeader[:7] != "Bearer " {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "invalid_admin_auth_format",
				"message": "Authorization header must use Bearer token format",
			})
			c.Abort()
			return
		}

		token := authHeader[7:]
		
		// In production, validate admin token against auth service
		// For now, check for admin token format (should be longer)
		if len(token) < 32 {
			c.JSON(http.StatusForbidden, gin.H{
				"error": "insufficient_privileges",
				"message": "Admin privileges required for monitoring management",
			})
			c.Abort()
			return
		}

		// Store admin context
		c.Set("admin_authenticated", true)
		c.Set("admin_token", token)
		c.Next()
	}
}