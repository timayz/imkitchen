package middleware

import (
	"crypto/rand"
	"encoding/hex"

	"github.com/gin-gonic/gin"
)

const RequestIDKey = "X-Request-ID"

// RequestID returns a gin.HandlerFunc that generates a unique request ID
func RequestID() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Check if request ID already exists in header
		requestID := c.GetHeader(RequestIDKey)
		
		if requestID == "" {
			// Generate new request ID
			requestID = generateRequestID()
		}
		
		// Set request ID in context and response header
		c.Set("RequestID", requestID)
		c.Header(RequestIDKey, requestID)
		
		c.Next()
	}
}

// generateRequestID creates a random hex string for request tracking
func generateRequestID() string {
	bytes := make([]byte, 16)
	if _, err := rand.Read(bytes); err != nil {
		// Fallback to timestamp-based ID if random generation fails
		return "fallback-id"
	}
	return hex.EncodeToString(bytes)
}