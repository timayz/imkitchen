package handlers

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

type RecipeImportHandler struct {
	communityImportService services.CommunityImportService
}

func NewRecipeImportHandler(communityImportService services.CommunityImportService) *RecipeImportHandler {
	return &RecipeImportHandler{
		communityImportService: communityImportService,
	}
}

// ImportCommunityRecipe handles POST /api/v1/recipes/import
func (h *RecipeImportHandler) ImportCommunityRecipe(c *gin.Context) {
	// Get user ID from context (set by auth middleware)
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}
	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	var input models.RecipeImportRequest
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	// Validate required fields
	if input.CommunityRecipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "communityRecipeId is required",
		})
		return
	}

	// Parse community recipe ID as UUID
	communityRecipeUUID, err := uuid.Parse(input.CommunityRecipeID)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid community recipe ID format",
		})
		return
	}

	result, err := h.communityImportService.ImportCommunityRecipe(userID, communityRecipeUUID, &input)
	if err != nil {
		// Determine appropriate status code based on error
		statusCode := http.StatusInternalServerError
		errorMessage := "Failed to import recipe"

		switch err.Error() {
		case "community recipe not found":
			statusCode = http.StatusNotFound
			errorMessage = "Community recipe not found"
		case "recipe already imported":
			statusCode = http.StatusConflict
			errorMessage = "Recipe already exists in your collection"
		case "import rate limit exceeded":
			statusCode = http.StatusTooManyRequests
			errorMessage = "Import rate limit exceeded. Please try again later"
		case "validation error":
			statusCode = http.StatusBadRequest
			errorMessage = "Invalid import parameters"
		default:
			// Log the actual error for debugging
			c.Header("X-Error-Context", err.Error())
		}

		c.JSON(statusCode, gin.H{
			"error":   errorMessage,
			"success": false,
		})
		return
	}

	c.JSON(http.StatusCreated, result)
}

// GetImportHistory handles GET /api/v1/recipes/import/history
func (h *RecipeImportHandler) GetImportHistory(c *gin.Context) {
	// Get user ID from context
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}
	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	// Parse query parameters
	page := 1
	limit := 20
	
	if pageParam := c.Query("page"); pageParam != "" {
		if parsedPage, err := parseIntParam(pageParam, 1, 100); err == nil {
			page = parsedPage
		}
	}
	
	if limitParam := c.Query("limit"); limitParam != "" {
		if parsedLimit, err := parseIntParam(limitParam, 1, 100); err == nil {
			limit = parsedLimit
		}
	}

	history, total, err := h.communityImportService.GetImportHistory(userID, page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "Failed to fetch import history",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"imports": history,
		"pagination": gin.H{
			"page":       page,
			"limit":      limit,
			"total":      total,
			"hasNext":    page*limit < total,
			"hasPrevious": page > 1,
		},
	})
}

// CheckImportConflict handles GET /api/v1/recipes/import/check/:communityRecipeId
func (h *RecipeImportHandler) CheckImportConflict(c *gin.Context) {
	// Get user ID from context
	userIDInterface, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}
	userID, ok := userIDInterface.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	// Parse community recipe ID from path parameter
	communityRecipeIDParam := c.Param("communityRecipeId")
	communityRecipeID, err := uuid.Parse(communityRecipeIDParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid community recipe ID format",
		})
		return
	}

	conflict, err := h.communityImportService.CheckImportConflict(userID, communityRecipeID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "Failed to check import conflict",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"hasConflict": conflict != nil,
		"conflict":    conflict,
	})
}

// Helper function to parse integer parameters with bounds
func parseIntParam(param string, min, max int) (int, error) {
	value, err := strconv.Atoi(param)
	if err != nil {
		return 0, err
	}
	if value < min {
		value = min
	}
	if value > max {
		value = max
	}
	return value, nil
}