package handlers

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// ShoppingListHandlers handles HTTP requests for shopping lists
type ShoppingListHandlers struct {
	service *services.ShoppingListService
}

// NewShoppingListHandlers creates new shopping list handlers
func NewShoppingListHandlers(service *services.ShoppingListService) *ShoppingListHandlers {
	return &ShoppingListHandlers{
		service: service,
	}
}

// GenerateShoppingList generates a shopping list from a meal plan
// POST /api/v1/shopping-lists/generate
func (h *ShoppingListHandlers) GenerateShoppingList(c *gin.Context) {
	userID, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	var req models.ShoppingListGenerateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	mealPlanID, err := uuid.Parse(req.MealPlanID)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid meal plan ID"})
		return
	}

	shoppingList, err := h.service.GenerateFromMealPlan(userUUID, mealPlanID, req.MergeExisting)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to generate shopping list", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, shoppingList)
}

// GetShoppingLists retrieves all shopping lists for the authenticated user
// GET /api/v1/shopping-lists
func (h *ShoppingListHandlers) GetShoppingLists(c *gin.Context) {
	userID, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	// Get query parameters
	status := c.Query("status")
	sortBy := c.DefaultQuery("sortBy", "created")

	shoppingLists, err := h.service.GetUserShoppingLists(userUUID, status, sortBy)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve shopping lists", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"shoppingLists": shoppingLists})
}

// GetShoppingList retrieves a specific shopping list
// GET /api/v1/shopping-lists/:id
func (h *ShoppingListHandlers) GetShoppingList(c *gin.Context) {
	userID, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	listID, err := uuid.Parse(c.Param("id"))
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid shopping list ID"})
		return
	}

	shoppingList, err := h.service.GetShoppingList(userUUID, listID)
	if err != nil {
		if err.Error() == "shopping list not found" {
			c.JSON(http.StatusNotFound, gin.H{"error": "Shopping list not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve shopping list", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, shoppingList)
}

// UpdateShoppingItem updates a shopping list item
// PUT /api/v1/shopping-lists/:id/items/:itemId
func (h *ShoppingListHandlers) UpdateShoppingItem(c *gin.Context) {
	userID, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	listID, err := uuid.Parse(c.Param("id"))
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid shopping list ID"})
		return
	}

	itemID, err := uuid.Parse(c.Param("itemId"))
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid item ID"})
		return
	}

	var req models.ShoppingItemUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	err = h.service.UpdateItem(userUUID, listID, itemID, &req)
	if err != nil {
		if err.Error() == "shopping list not found or access denied" {
			c.JSON(http.StatusNotFound, gin.H{"error": "Shopping list not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update item", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Item updated successfully"})
}

// ExportShoppingList exports a shopping list in the specified format
// GET /api/v1/shopping-lists/:id/export
func (h *ShoppingListHandlers) ExportShoppingList(c *gin.Context) {
	userID, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	listID, err := uuid.Parse(c.Param("id"))
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid shopping list ID"})
		return
	}

	format := c.DefaultQuery("format", "json")
	includeRecipeSourcesStr := c.DefaultQuery("includeRecipeSources", "false")
	includeRecipeSources, _ := strconv.ParseBool(includeRecipeSourcesStr)

	data, filename, err := h.service.ExportShoppingList(userUUID, listID, format, includeRecipeSources)
	if err != nil {
		if err.Error() == "shopping list not found" {
			c.JSON(http.StatusNotFound, gin.H{"error": "Shopping list not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to export shopping list", "details": err.Error()})
		return
	}

	// Set appropriate content type and headers
	var contentType string
	switch format {
	case "csv":
		contentType = "text/csv"
	case "txt":
		contentType = "text/plain"
	default:
		contentType = "application/json"
	}

	c.Header("Content-Type", contentType)
	c.Header("Content-Disposition", "attachment; filename="+filename)
	c.Data(http.StatusOK, contentType, data)
}

// DeleteShoppingList deletes a shopping list
// DELETE /api/v1/shopping-lists/:id
func (h *ShoppingListHandlers) DeleteShoppingList(c *gin.Context) {
	userID, exists := c.Get("userID")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userUUID, ok := userID.(uuid.UUID)
	if !ok {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid user ID format"})
		return
	}

	listID, err := uuid.Parse(c.Param("id"))
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid shopping list ID"})
		return
	}

	err = h.service.DeleteShoppingList(userUUID, listID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to delete shopping list", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Shopping list deleted successfully"})
}

// RegisterShoppingListRoutes registers all shopping list related routes
func RegisterShoppingListRoutes(router *gin.RouterGroup, handler *ShoppingListHandlers) {
	shoppingLists := router.Group("/shopping-lists")
	{
		shoppingLists.POST("/generate", handler.GenerateShoppingList)
		shoppingLists.GET("", handler.GetShoppingLists)
		shoppingLists.GET("/:id", handler.GetShoppingList)
		shoppingLists.PUT("/:id/items/:itemId", handler.UpdateShoppingItem)
		shoppingLists.GET("/:id/export", handler.ExportShoppingList)
		shoppingLists.DELETE("/:id", handler.DeleteShoppingList)
	}
}