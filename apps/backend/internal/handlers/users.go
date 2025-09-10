package handlers

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/services"
)

// GetCurrentUser returns current user information
func GetCurrentUser(userService *services.UserService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

		profile, err := userService.GetUserProfile(userIDStr)
		if err != nil {
			if strings.Contains(err.Error(), "not found") {
				c.JSON(http.StatusNotFound, gin.H{
					"error":   "User profile not found",
					"status":  "error",
					"details": err.Error(),
				})
				return
			}

			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to get user profile",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "User profile retrieved successfully",
			"status":  "success",
			"data":    profile,
		})
	}
}

// UpdateCurrentUser handles user profile updates
func UpdateCurrentUser(userService *services.UserService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

		var req services.UpdateProfileRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Validate the request
		if validationError := services.ValidateUpdateProfileRequest(req); validationError != "" {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Validation failed",
				"status":  "error",
				"details": validationError,
			})
			return
		}

		profile, err := userService.UpdateUserProfile(userIDStr, req)
		if err != nil {
			if strings.Contains(err.Error(), "not found") {
				c.JSON(http.StatusNotFound, gin.H{
					"error":   "User profile not found",
					"status":  "error",
					"details": err.Error(),
				})
				return
			}

			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to update user profile",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "User profile updated successfully",
			"status":  "success",
			"data":    profile,
		})
	}
}

// ChangeUserEmail handles user email change requests
func ChangeUserEmail(userService *services.UserService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

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

		var req services.ChangeEmailRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		err := userService.ChangeEmail(userIDStr, req, tokenStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Failed to initiate email change",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Email change verification sent successfully",
			"status":  "success",
		})
	}
}

// DeleteCurrentUser handles user account deletion
func DeleteCurrentUser(userService *services.UserService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

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

		exportData, err := userService.DeleteUserAccount(userIDStr, tokenStr)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to delete user account",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "User account deleted successfully",
			"status":  "success",
			"data":    exportData,
		})
	}
}

// ExportUserData handles user data export requests
func ExportUserData(userService *services.UserService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

		exportData, err := userService.ExportUserData(userIDStr)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to export user data",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "User data exported successfully",
			"status":  "success",
			"data":    exportData,
		})
	}
}

// GetUserPreferences retrieves current user cooking preferences
func GetUserPreferences(preferenceService services.UserPreferenceService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

		userUUID, err := uuid.Parse(userIDStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":  "Invalid user ID",
				"status": "error",
			})
			return
		}

		preferences, err := preferenceService.GetUserPreferences(userUUID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to get user preferences",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "User preferences retrieved successfully",
			"status":  "success",
			"data":    preferences,
		})
	}
}

// UpdateUserPreferences updates user cooking preferences
func UpdateUserPreferences(preferenceService services.UserPreferenceService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

		userUUID, err := uuid.Parse(userIDStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":  "Invalid user ID",
				"status": "error",
			})
			return
		}

		var req services.UpdatePreferencesRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Validate preferences
		if errors := preferenceService.ValidatePreferences(&req); len(errors) > 0 {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Validation failed",
				"status":  "error",
				"details": errors,
			})
			return
		}

		preferences, err := preferenceService.UpdateUserPreferences(userUUID, &req)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to update user preferences",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "User preferences updated successfully",
			"status":  "success",
			"data":    preferences,
		})
	}
}

// GetPreferenceHistory retrieves user preference change history
func GetPreferenceHistory(preferenceService services.UserPreferenceService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":  "User not authenticated",
				"status": "error",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":  "Invalid user ID format",
				"status": "error",
			})
			return
		}

		userUUID, err := uuid.Parse(userIDStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":  "Invalid user ID",
				"status": "error",
			})
			return
		}

		history, err := preferenceService.GetPreferenceHistory(userUUID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to get preference history",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Preference history retrieved successfully",
			"status":  "success",
			"data":    history,
		})
	}
}