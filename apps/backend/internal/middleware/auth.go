package middleware

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/services"
)

// RequireAuth returns a gin.HandlerFunc that validates JWT tokens using Supabase Auth
func RequireAuth(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		// Get Authorization header
		authHeader := c.GetHeader("Authorization")
		
		if authHeader == "" {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "Authorization header required",
				"status": "error",
			})
			c.Abort()
			return
		}

		// Check Bearer token format
		parts := strings.Split(authHeader, " ")
		if len(parts) != 2 || parts[0] != "Bearer" {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "Invalid authorization header format",
				"status": "error",
			})
			c.Abort()
			return
		}

		tokenString := parts[1]
		
		if tokenString == "" {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "Empty token",
				"status": "error",
			})
			c.Abort()
			return
		}

		// Validate JWT token using Supabase Auth service
		userID, err := authService.ValidateToken(tokenString)
		if err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "Invalid or expired token",
				"status": "error",
				"details": err.Error(),
			})
			c.Abort()
			return
		}

		// Get user details from token
		user, err := authService.GetUserFromToken(tokenString)
		if err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error": "Failed to retrieve user information",
				"status": "error",
				"details": err.Error(),
			})
			c.Abort()
			return
		}

		// Set user context for downstream handlers
		c.Set("UserID", userID)
		c.Set("User", user)
		c.Set("Token", tokenString)
		
		c.Next()
	}
}

