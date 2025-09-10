package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

type PhotoHandler struct {
	photoService  services.PhotoService
	recipeService services.RecipeService
}

type PhotoUploadResponse struct {
	URL string `json:"url"`
}

func NewPhotoHandler(photoService services.PhotoService, recipeService services.RecipeService) *PhotoHandler {
	return &PhotoHandler{
		photoService:  photoService,
		recipeService: recipeService,
	}
}

// UploadRecipePhoto handles POST /api/recipes/:id/photo
func (h *PhotoHandler) UploadRecipePhoto(c *gin.Context) {
	// Get recipe ID from URL
	recipeIDParam := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid recipe ID format",
		})
		return
	}

	// Check if recipe exists
	recipe, err := h.recipeService.GetRecipe(recipeID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{
			"error": "Recipe not found",
		})
		return
	}

	// Parse multipart form
	err = c.Request.ParseMultipartForm(10 << 20) // 10 MB max
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Failed to parse form data",
			"details": err.Error(),
		})
		return
	}

	// Get file from form
	file, header, err := c.Request.FormFile("photo")
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "No photo file provided",
			"details": "Expected form field 'photo' with image file",
		})
		return
	}
	defer file.Close()

	// Upload photo
	photoURL, err := h.photoService.UploadRecipePhoto(file, header, recipeID.String())
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "invalid file type" || err.Error() == "file too large" {
			statusCode = http.StatusBadRequest
		}
		
		c.JSON(statusCode, gin.H{
			"error":   "Failed to upload photo",
			"details": err.Error(),
		})
		return
	}

	// Delete old photo if exists
	if recipe.ImageURL != nil && *recipe.ImageURL != "" {
		// Don't fail the request if old photo deletion fails, just log it
		if err := h.photoService.DeleteRecipePhoto(*recipe.ImageURL); err != nil {
			// Log error but continue
			c.Header("X-Warning", "Failed to delete old photo")
		}
	}

	// Update recipe with new photo URL
	updateInput := &models.UpdateRecipeInput{
		ImageURL: &photoURL,
	}

	updatedRecipe, err := h.recipeService.UpdateRecipe(recipeID, updateInput)
	if err != nil {
		// Photo was uploaded but recipe update failed
		// Try to clean up uploaded photo
		h.photoService.DeleteRecipePhoto(photoURL)
		
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Photo uploaded but failed to update recipe",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, PhotoUploadResponse{
		URL: photoURL,
	})
}

// DeleteRecipePhoto handles DELETE /api/recipes/:id/photo
func (h *PhotoHandler) DeleteRecipePhoto(c *gin.Context) {
	// Get recipe ID from URL
	recipeIDParam := c.Param("id")
	recipeID, err := uuid.Parse(recipeIDParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid recipe ID format",
		})
		return
	}

	// Get recipe to check if it has a photo
	recipe, err := h.recipeService.GetRecipe(recipeID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{
			"error": "Recipe not found",
		})
		return
	}

	if recipe.ImageURL == nil || *recipe.ImageURL == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Recipe has no photo to delete",
		})
		return
	}

	// Delete photo from storage
	err = h.photoService.DeleteRecipePhoto(*recipe.ImageURL)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to delete photo from storage",
			"details": err.Error(),
		})
		return
	}

	// Update recipe to remove photo URL
	updateInput := &models.UpdateRecipeInput{
		ImageURL: stringPtr(""), // Set to empty string to clear the field
	}

	_, err = h.recipeService.UpdateRecipe(recipeID, updateInput)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Photo deleted but failed to update recipe",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Photo deleted successfully",
	})
}

// RegisterPhotoRoutes registers all photo-related routes
func RegisterPhotoRoutes(router *gin.RouterGroup, photoHandler *PhotoHandler) {
	photos := router.Group("/recipes/:id/photo")
	{
		photos.POST("", photoHandler.UploadRecipePhoto)
		photos.DELETE("", photoHandler.DeleteRecipePhoto)
	}
}

// Helper function to create string pointer
func stringPtr(s string) *string {
	return &s
}