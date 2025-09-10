package handlers

import (
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
)

// PreferenceHandler handles HTTP requests for user preferences
type PreferenceHandler struct {
	service        *services.PreferenceService
	preferenceRepo *repositories.PreferenceRepository
}

// NewPreferenceHandler creates a new preference handler
func NewPreferenceHandler(service *services.PreferenceService, preferenceRepo *repositories.PreferenceRepository) *PreferenceHandler {
	return &PreferenceHandler{
		service:        service,
		preferenceRepo: preferenceRepo,
	}
}

// GetUserPreferences handles GET /api/v1/users/preferences
func (ph *PreferenceHandler) GetUserPreferences(c *gin.Context) {
	// Get user ID from JWT middleware (assumed to be set by RequireAuth middleware)
	userIDValue, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error":   "Unauthorized",
			"message": "User ID not found in token",
		})
		return
	}

	userID, ok := userIDValue.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid user ID",
			"message": "User ID format is invalid",
		})
		return
	}

	// Get preferences from service
	preferences, err := ph.service.GetUserPreferences(userID)
	if err != nil {
		if err.Error() == "user not found" {
			c.JSON(http.StatusNotFound, gin.H{
				"error":   "User not found",
				"message": "The specified user does not exist",
			})
			return
		}

		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to retrieve user preferences",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": preferences,
		"metadata": gin.H{
			"retrievedAt": time.Now().Format(time.RFC3339),
		},
	})
}

// UpdateUserPreferences handles PUT /api/v1/users/preferences
func (ph *PreferenceHandler) UpdateUserPreferences(c *gin.Context) {
	// Get user ID from JWT middleware
	userIDValue, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error":   "Unauthorized", 
			"message": "User ID not found in token",
		})
		return
	}

	userID, ok := userIDValue.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid user ID",
			"message": "User ID format is invalid",
		})
		return
	}

	// Parse request body
	var requestBody models.CoreUserPreferences
	if err := c.ShouldBindJSON(&requestBody); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"message": err.Error(),
		})
		return
	}

	// Update preferences through service
	err := ph.service.UpdateUserPreferences(userID, &requestBody)
	if err != nil {
		if err.Error() == "user not found" {
			c.JSON(http.StatusNotFound, gin.H{
				"error":   "User not found",
				"message": "The specified user does not exist",
			})
			return
		}

		// Check if it's a validation error
		if len(err.Error()) > 0 && (
			contains(err.Error(), "validation failed") ||
			contains(err.Error(), "must be between") ||
			contains(err.Error(), "must be one of")) {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Validation error",
				"message": err.Error(),
			})
			return
		}

		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to update user preferences",
		})
		return
	}

	// Return updated preferences
	updatedPreferences, err := ph.service.GetUserPreferences(userID)
	if err != nil {
		// If we can't retrieve updated preferences, still return success
		c.JSON(http.StatusOK, gin.H{
			"message": "Preferences updated successfully",
			"updatedAt": time.Now().Format(time.RFC3339),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Preferences updated successfully",
		"data": updatedPreferences,
		"metadata": gin.H{
			"updatedAt": time.Now().Format(time.RFC3339),
		},
	})
}

// ResetUserPreferences handles POST /api/v1/users/preferences/reset (optional endpoint)
func (ph *PreferenceHandler) ResetUserPreferences(c *gin.Context) {
	// Get user ID from JWT middleware
	userIDValue, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error":   "Unauthorized",
			"message": "User ID not found in token",
		})
		return
	}

	userID, ok := userIDValue.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid user ID",
			"message": "User ID format is invalid",
		})
		return
	}

	// Reset preferences through service
	err := ph.service.ResetUserPreferences(userID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to reset user preferences",
		})
		return
	}

	// Return default preferences
	preferences, _ := ph.service.GetUserPreferences(userID)
	
	c.JSON(http.StatusOK, gin.H{
		"message": "Preferences reset to defaults successfully",
		"data": preferences,
		"metadata": gin.H{
			"resetAt": time.Now().Format(time.RFC3339),
		},
	})
}

// RegisterPreferenceRoutes registers preference-related routes
func RegisterPreferenceRoutes(router *gin.RouterGroup, handler *PreferenceHandler) {
	users := router.Group("/users")
	{
		// Core preferences
		users.GET("/preferences", handler.GetUserPreferences)
		users.PUT("/preferences", handler.UpdateUserPreferences)
		users.POST("/preferences/reset", handler.ResetUserPreferences)
		
		// Weekly patterns
		users.GET("/preferences/patterns", handler.GetUserWeeklyPatterns)
		users.PUT("/preferences/patterns", handler.UpdateUserWeeklyPatterns)
		
		// Favorites
		users.GET("/favorites", handler.GetUserFavorites)
		users.POST("/favorites/:recipeId", handler.AddUserFavorite)
		users.DELETE("/favorites/:recipeId", handler.RemoveUserFavorite)
	}
}

// GetUserFavorites handles GET /api/v1/users/favorites
func (ph *PreferenceHandler) GetUserFavorites(c *gin.Context) {
	userID, err := ph.getUserIDFromContext(c)
	if err != nil {
		return
	}

	// Parse query parameters
	page := 1
	limit := 20
	
	if pageStr := c.Query("page"); pageStr != "" {
		if p, err := strconv.Atoi(pageStr); err == nil && p > 0 {
			page = p
		}
	}
	
	if limitStr := c.Query("limit"); limitStr != "" {
		if l, err := strconv.Atoi(limitStr); err == nil && l > 0 && l <= 50 {
			limit = l
		}
	}

	// Get favorites from repository
	favorites, total, err := ph.preferenceRepo.GetUserFavorites(userID, page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to retrieve favorites",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": favorites,
		"metadata": gin.H{
			"page":       page,
			"limit":      limit,
			"total":      total,
			"totalPages": (total + int64(limit) - 1) / int64(limit),
		},
	})
}

// AddUserFavorite handles POST /api/v1/users/favorites/{recipeId}
func (ph *PreferenceHandler) AddUserFavorite(c *gin.Context) {
	userID, err := ph.getUserIDFromContext(c)
	if err != nil {
		return
	}

	// Parse recipe ID from URL parameter
	recipeIDStr := c.Param("recipeId")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid recipe ID",
			"message": "Recipe ID must be a valid UUID",
		})
		return
	}

	// Add favorite
	favorite, err := ph.preferenceRepo.AddUserFavorite(userID, recipeID)
	if err != nil {
		if err.Error() == "recipe is already favorited" {
			c.JSON(http.StatusConflict, gin.H{
				"error":   "Recipe already favorited",
				"message": "This recipe is already in your favorites",
			})
			return
		}
		
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error", 
			"message": "Failed to add favorite",
		})
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"message": "Recipe added to favorites successfully",
		"data":    favorite,
	})
}

// RemoveUserFavorite handles DELETE /api/v1/users/favorites/{recipeId}
func (ph *PreferenceHandler) RemoveUserFavorite(c *gin.Context) {
	userID, err := ph.getUserIDFromContext(c)
	if err != nil {
		return
	}

	// Parse recipe ID from URL parameter
	recipeIDStr := c.Param("recipeId")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid recipe ID",
			"message": "Recipe ID must be a valid UUID",
		})
		return
	}

	// Remove favorite
	err = ph.preferenceRepo.RemoveUserFavorite(userID, recipeID)
	if err != nil {
		if err.Error() == "favorite not found" {
			c.JSON(http.StatusNotFound, gin.H{
				"error":   "Favorite not found",
				"message": "This recipe is not in your favorites",
			})
			return
		}
		
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to remove favorite",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Recipe removed from favorites successfully",
	})
}

// GetUserWeeklyPatterns handles GET /api/v1/users/preferences/patterns
func (ph *PreferenceHandler) GetUserWeeklyPatterns(c *gin.Context) {
	userID, err := ph.getUserIDFromContext(c)
	if err != nil {
		return
	}

	// Get patterns from repository
	patterns, err := ph.preferenceRepo.GetUserWeeklyPatterns(userID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to retrieve weekly patterns",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": patterns,
	})
}

// UpdateUserWeeklyPatterns handles PUT /api/v1/users/preferences/patterns
func (ph *PreferenceHandler) UpdateUserWeeklyPatterns(c *gin.Context) {
	userID, err := ph.getUserIDFromContext(c)
	if err != nil {
		return
	}

	// Parse request body
	var patterns []models.UserWeeklyPattern
	if err := c.ShouldBindJSON(&patterns); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"message": err.Error(),
		})
		return
	}

	// Validate patterns
	for i, pattern := range patterns {
		if err := ph.preferenceRepo.ValidateWeeklyPattern(&pattern); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Validation error",
				"message": "Pattern " + strconv.Itoa(i+1) + ": " + err.Error(),
			})
			return
		}
	}

	// Update patterns
	updatedPatterns, err := ph.preferenceRepo.UpsertUserWeeklyPatterns(userID, patterns)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Internal server error",
			"message": "Failed to update weekly patterns",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Weekly patterns updated successfully",
		"data":    updatedPatterns,
	})
}

// getUserIDFromContext extracts and validates user ID from the Gin context
func (ph *PreferenceHandler) getUserIDFromContext(c *gin.Context) (uuid.UUID, error) {
	userIDValue, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error":   "Unauthorized",
			"message": "User ID not found in token",
		})
		return uuid.Nil, fmt.Errorf("user ID not found")
	}

	userID, ok := userIDValue.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid user ID",
			"message": "User ID format is invalid",
		})
		return uuid.Nil, fmt.Errorf("invalid user ID format")
	}

	return userID, nil
}

// Helper function to check if a string contains a substring
func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || (len(s) > len(substr) && 
		(s[:len(substr)] == substr || s[len(s)-len(substr):] == substr || 
		 containsHelper(s, substr))))
}

func containsHelper(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}