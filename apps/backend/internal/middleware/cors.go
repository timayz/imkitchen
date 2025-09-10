package middleware

import (
	"net/http"
	"os"
	"strings"

	"github.com/gin-gonic/gin"
)

// CORS returns a gin.HandlerFunc for handling Cross-Origin Resource Sharing
func CORS() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Configure CORS origins based on environment
		allowedOrigins := getAllowedOrigins()
		origin := c.Request.Header.Get("Origin")
		
		if isAllowedOrigin(origin, allowedOrigins) {
			c.Header("Access-Control-Allow-Origin", origin)
		} else if len(allowedOrigins) == 1 && allowedOrigins[0] == "*" {
			// Only allow wildcard in development
			c.Header("Access-Control-Allow-Origin", "*")
		}
		c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")
		c.Header("Access-Control-Allow-Headers", "Origin, Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, X-Request-ID")
		c.Header("Access-Control-Expose-Headers", "Content-Length, X-Request-ID")
		c.Header("Access-Control-Allow-Credentials", "true")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(http.StatusNoContent)
			return
		}

		c.Next()
	}
}

// getAllowedOrigins returns the list of allowed CORS origins from environment
func getAllowedOrigins() []string {
	// Check for environment-specific configuration
	allowedOrigins := os.Getenv("ALLOWED_ORIGINS")
	if allowedOrigins != "" {
		return strings.Split(allowedOrigins, ",")
	}
	
	// Default based on environment
	env := os.Getenv("ENVIRONMENT")
	switch env {
	case "production":
		// In production, specify exact domains (these should be set via ALLOWED_ORIGINS)
		return []string{"https://imkitchen.app", "https://app.imkitchen.app"}
	case "staging":
		return []string{"https://staging.imkitchen.app", "https://staging-app.imkitchen.app"}
	default:
		// Development - allow localhost and common development ports
		return []string{"*"} // Fallback to wildcard for development only
	}
}

// isAllowedOrigin checks if the origin is in the allowed list
func isAllowedOrigin(origin string, allowedOrigins []string) bool {
	if origin == "" {
		return false
	}
	
	for _, allowed := range allowedOrigins {
		if allowed == "*" || allowed == origin {
			return true
		}
	}
	
	return false
}