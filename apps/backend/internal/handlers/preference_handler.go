package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// PreferenceHandler handles HTTP requests for user preferences
type PreferenceHandler struct {
	preferenceService *services.PreferenceService
}

// NewPreferenceHandler creates a new preference handler
func NewPreferenceHandler(preferenceService *services.PreferenceService) *PreferenceHandler {
	return &PreferenceHandler{
		preferenceService: preferenceService,
	}
}

// GetUserPreferences handles GET /api/v1/users/preferences
func (ph *PreferenceHandler) GetUserPreferences(c *gin.Context) {
	// Extract user ID from JWT context (set by auth middleware)
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "User not authenticated",
		})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "Invalid user ID format",
		})
		return
	}

	preferences, err := ph.preferenceService.GetUserPreferences(userUUID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to retrieve user preferences",
			"message": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": preferences,
		"metadata": gin.H{
			"retrievedAt": "2025-09-08T00:00:00Z",
		},
	})
}

// UpdateUserPreferences handles PUT /api/v1/users/preferences
func (ph *PreferenceHandler) UpdateUserPreferences(c *gin.Context) {
	// Extract user ID from JWT context
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "User not authenticated",
		})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "Invalid user ID format",
		})
		return
	}

	// Parse request body
	var preferences models.CoreUserPreferences
	if err := c.ShouldBindJSON(&preferences); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"message": err.Error(),
		})
		return
	}

	// Update preferences
	err := ph.preferenceService.UpdateUserPreferences(userUUID, &preferences)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Failed to update preferences",
			"message": err.Error(),
		})
		return
	}

	// Return updated preferences
	updatedPreferences, err := ph.preferenceService.GetUserPreferences(userUUID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to retrieve updated preferences",
			"message": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": updatedPreferences,
		"metadata": gin.H{
			"updatedAt": "2025-09-08T00:00:00Z",
		},
	})
}

// ResetUserPreferences handles POST /api/v1/users/preferences/reset
func (ph *PreferenceHandler) ResetUserPreferences(c *gin.Context) {
	// Extract user ID from JWT context
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "User not authenticated",
		})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "Invalid user ID format",
		})
		return
	}

	// Reset preferences to defaults
	err := ph.preferenceService.ResetUserPreferences(userUUID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to reset preferences",
			"message": err.Error(),
		})
		return
	}

	// Return reset preferences
	preferences, err := ph.preferenceService.GetUserPreferences(userUUID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to retrieve reset preferences",
			"message": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": preferences,
		"metadata": gin.H{
			"resetAt": "2025-09-08T00:00:00Z",
		},
	})
}