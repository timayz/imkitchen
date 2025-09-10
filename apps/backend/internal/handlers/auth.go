package handlers

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/services"
)

// Register handles user registration
func Register(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		var req services.RegisterRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": formatValidationError(err.Error()),
			})
			return
		}

		// Additional validation
		if err := validateRegistrationRequest(req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Validation failed",
				"status":  "error",
				"details": err,
			})
			return
		}

		// Register user with Supabase Auth
		response, err := authService.Register(req)
		if err != nil {
			statusCode := http.StatusBadRequest
			errorMessage := "Registration failed"
			
			// Handle specific Supabase errors
			if strings.Contains(err.Error(), "already registered") {
				statusCode = http.StatusConflict
				errorMessage = "User already exists"
			} else if strings.Contains(err.Error(), "invalid email") {
				errorMessage = "Invalid email format"
			} else if strings.Contains(err.Error(), "weak password") {
				errorMessage = "Password is too weak"
			}
			
			c.JSON(statusCode, gin.H{
				"error":   errorMessage,
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusCreated, gin.H{
			"message": "User registered successfully",
			"status":  "success",
			"data":    response,
		})
	}
}

// Login handles user authentication
func Login(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		var req services.LoginRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Authenticate user with Supabase Auth
		response, err := authService.Login(req)
		if err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":   "Authentication failed",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Login successful",
			"status":  "success",
			"data":    response,
		})
	}
}

// RefreshToken handles JWT token refresh
func RefreshToken(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		var req services.RefreshTokenRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Refresh token with Supabase Auth
		response, err := authService.RefreshToken(req)
		if err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":   "Token refresh failed",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Token refreshed successfully",
			"status":  "success",
			"data":    response,
		})
	}
}

// Logout handles user logout
func Logout(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		// Get the access token from context (set by auth middleware)
		token, exists := c.Get("Token")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "No access token found",
				"status": "error",
			})
			return
		}

		tokenStr, ok := token.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid token format",
				"status": "error",
			})
			return
		}

		// Logout user with Supabase Auth
		err := authService.Logout(tokenStr)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Logout failed",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Logout successful",
			"status":  "success",
		})
	}
}

// ForgotPassword handles password reset requests
func ForgotPassword(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		var req services.ForgotPasswordRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Send password reset email with Supabase Auth
		err := authService.ForgotPassword(req)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to send reset password email",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Password reset email sent successfully",
			"status":  "success",
		})
	}
}

// ResetPassword handles password reset
func ResetPassword(authService *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		var req services.ResetPasswordRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Reset password with Supabase Auth
		err := authService.ResetPassword(req)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Password reset failed",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Password reset successfully",
			"status":  "success",
		})
	}
}

// Helper functions for validation and error formatting
func validateRegistrationRequest(req services.RegisterRequest) string {
	if len(req.Email) == 0 {
		return "Email is required"
	}
	if len(req.Email) > 255 {
		return "Email is too long (max 255 characters)"
	}
	if len(req.Password) < 8 {
		return "Password must be at least 8 characters long"
	}
	if len(req.Password) > 128 {
		return "Password is too long (max 128 characters)"
	}
	if len(req.Name) < 2 {
		return "Name must be at least 2 characters long"
	}
	if len(req.Name) > 100 {
		return "Name is too long (max 100 characters)"
	}
	
	// Check for special characters in password
	hasSpecial := strings.ContainsAny(req.Password, "!@#$%^&*()")
	if !hasSpecial {
		return "Password must contain at least one special character (!@#$%^&*())"
	}
	
	return ""
}

func formatValidationError(err string) string {
	// Clean up Gin binding errors for better user experience
	if strings.Contains(err, "email") {
		return "Invalid email format"
	}
	if strings.Contains(err, "min") && strings.Contains(err, "password") {
		return "Password must be at least 8 characters long"
	}
	if strings.Contains(err, "required") {
		return "Missing required field"
	}
	return "Invalid input format"
}