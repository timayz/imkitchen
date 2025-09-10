package middleware

import (
	"github.com/gin-gonic/gin"
)

// Security returns a gin.HandlerFunc that sets security headers
func Security() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Prevent XSS attacks
		c.Header("X-Content-Type-Options", "nosniff")
		c.Header("X-Frame-Options", "DENY")
		c.Header("X-XSS-Protection", "1; mode=block")
		
		// Enforce HTTPS in production
		// c.Header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
		
		// Content Security Policy - can be expanded based on needs
		c.Header("Content-Security-Policy", "default-src 'self'")
		
		// Referrer policy
		c.Header("Referrer-Policy", "strict-origin-when-cross-origin")
		
		c.Next()
	}
}