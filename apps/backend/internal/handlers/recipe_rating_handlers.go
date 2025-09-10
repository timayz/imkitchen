package handlers

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/services"
)

// RecipeRatingHandlers handles HTTP requests for recipe ratings
type RecipeRatingHandlers struct {
	ratingService *services.RecipeRatingService
}

// NewRecipeRatingHandlers creates new recipe rating handlers
func NewRecipeRatingHandlers(ratingService *services.RecipeRatingService) *RecipeRatingHandlers {
	return &RecipeRatingHandlers{
		ratingService: ratingService,
	}
}

// SubmitRating handles POST /api/v1/recipes/:id/rating
func (h *RecipeRatingHandlers) SubmitRating(c *gin.Context) {
	// Get user ID from auth middleware
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "authentication_required",
			"message": "Authentication required",
		})
		return
	}

	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Invalid user context",
		})
		return
	}

	// Get recipe ID from URL
	recipeIDStr := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_recipe_id",
			"message": "Invalid recipe ID format",
		})
		return
	}

	// Parse request body
	var req services.RatingSubmissionRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_request",
			"message": "Invalid request format",
			"details": err.Error(),
		})
		return
	}

	// Set recipe ID from URL parameter
	req.RecipeID = recipeID

	// Submit rating
	rating, err := h.ratingService.SubmitRating(userID, &req)
	if err != nil {
		switch err {
		case services.ErrDuplicateRating:
			c.JSON(http.StatusConflict, gin.H{
				"error": "duplicate_rating",
				"message": "You have already rated this recipe. Update your existing rating instead.",
			})
		case services.ErrRecipeNotFound:
			c.JSON(http.StatusNotFound, gin.H{
				"error": "recipe_not_found",
				"message": "Recipe not available for rating",
			})
		case services.ErrInvalidRating:
			c.JSON(http.StatusBadRequest, gin.H{
				"error": "invalid_rating",
				"message": "Rating must be between 1 and 5 stars",
			})
		default:
			c.JSON(http.StatusInternalServerError, gin.H{
				"error": "internal_error",
				"message": "Failed to submit rating",
			})
		}
		return
	}

	// Return success response
	c.JSON(http.StatusCreated, gin.H{
		"rating": rating,
		"message": "Rating submitted successfully",
	})
}

// GetRecipeRatings handles GET /api/v1/recipes/:id/ratings
func (h *RecipeRatingHandlers) GetRecipeRatings(c *gin.Context) {
	// Get recipe ID from URL
	recipeIDStr := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_recipe_id",
			"message": "Invalid recipe ID format",
		})
		return
	}

	// Parse pagination parameters
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))

	// Validate pagination
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 100 {
		limit = 20
	}

	// Get ratings
	response, err := h.ratingService.GetRatingsByRecipe(recipeID, page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve ratings",
		})
		return
	}

	c.JSON(http.StatusOK, response)
}

// UpdateRating handles PUT /api/v1/recipes/:id/rating
func (h *RecipeRatingHandlers) UpdateRating(c *gin.Context) {
	// Get user ID from auth middleware
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "authentication_required",
			"message": "Authentication required",
		})
		return
	}

	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Invalid user context",
		})
		return
	}

	// Get recipe ID from URL
	recipeIDStr := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_recipe_id",
			"message": "Invalid recipe ID format",
		})
		return
	}

	// Parse request body
	var req services.RatingUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_request",
			"message": "Invalid request format",
			"details": err.Error(),
		})
		return
	}

	// Update rating
	rating, err := h.ratingService.UpdateRating(userID, recipeID, &req)
	if err != nil {
		switch err {
		case services.ErrRatingNotFound:
			c.JSON(http.StatusNotFound, gin.H{
				"error": "rating_not_found",
				"message": "Rating not found or you haven't rated this recipe yet",
			})
		case services.ErrInvalidRating:
			c.JSON(http.StatusBadRequest, gin.H{
				"error": "invalid_rating",
				"message": "Rating must be between 1 and 5 stars",
			})
		default:
			c.JSON(http.StatusInternalServerError, gin.H{
				"error": "internal_error",
				"message": "Failed to update rating",
			})
		}
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"rating": rating,
		"message": "Rating updated successfully",
	})
}

// DeleteRating handles DELETE /api/v1/recipes/:id/rating
func (h *RecipeRatingHandlers) DeleteRating(c *gin.Context) {
	// Get user ID from auth middleware
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "authentication_required",
			"message": "Authentication required",
		})
		return
	}

	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Invalid user context",
		})
		return
	}

	// Get recipe ID from URL
	recipeIDStr := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_recipe_id",
			"message": "Invalid recipe ID format",
		})
		return
	}

	// Delete rating
	err = h.ratingService.DeleteRating(userID, recipeID)
	if err != nil {
		switch err {
		case services.ErrRatingNotFound:
			c.JSON(http.StatusNotFound, gin.H{
				"error": "rating_not_found",
				"message": "Rating not found",
			})
		default:
			c.JSON(http.StatusInternalServerError, gin.H{
				"error": "internal_error",
				"message": "Failed to delete rating",
			})
		}
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Rating deleted successfully",
	})
}

// GetUserRatings handles GET /api/v1/users/me/ratings
func (h *RecipeRatingHandlers) GetUserRatings(c *gin.Context) {
	// Get user ID from auth middleware
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "authentication_required",
			"message": "Authentication required",
		})
		return
	}

	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Invalid user context",
		})
		return
	}

	// Parse pagination parameters
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))

	// Validate pagination
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 100 {
		limit = 20
	}

	// Get user's rating history
	response, err := h.ratingService.GetUserRatingHistory(userID, page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve rating history",
		})
		return
	}

	c.JSON(http.StatusOK, response)
}

// GetUserRatingForRecipe handles GET /api/v1/recipes/:id/rating/me
func (h *RecipeRatingHandlers) GetUserRatingForRecipe(c *gin.Context) {
	// Get user ID from auth middleware
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{
			"error": "authentication_required",
			"message": "Authentication required",
		})
		return
	}

	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Invalid user context",
		})
		return
	}

	// Get recipe ID from URL
	recipeIDStr := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_recipe_id",
			"message": "Invalid recipe ID format",
		})
		return
	}

	// Get user's rating for this recipe
	rating, err := h.ratingService.GetUserRatingForRecipe(userID, recipeID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve rating",
		})
		return
	}

	if rating == nil {
		c.JSON(http.StatusNotFound, gin.H{
			"error": "rating_not_found",
			"message": "You haven't rated this recipe yet",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"rating": rating,
	})
}

// FlagRating handles POST /api/v1/ratings/:id/flag
func (h *RecipeRatingHandlers) FlagRating(c *gin.Context) {
	// Get rating ID from URL
	ratingIDStr := c.Param("id")
	ratingID, err := uuid.Parse(ratingIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_rating_id",
			"message": "Invalid rating ID format",
		})
		return
	}

	// Parse request body for flagging reason
	var req struct {
		Reason string `json:"reason" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_request",
			"message": "Flagging reason is required",
		})
		return
	}

	// Flag rating
	err = h.ratingService.FlagRating(ratingID, req.Reason)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to flag rating",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Rating flagged for review",
	})
}

// GetPendingModerationRatings handles GET /api/v1/admin/ratings/pending (admin only)
func (h *RecipeRatingHandlers) GetPendingModerationRatings(c *gin.Context) {
	// Parse pagination parameters
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "50"))

	// Validate pagination
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 100 {
		limit = 50
	}

	// Get pending ratings
	response, err := h.ratingService.GetPendingModerationRatings(page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve pending ratings",
		})
		return
	}

	c.JSON(http.StatusOK, response)
}

// RegisterRatingRoutes registers all rating-related routes
func (h *RecipeRatingHandlers) RegisterRatingRoutes(router *gin.RouterGroup, rateLimiter *middleware.RatingRateLimiter) {
	// Recipe rating routes
	recipeRatings := router.Group("/recipes/:id")
	{
		recipeRatings.POST("/rating", rateLimiter.RatingRateLimit(), middleware.RatingValidationMiddleware(), h.SubmitRating)
		recipeRatings.GET("/ratings", h.GetRecipeRatings)
		recipeRatings.PUT("/rating", rateLimiter.RatingRateLimit(), middleware.RatingValidationMiddleware(), h.UpdateRating)
		recipeRatings.DELETE("/rating", h.DeleteRating)
		recipeRatings.GET("/rating/me", h.GetUserRatingForRecipe)
	}

	// User rating routes
	userRatings := router.Group("/users/me")
	{
		userRatings.GET("/ratings", h.GetUserRatings)
	}

	// Rating moderation routes
	ratings := router.Group("/ratings")
	{
		ratings.POST("/:id/flag", h.FlagRating)
	}

	// Admin routes
	admin := router.Group("/admin")
	admin.Use(middleware.AdminOnlyMiddleware())
	{
		admin.GET("/ratings/pending", h.GetPendingModerationRatings)
	}
}