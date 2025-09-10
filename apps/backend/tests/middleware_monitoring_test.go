package tests

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/middleware"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
)

// MonitoringMiddlewareTestSuite tests monitoring authentication and rate limiting middleware
type MonitoringMiddlewareTestSuite struct {
	suite.Suite
	router      *gin.Engine
	rateLimiter *middleware.MonitoringRateLimiter
}

func (suite *MonitoringMiddlewareTestSuite) SetupTest() {
	gin.SetMode(gin.TestMode)
	suite.router = gin.New()
	suite.rateLimiter = middleware.NewMonitoringRateLimiter()
}

func TestMonitoringMiddlewareTestSuite(t *testing.T) {
	suite.Run(t, new(MonitoringMiddlewareTestSuite))
}

// Test MonitoringAuthMiddleware
func (suite *MonitoringMiddlewareTestSuite) TestMonitoringAuthMiddleware() {
	suite.router.GET("/test", middleware.MonitoringAuthMiddleware(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "authenticated"})
	})

	suite.Run("Missing Authorization Header", func() {
		req := httptest.NewRequest("GET", "/test", nil)
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "authentication_required")
	})

	suite.Run("Invalid Authorization Format", func() {
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "InvalidFormat token123")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "invalid_auth_format")
	})

	suite.Run("Empty Bearer Token", func() {
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer ")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "missing_token")
	})

	suite.Run("Token Too Short", func() {
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer short")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "invalid_token")
	})

	suite.Run("Valid Authentication", func() {
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer validtokenthatis16chars")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusOK, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "authenticated")
	})
}

// Test AdminMonitoringAuthMiddleware
func (suite *MonitoringMiddlewareTestSuite) TestAdminMonitoringAuthMiddleware() {
	suite.router.GET("/admin", middleware.AdminMonitoringAuthMiddleware(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "admin_authenticated"})
	})

	suite.Run("Missing Admin Authorization", func() {
		req := httptest.NewRequest("GET", "/admin", nil)
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "admin_authentication_required")
	})

	suite.Run("Invalid Admin Token Format", func() {
		req := httptest.NewRequest("GET", "/admin", nil)
		req.Header.Set("Authorization", "Basic admintoken")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "invalid_admin_auth_format")
	})

	suite.Run("Admin Token Too Short", func() {
		req := httptest.NewRequest("GET", "/admin", nil)
		req.Header.Set("Authorization", "Bearer shortadmintoken")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusForbidden, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "insufficient_privileges")
	})

	suite.Run("Valid Admin Authentication", func() {
		req := httptest.NewRequest("GET", "/admin", nil)
		req.Header.Set("Authorization", "Bearer validadmintokenthatislong32char")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusOK, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "admin_authenticated")
	})
}

// Test MonitoringRateLimiter
func (suite *MonitoringMiddlewareTestSuite) TestMonitoringRateLimit() {
	// Set low limits for testing
	suite.rateLimiter.UpdateMonitoringConfig(3, time.Second*2, true)

	suite.router.GET("/rate-test", suite.rateLimiter.MonitoringRateLimit(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// First few requests should succeed
	for i := 0; i < 3; i++ {
		suite.Run("Request Within Limit", func() {
			req := httptest.NewRequest("GET", "/rate-test", nil)
			req.Header.Set("User-Agent", "TestAgent")
			w := httptest.NewRecorder()
			suite.router.ServeHTTP(w, req)

			assert.Equal(suite.T(), http.StatusOK, w.Code)
			assert.Contains(suite.T(), w.Body.String(), "success")
		})
	}

	// Fourth request should be rate limited
	suite.Run("Request Exceeds Limit", func() {
		req := httptest.NewRequest("GET", "/rate-test", nil)
		req.Header.Set("User-Agent", "TestAgent")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusTooManyRequests, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "monitoring_rate_limit_exceeded")
	})

	// After window expires, requests should work again
	suite.Run("Request After Window Reset", func() {
		time.Sleep(time.Second * 3) // Wait for window to expire

		req := httptest.NewRequest("GET", "/rate-test", nil)
		req.Header.Set("User-Agent", "TestAgent")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)

		assert.Equal(suite.T(), http.StatusOK, w.Code)
		assert.Contains(suite.T(), w.Body.String(), "success")
	})
}

// Test rate limiter with different clients
func (suite *MonitoringMiddlewareTestSuite) TestMonitoringRateLimitDifferentClients() {
	suite.rateLimiter.UpdateMonitoringConfig(2, time.Second*5, true)

	suite.router.GET("/multi-client", suite.rateLimiter.MonitoringRateLimit(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// Client A makes requests
	for i := 0; i < 2; i++ {
		req := httptest.NewRequest("GET", "/multi-client", nil)
		req.Header.Set("User-Agent", "ClientA")
		req.RemoteAddr = "192.168.1.1:1234"
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		assert.Equal(suite.T(), http.StatusOK, w.Code)
	}

	// Client B makes requests (should not be affected by Client A's limit)
	for i := 0; i < 2; i++ {
		req := httptest.NewRequest("GET", "/multi-client", nil)
		req.Header.Set("User-Agent", "ClientB")
		req.RemoteAddr = "192.168.1.2:1234"
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		assert.Equal(suite.T(), http.StatusOK, w.Code)
	}

	// Client A's third request should be rate limited
	req := httptest.NewRequest("GET", "/multi-client", nil)
	req.Header.Set("User-Agent", "ClientA")
	req.RemoteAddr = "192.168.1.1:1234"
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	assert.Equal(suite.T(), http.StatusTooManyRequests, w.Code)

	// Client B's third request should also be rate limited
	req = httptest.NewRequest("GET", "/multi-client", nil)
	req.Header.Set("User-Agent", "ClientB")
	req.RemoteAddr = "192.168.1.2:1234"
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	assert.Equal(suite.T(), http.StatusTooManyRequests, w.Code)
}

// Test rate limiter disabled
func (suite *MonitoringMiddlewareTestSuite) TestMonitoringRateLimitDisabled() {
	suite.rateLimiter.UpdateMonitoringConfig(1, time.Second, false) // Disabled

	suite.router.GET("/disabled-limit", suite.rateLimiter.MonitoringRateLimit(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// Multiple requests should all succeed when disabled
	for i := 0; i < 5; i++ {
		req := httptest.NewRequest("GET", "/disabled-limit", nil)
		req.Header.Set("User-Agent", "TestAgent")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		assert.Equal(suite.T(), http.StatusOK, w.Code)
	}
}

// Test rate limiter statistics
func (suite *MonitoringMiddlewareTestSuite) TestMonitoringRateLimitStats() {
	suite.rateLimiter.UpdateMonitoringConfig(5, time.Minute, true)

	// Make some requests
	suite.router.GET("/stats-test", suite.rateLimiter.MonitoringRateLimit(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	for i := 0; i < 3; i++ {
		req := httptest.NewRequest("GET", "/stats-test", nil)
		req.Header.Set("User-Agent", "StatsTestAgent")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		assert.Equal(suite.T(), http.StatusOK, w.Code)
	}

	// Check stats
	stats := suite.rateLimiter.GetMonitoringStats()
	assert.Equal(suite.T(), 1, stats["active_clients"])
	assert.Equal(suite.T(), 5, stats["max_requests"])
	assert.Equal(suite.T(), 60, stats["window_seconds"])
	assert.Equal(suite.T(), true, stats["enabled"])
	assert.Equal(suite.T(), "monitoring", stats["endpoint_type"])
}

// Test combined authentication and rate limiting
func (suite *MonitoringMiddlewareTestSuite) TestCombinedAuthAndRateLimit() {
	suite.rateLimiter.UpdateMonitoringConfig(2, time.Second*3, true)

	suite.router.GET("/combined", 
		middleware.MonitoringAuthMiddleware(),
		suite.rateLimiter.MonitoringRateLimit(),
		func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "authenticated_and_rate_limited"})
		})

	// Request without auth should fail before rate limiting
	req := httptest.NewRequest("GET", "/combined", nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	assert.Equal(suite.T(), http.StatusUnauthorized, w.Code)

	// Valid auth requests should work within rate limit
	for i := 0; i < 2; i++ {
		req := httptest.NewRequest("GET", "/combined", nil)
		req.Header.Set("Authorization", "Bearer validtokenthatis16chars")
		req.Header.Set("User-Agent", "CombinedTestAgent")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		assert.Equal(suite.T(), http.StatusOK, w.Code)
	}

	// Third request should be rate limited even with valid auth
	req = httptest.NewRequest("GET", "/combined", nil)
	req.Header.Set("Authorization", "Bearer validtokenthatis16chars")
	req.Header.Set("User-Agent", "CombinedTestAgent")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	assert.Equal(suite.T(), http.StatusTooManyRequests, w.Code)
}

// Test rate limiter configuration updates
func (suite *MonitoringMiddlewareTestSuite) TestRateLimitConfigUpdate() {
	// Start with restrictive config
	suite.rateLimiter.UpdateMonitoringConfig(1, time.Second, true)

	suite.router.GET("/config-test", suite.rateLimiter.MonitoringRateLimit(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// First request should succeed
	req := httptest.NewRequest("GET", "/config-test", nil)
	req.Header.Set("User-Agent", "ConfigTestAgent")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	assert.Equal(suite.T(), http.StatusOK, w.Code)

	// Second request should fail
	req = httptest.NewRequest("GET", "/config-test", nil)
	req.Header.Set("User-Agent", "ConfigTestAgent")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	assert.Equal(suite.T(), http.StatusTooManyRequests, w.Code)

	// Update config to be more permissive
	suite.rateLimiter.UpdateMonitoringConfig(10, time.Second, true)

	// Wait for window reset and test again
	time.Sleep(time.Second * 2)

	// Should now allow more requests
	for i := 0; i < 5; i++ {
		req := httptest.NewRequest("GET", "/config-test", nil)
		req.Header.Set("User-Agent", "ConfigTestAgent2")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		assert.Equal(suite.T(), http.StatusOK, w.Code)
	}
}