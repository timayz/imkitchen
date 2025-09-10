package handlers

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

type RecipeHandler struct {
	recipeService services.RecipeService
}

func NewRecipeHandler(recipeService services.RecipeService) *RecipeHandler {
	return &RecipeHandler{
		recipeService: recipeService,
	}
}

// CreateRecipe handles POST /api/recipes
func (h *RecipeHandler) CreateRecipe(c *gin.Context) {
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

	var input models.CreateRecipeInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	recipe, err := h.recipeService.CreateRecipe(userID, &input)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "validation error" {
			statusCode = http.StatusBadRequest
		}
		
		c.JSON(statusCode, gin.H{
			"error":   "Failed to create recipe",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusCreated, recipe)
}

// GetRecipe handles GET /api/recipes/:id
func (h *RecipeHandler) GetRecipe(c *gin.Context) {
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

	idParam := c.Param("id")
	id, err := uuid.Parse(idParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid recipe ID format",
		})
		return
	}

	recipe, err := h.recipeService.GetRecipe(id, userID)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		
		c.JSON(statusCode, gin.H{
			"error":   "Failed to get recipe",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, recipe)
}

// GetUserRecipes handles GET /api/recipes/my
func (h *RecipeHandler) GetUserRecipes(c *gin.Context) {
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

	// Get pagination parameters
	limit := 20 // default
	offset := 0 // default
	
	if limitStr := c.Query("limit"); limitStr != "" {
		if parsedLimit, err := strconv.Atoi(limitStr); err == nil && parsedLimit > 0 && parsedLimit <= 100 {
			limit = parsedLimit
		}
	}
	
	if offsetStr := c.Query("offset"); offsetStr != "" {
		if parsedOffset, err := strconv.Atoi(offsetStr); err == nil && parsedOffset >= 0 {
			offset = parsedOffset
		}
	}

	recipes, err := h.recipeService.GetUserRecipes(userID, limit, offset)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to get user recipes",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"recipes": recipes,
		"limit":   limit,
		"offset":  offset,
		"total":   len(recipes),
	})
}

// UpdateRecipe handles PUT /api/recipes/:id
func (h *RecipeHandler) UpdateRecipe(c *gin.Context) {
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

	idParam := c.Param("id")
	id, err := uuid.Parse(idParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid recipe ID format",
		})
		return
	}

	var input models.UpdateRecipeInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	recipe, err := h.recipeService.UpdateRecipe(id, userID, &input)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "validation error" {
			statusCode = http.StatusBadRequest
		} else if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		
		c.JSON(statusCode, gin.H{
			"error":   "Failed to update recipe",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, recipe)
}

// DeleteRecipe handles DELETE /api/recipes/:id
func (h *RecipeHandler) DeleteRecipe(c *gin.Context) {
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

	idParam := c.Param("id")
	id, err := uuid.Parse(idParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid recipe ID format",
		})
		return
	}

	if err := h.recipeService.DeleteRecipe(id, userID); err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		
		c.JSON(statusCode, gin.H{
			"error":   "Failed to delete recipe",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Recipe deleted successfully",
	})
}

// SearchRecipes handles GET /api/recipes
func (h *RecipeHandler) SearchRecipes(c *gin.Context) {
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

	var params models.RecipeSearchParams
	
	// Bind query parameters
	if err := c.ShouldBindQuery(&params); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid query parameters",
			"details": err.Error(),
		})
		return
	}

	// Set defaults if not provided
	if params.Page == 0 {
		params.Page = 1
	}
	if params.Limit == 0 {
		params.Limit = 20
	}
	if params.SortBy == "" {
		params.SortBy = "created_at"
	}
	if params.SortOrder == "" {
		params.SortOrder = "desc"
	}

	result, err := h.recipeService.SearchRecipes(userID, &params)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to search recipes",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, result)
}

// ImportRecipe handles POST /api/recipes/import
func (h *RecipeHandler) ImportRecipe(c *gin.Context) {
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

	var input models.ImportRecipeInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	result, err := h.recipeService.ImportRecipe(userID, &input)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to process import request",
			"details": err.Error(),
		})
		return
	}

	if !result.Success {
		c.JSON(http.StatusBadRequest, result)
		return
	}

	c.JSON(http.StatusCreated, result)
}

// RegisterRecipeRoutes registers all recipe-related routes
func RegisterRecipeRoutes(router *gin.RouterGroup, recipeHandler *RecipeHandler) {
	recipes := router.Group("/recipes")
	{
		recipes.POST("", recipeHandler.CreateRecipe)
		recipes.GET("", recipeHandler.SearchRecipes)
		recipes.GET("/my", recipeHandler.GetUserRecipes)
		recipes.GET("/:id", recipeHandler.GetRecipe)
		recipes.PUT("/:id", recipeHandler.UpdateRecipe)
		recipes.DELETE("/:id", recipeHandler.DeleteRecipe)
		recipes.POST("/import", recipeHandler.ImportRecipe)
	}
}