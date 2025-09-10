package tests

import (
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"github.com/imkitchen/backend/internal/middleware"
	"github.com/stretchr/testify/assert"
)

func TestAuthMiddleware(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Missing authorization header", func(t *testing.T) {
		router := gin.New()
		router.Use(middleware.RequireAuth())
		router.GET("/protected", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "success"})
		})

		req, _ := http.NewRequest("GET", "/protected", nil)
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusUnauthorized, resp.Code)
		assert.Contains(t, resp.Body.String(), "Authorization header required")
	})

	t.Run("Invalid authorization header format", func(t *testing.T) {
		router := gin.New()
		router.Use(middleware.RequireAuth())
		router.GET("/protected", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "success"})
		})

		req, _ := http.NewRequest("GET", "/protected", nil)
		req.Header.Set("Authorization", "InvalidFormat")
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusUnauthorized, resp.Code)
		assert.Contains(t, resp.Body.String(), "Invalid authorization header format")
	})

	t.Run("Valid JWT token allows access", func(t *testing.T) {
		// Create a valid JWT token for testing
		testUserID := "test-user-123"
		token := createValidJWTToken(t, testUserID)

		router := gin.New()
		router.Use(middleware.RequireAuth())
		router.GET("/protected", func(c *gin.Context) {
			userID := c.GetString("UserID")
			c.JSON(http.StatusOK, gin.H{"userID": userID})
		})

		req, _ := http.NewRequest("GET", "/protected", nil)
		req.Header.Set("Authorization", "Bearer "+token)
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		// Should pass JWT validation
		assert.Equal(t, http.StatusOK, resp.Code)
		assert.Contains(t, resp.Body.String(), testUserID)
	})

	t.Run("Invalid JWT token is rejected", func(t *testing.T) {
		router := gin.New()
		router.Use(middleware.RequireAuth())
		router.GET("/protected", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "success"})
		})

		req, _ := http.NewRequest("GET", "/protected", nil)
		req.Header.Set("Authorization", "Bearer invalid.jwt.token")
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusUnauthorized, resp.Code)
		assert.Contains(t, resp.Body.String(), "Invalid or expired token")
	})
}

func TestSecurityMiddleware(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Sets security headers", func(t *testing.T) {
		router := gin.New()
		router.Use(middleware.Security())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "test"})
		})

		req, _ := http.NewRequest("GET", "/test", nil)
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusOK, resp.Code)
		assert.Equal(t, "nosniff", resp.Header().Get("X-Content-Type-Options"))
		assert.Equal(t, "DENY", resp.Header().Get("X-Frame-Options"))
		assert.Equal(t, "1; mode=block", resp.Header().Get("X-XSS-Protection"))
		assert.Equal(t, "default-src 'self'", resp.Header().Get("Content-Security-Policy"))
	})
}

func TestCORSMiddleware(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Handles CORS headers in development", func(t *testing.T) {
		// Set development environment
		os.Setenv("ENVIRONMENT", "development")
		defer os.Unsetenv("ENVIRONMENT")

		router := gin.New()
		router.Use(middleware.CORS())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "test"})
		})

		// Test without origin header (should get wildcard)
		req, _ := http.NewRequest("GET", "/test", nil)
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusOK, resp.Code)
		assert.Equal(t, "*", resp.Header().Get("Access-Control-Allow-Origin"))

		// Test with origin header (should echo back the origin in development)
		req2, _ := http.NewRequest("GET", "/test", nil)
		req2.Header.Set("Origin", "http://localhost:3000")
		resp2 := httptest.NewRecorder()
		router.ServeHTTP(resp2, req2)

		assert.Equal(t, http.StatusOK, resp2.Code)
		// In development with "*" wildcard, any origin is allowed and echoed back
		assert.Equal(t, "http://localhost:3000", resp2.Header().Get("Access-Control-Allow-Origin"))
		assert.Contains(t, resp2.Header().Get("Access-Control-Allow-Methods"), "GET")
		assert.Contains(t, resp2.Header().Get("Access-Control-Allow-Headers"), "Authorization")
	})

	t.Run("Restricts CORS in production", func(t *testing.T) {
		// Set production environment
		os.Setenv("ENVIRONMENT", "production")
		defer os.Unsetenv("ENVIRONMENT")

		router := gin.New()
		router.Use(middleware.CORS())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "test"})
		})

		// Test with allowed origin
		req, _ := http.NewRequest("GET", "/test", nil)
		req.Header.Set("Origin", "https://imkitchen.app")
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusOK, resp.Code)
		assert.Equal(t, "https://imkitchen.app", resp.Header().Get("Access-Control-Allow-Origin"))

		// Test with disallowed origin
		req2, _ := http.NewRequest("GET", "/test", nil)
		req2.Header.Set("Origin", "https://malicious-site.com")
		resp2 := httptest.NewRecorder()
		router.ServeHTTP(resp2, req2)

		assert.Equal(t, http.StatusOK, resp.Code)
		assert.Empty(t, resp2.Header().Get("Access-Control-Allow-Origin"))
	})

	t.Run("Handles OPTIONS preflight request", func(t *testing.T) {
		router := gin.New()
		router.Use(middleware.CORS())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"message": "test"})
		})

		req, _ := http.NewRequest("OPTIONS", "/test", nil)
		resp := httptest.NewRecorder()
		router.ServeHTTP(resp, req)

		assert.Equal(t, http.StatusNoContent, resp.Code)
	})
}

// createValidJWTToken creates a valid JWT token for testing
func createValidJWTToken(t *testing.T, userID string) string {
	// Use the same secret as the middleware (development default)
	secret := "dev-secret-key-change-in-production"

	// Create token with claims
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"sub": userID,
		"exp": time.Now().Add(time.Hour).Unix(),
		"iat": time.Now().Unix(),
	})

	// Sign token
	tokenString, err := token.SignedString([]byte(secret))
	if err != nil {
		t.Fatalf("Failed to create test JWT token: %v", err)
	}

	return tokenString
}