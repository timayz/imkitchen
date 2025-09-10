package handlers

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/services"
)

// CommunityRecipeHandlers handles HTTP requests for community recipes
type CommunityRecipeHandlers struct {
	communityService *services.CommunityRecipeService
}

// NewCommunityRecipeHandlers creates new community recipe handlers
func NewCommunityRecipeHandlers(communityService *services.CommunityRecipeService) *CommunityRecipeHandlers {
	return &CommunityRecipeHandlers{
		communityService: communityService,
	}
}

// GetCommunityRecipes handles GET /api/v1/community/recipes
func (h *CommunityRecipeHandlers) GetCommunityRecipes(c *gin.Context) {
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

	// Parse filters
	filters := &services.CommunityRecipeFilters{
		SortBy: c.DefaultQuery("sortBy", "rating"),
	}

	// Search query
	if searchQuery := c.Query("search"); searchQuery != "" {
		filters.SearchQuery = &searchQuery
	}

	// Min rating filter
	if minRatingStr := c.Query("minRating"); minRatingStr != "" {
		if minRating, err := strconv.ParseFloat(minRatingStr, 64); err == nil {
			filters.MinRating = &minRating
		}
	}

	// Max prep time filter
	if maxPrepTimeStr := c.Query("maxPrepTime"); maxPrepTimeStr != "" {
		if maxPrepTime, err := strconv.Atoi(maxPrepTimeStr); err == nil {
			filters.MaxPrepTime = &maxPrepTime
		}
	}

	// Meal types filter
	if mealTypesStr := c.Query("mealTypes"); mealTypesStr != "" {
		filters.MealTypes = parseCommaSeparated(mealTypesStr)
	}

	// Complexities filter
	if complexitiesStr := c.Query("complexities"); complexitiesStr != "" {
		filters.Complexities = parseCommaSeparated(complexitiesStr)
	}

	// Cuisine types filter
	if cuisineTypesStr := c.Query("cuisineTypes"); cuisineTypesStr != "" {
		filters.CuisineTypes = parseCommaSeparated(cuisineTypesStr)
	}

	// Eligible only filter
	if eligibleOnlyStr := c.Query("eligibleOnly"); eligibleOnlyStr == "true" {
		filters.EligibleOnly = true
	}

	// Get community recipes
	response, err := h.communityService.GetCommunityRecipes(filters, page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve community recipes",
		})
		return
	}

	c.JSON(http.StatusOK, response)
}

// GetTrendingRecipes handles GET /api/v1/community/recipes/trending
func (h *CommunityRecipeHandlers) GetTrendingRecipes(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit < 1 || limit > 50 {
		limit = 20
	}

	recipes, err := h.communityService.GetTrendingRecipes(limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve trending recipes",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"recipes": recipes,
		"count": len(recipes),
	})
}

// GetHighlyRatedRecipes handles GET /api/v1/community/recipes/highly-rated
func (h *CommunityRecipeHandlers) GetHighlyRatedRecipes(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit < 1 || limit > 50 {
		limit = 20
	}

	minRatings, _ := strconv.Atoi(c.DefaultQuery("minRatings", "3"))
	if minRatings < 1 {
		minRatings = 3
	}

	recipes, err := h.communityService.GetHighlyRatedRecipes(minRatings, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve highly rated recipes",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"recipes": recipes,
		"count": len(recipes),
		"criteria": gin.H{
			"minRatings": minRatings,
		},
	})
}

// GetRecommendedRecipes handles GET /api/v1/users/me/recommendations
func (h *CommunityRecipeHandlers) GetRecommendedRecipes(c *gin.Context) {
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

	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "10"))
	if limit < 1 || limit > 50 {
		limit = 10
	}

	recipes, err := h.communityService.GetRecommendedRecipesForUser(userID, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to retrieve personalized recommendations",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"recipes": recipes,
		"count": len(recipes),
		"personalized": true,
	})
}

// ImportExternalRecipe handles POST /api/v1/community/recipes/import
func (h *CommunityRecipeHandlers) ImportExternalRecipe(c *gin.Context) {
	var req services.RecipeImportRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid_request",
			"message": "Invalid request format",
			"details": err.Error(),
		})
		return
	}

	// Validate required fields
	if req.Source == "" || req.ExternalID == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "validation_error",
			"message": "Source and external ID are required",
		})
		return
	}

	// Validate source
	validSources := []string{"spoonacular", "edamam"}
	isValidSource := false
	for _, source := range validSources {
		if req.Source == source {
			isValidSource = true
			break
		}
	}

	if !isValidSource {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "validation_error",
			"message": "Invalid source. Supported sources: spoonacular, edamam",
		})
		return
	}

	// Import recipe
	recipe, err := h.communityService.ImportExternalRecipe(&req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "import_failed",
			"message": "Failed to import recipe from external source",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"recipe": recipe,
		"message": "Recipe imported successfully",
	})
}

// PromoteRecipeToPublic handles POST /api/v1/recipes/:id/promote
func (h *CommunityRecipeHandlers) PromoteRecipeToPublic(c *gin.Context) {
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

	// Parse request body for community flag
	var req struct {
		MakeCommunity bool `json:"makeCommunity"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		// Default to false if not specified
		req.MakeCommunity = false
	}

	// Promote recipe
	recipe, err := h.communityService.PromoteToPublic(recipeID, userID, req.MakeCommunity)
	if err != nil {
		switch err {
		case services.ErrRecipeNotFound:
			c.JSON(http.StatusNotFound, gin.H{
				"error": "recipe_not_found",
				"message": "Recipe not found",
			})
		case services.ErrUnauthorized:
			c.JSON(http.StatusForbidden, gin.H{
				"error": "access_denied",
				"message": "You can only promote your own recipes",
			})
		default:
			c.JSON(http.StatusInternalServerError, gin.H{
				"error": "internal_error",
				"message": "Failed to promote recipe",
			})
		}
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"recipe": recipe,
		"message": "Recipe promoted to public successfully",
	})
}

// SearchCommunityRecipes handles GET /api/v1/community/recipes/search
func (h *CommunityRecipeHandlers) SearchCommunityRecipes(c *gin.Context) {
	searchQuery := c.Query("q")
	if searchQuery == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "validation_error",
			"message": "Search query is required",
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

	// Build filters for search
	filters := &services.CommunityRecipeFilters{
		SearchQuery: &searchQuery,
		SortBy:     c.DefaultQuery("sortBy", "rating"),
	}

	// Add other filters if provided
	if minRatingStr := c.Query("minRating"); minRatingStr != "" {
		if minRating, err := strconv.ParseFloat(minRatingStr, 64); err == nil {
			filters.MinRating = &minRating
		}
	}

	// Get search results
	response, err := h.communityService.GetCommunityRecipes(filters, page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "internal_error",
			"message": "Failed to search community recipes",
		})
		return
	}

	c.JSON(http.StatusOK, response)
}

// RegisterCommunityRoutes registers all community recipe routes
func (h *CommunityRecipeHandlers) RegisterCommunityRoutes(router *gin.RouterGroup) {
	// Community recipe routes
	community := router.Group("/community")
	{
		recipes := community.Group("/recipes")
		{
			recipes.GET("", h.GetCommunityRecipes)
			recipes.GET("/search", h.SearchCommunityRecipes)
			recipes.GET("/trending", h.GetTrendingRecipes)
			recipes.GET("/highly-rated", h.GetHighlyRatedRecipes)
			recipes.POST("/import", h.ImportExternalRecipe)
		}
	}

	// User-specific routes
	users := router.Group("/users/me")
	{
		users.GET("/recommendations", h.GetRecommendedRecipes)
	}

	// Recipe management routes
	recipes := router.Group("/recipes")
	{
		recipes.POST("/:id/promote", h.PromoteRecipeToPublic)
	}
}

// Helper functions

// parseCommaSeparated parses comma-separated string into slice
func parseCommaSeparated(s string) []string {
	if s == "" {
		return nil
	}
	
	var result []string
	items := strings.Split(s, ",")
	for _, item := range items {
		trimmed := strings.TrimSpace(item)
		if trimmed != "" {
			result = append(result, trimmed)
		}
	}
	
	return result
}

// validateSortOption validates sort option
func validateSortOption(sortBy string) string {
	validOptions := []string{"rating", "recent", "popular", "trending"}
	for _, option := range validOptions {
		if sortBy == option {
			return sortBy
		}
	}
	return "rating" // Default
}