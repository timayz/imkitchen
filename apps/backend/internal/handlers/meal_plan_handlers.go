package handlers

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
)

type MealPlanHandler struct {
	mealPlanService services.MealPlanService
	rotationService services.RotationService
	userRepo        repositories.UserRepository
}

func NewMealPlanHandler(mealPlanService services.MealPlanService, rotationService services.RotationService, userRepo repositories.UserRepository) *MealPlanHandler {
	return &MealPlanHandler{
		mealPlanService: mealPlanService,
		rotationService: rotationService,
		userRepo:        userRepo,
	}
}

// CreateMealPlan handles POST /api/meal-plans
func (h *MealPlanHandler) CreateMealPlan(c *gin.Context) {
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

	var input models.CreateMealPlanInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	mealPlan, err := h.mealPlanService.CreateMealPlan(userID, &input)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "validation error" {
			statusCode = http.StatusBadRequest
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to create meal plan",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusCreated, mealPlan)
}

// GetMealPlans handles GET /api/meal-plans
func (h *MealPlanHandler) GetMealPlans(c *gin.Context) {
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

	// Parse query filters
	var filters models.MealPlanFilters
	if err := c.ShouldBindQuery(&filters); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid query parameters",
			"details": err.Error(),
		})
		return
	}

	mealPlans, err := h.mealPlanService.GetUserMealPlans(userID, &filters)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to get meal plans",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"mealPlans": mealPlans,
		"count":     len(mealPlans),
	})
}

// GetMealPlan handles GET /api/meal-plans/:id
func (h *MealPlanHandler) GetMealPlan(c *gin.Context) {
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

	// Parse meal plan ID
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid meal plan ID",
			"details": err.Error(),
		})
		return
	}

	mealPlan, err := h.mealPlanService.GetMealPlan(id, userID)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to get meal plan",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, mealPlan)
}

// GetMealPlanByWeek handles GET /api/meal-plans/week/:date
func (h *MealPlanHandler) GetMealPlanByWeek(c *gin.Context) {
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

	// Parse week start date
	dateStr := c.Param("date")
	weekStart, err := time.Parse("2006-01-02", dateStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid date format, expected YYYY-MM-DD",
			"details": err.Error(),
		})
		return
	}

	mealPlan, err := h.mealPlanService.GetMealPlanByWeek(userID, weekStart)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to get meal plan for week",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, mealPlan)
}

// UpdateMealPlan handles PUT /api/meal-plans/:id
func (h *MealPlanHandler) UpdateMealPlan(c *gin.Context) {
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

	// Parse meal plan ID
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid meal plan ID",
			"details": err.Error(),
		})
		return
	}

	var input models.UpdateMealPlanInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	mealPlan, err := h.mealPlanService.UpdateMealPlan(id, userID, &input)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		} else if err.Error() == "validation error" {
			statusCode = http.StatusBadRequest
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to update meal plan",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, mealPlan)
}

// UpdateMealSlot handles PUT /api/meal-plans/:id/entries/:day/:mealType
func (h *MealPlanHandler) UpdateMealSlot(c *gin.Context) {
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

	// Parse meal plan ID
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid meal plan ID",
			"details": err.Error(),
		})
		return
	}

	// Get day and meal type from URL params
	day := c.Param("day")
	mealType := c.Param("mealType")

	var input models.UpdateMealSlotInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	mealPlan, err := h.mealPlanService.UpdateMealSlot(id, userID, day, mealType, &input)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		} else if err.Error() == "validation error" {
			statusCode = http.StatusBadRequest
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to update meal slot",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, mealPlan)
}

// DeleteMealSlot handles DELETE /api/meal-plans/:id/entries/:day/:mealType
func (h *MealPlanHandler) DeleteMealSlot(c *gin.Context) {
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

	// Parse meal plan ID
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid meal plan ID",
			"details": err.Error(),
		})
		return
	}

	// Get day and meal type from URL params
	day := c.Param("day")
	mealType := c.Param("mealType")

	mealPlan, err := h.mealPlanService.DeleteMealSlot(id, userID, day, mealType)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to delete meal slot",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, mealPlan)
}

// DeleteMealPlan handles DELETE /api/meal-plans/:id
func (h *MealPlanHandler) DeleteMealPlan(c *gin.Context) {
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

	// Parse meal plan ID
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid meal plan ID",
			"details": err.Error(),
		})
		return
	}

	err = h.mealPlanService.DeleteMealPlan(id, userID)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "record not found" {
			statusCode = http.StatusNotFound
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to delete meal plan",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusNoContent, nil)
}

// GenerateMealPlanInput represents input for "Fill My Week" meal plan generation
type GenerateMealPlanInput struct {
	WeekStartDate           *time.Time `json:"weekStartDate,omitempty"`
	MaxPrepTimePerMeal      *int       `json:"maxPrepTimePerMeal,omitempty" validate:"omitempty,min=15,max=180"`
	PreferredComplexityLevel *string    `json:"preferredComplexityLevel,omitempty" validate:"omitempty,oneof=simple moderate complex"`
	AvoidRecipeIDs          []string   `json:"avoidRecipeIDs,omitempty"`
	CuisinePreferences      []string   `json:"cuisinePreferences,omitempty"`
}

// MealPlanGenerationResponse represents the response from meal plan generation
type MealPlanGenerationResponse struct {
	MealPlan        *models.MealPlanResponse `json:"mealPlan"`
	GenerationTime  int64                    `json:"generationTimeMs"`
	VarietyScore    float64                  `json:"varietyScore"`
	RecipesUsed     int                      `json:"recipesUsed"`
	RotationCycle   int                      `json:"rotationCycle"`
	Warnings        []string                 `json:"warnings,omitempty"`
}

// GenerateWeeklyMealPlan handles POST /api/meal-plans/generate - "Fill My Week" functionality
func (h *MealPlanHandler) GenerateWeeklyMealPlan(c *gin.Context) {
	startTime := time.Now()
	
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

	// Parse request body
	var input GenerateMealPlanInput
	if err := c.ShouldBindJSON(&input); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "Invalid request body",
			"details": err.Error(),
		})
		return
	}

	// Set default week start date to upcoming Monday if not provided
	weekStartDate := time.Now()
	if input.WeekStartDate != nil {
		weekStartDate = *input.WeekStartDate
	}
	
	// Adjust to Monday of the week
	for weekStartDate.Weekday() != time.Monday {
		if weekStartDate.Weekday() == time.Sunday {
			weekStartDate = weekStartDate.AddDate(0, 0, 1)
		} else {
			weekStartDate = weekStartDate.AddDate(0, 0, -1)
		}
	}
	weekStartDate = time.Date(weekStartDate.Year(), weekStartDate.Month(), weekStartDate.Day(), 0, 0, 0, 0, weekStartDate.Location())

	// Get user preferences
	userPrefs, err := h.userRepo.GetUserPreferences(userID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to get user preferences",
			"details": err.Error(),
		})
		return
	}

	// Override preferences with request inputs
	if input.MaxPrepTimePerMeal != nil {
		userPrefs.MaxPrepTimePerMeal = *input.MaxPrepTimePerMeal
	}
	if input.PreferredComplexityLevel != nil {
		userPrefs.PreferredMealComplexity = *input.PreferredComplexityLevel
	}
	if len(input.CuisinePreferences) > 0 {
		userPrefs.CuisinePreferences = input.CuisinePreferences
	}

	// Generate intelligent weekly meal plan using rotation service
	weeklyMeals, err := h.rotationService.SelectRecipesForWeek(userID, userPrefs)
	if err != nil {
		statusCode := http.StatusInternalServerError
		if err.Error() == "insufficient recipes available" {
			statusCode = http.StatusBadRequest
		}
		c.JSON(statusCode, gin.H{
			"error":   "Failed to generate meal plan",
			"details": err.Error(),
		})
		return
	}

	// Create meal plan in database
	createInput := &models.CreateMealPlanInput{
		WeekStartDate:  weekStartDate,
		GenerationType: "automated",
		Meals:          *weeklyMeals,
	}

	mealPlan, err := h.mealPlanService.CreateMealPlan(userID, createInput)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to save generated meal plan",
			"details": err.Error(),
		})
		return
	}

	// Get the populated meal plan response
	mealPlanResponse, err := h.mealPlanService.GetMealPlan(mealPlan.ID, userID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "Failed to get populated meal plan",
			"details": err.Error(),
		})
		return
	}

	// Calculate variety score and gather statistics
	recipeIDs := make([]string, 0)
	for _, dayMeals := range [][]models.MealSlotWithRecipe{
		mealPlanResponse.PopulatedMeals.Monday,
		mealPlanResponse.PopulatedMeals.Tuesday,
		mealPlanResponse.PopulatedMeals.Wednesday,
		mealPlanResponse.PopulatedMeals.Thursday,
		mealPlanResponse.PopulatedMeals.Friday,
		mealPlanResponse.PopulatedMeals.Saturday,
		mealPlanResponse.PopulatedMeals.Sunday,
	} {
		for _, meal := range dayMeals {
			if meal.Recipe != nil {
				recipeIDs = append(recipeIDs, meal.Recipe.ID.String())
			}
		}
	}

	varietyScore, _ := h.rotationService.GetVarietyScore(recipeIDs, userID)

	// Get rotation state for cycle info
	rotationState, _ := h.rotationService.GetRotationState(userID)
	rotationCycle := 0
	if rotationState != nil {
		rotationCycle = rotationState.CycleCount
	}

	// Calculate generation time
	generationTime := time.Since(startTime).Milliseconds()

	// Prepare warnings
	warnings := make([]string, 0)
	if generationTime > 2000 {
		warnings = append(warnings, "Generation took longer than expected (>2 seconds)")
	}
	if varietyScore < 0.5 {
		warnings = append(warnings, "Limited recipe variety - consider adding more recipes to your collection")
	}

	response := &MealPlanGenerationResponse{
		MealPlan:       mealPlanResponse,
		GenerationTime: generationTime,
		VarietyScore:   varietyScore,
		RecipesUsed:    len(recipeIDs),
		RotationCycle:  rotationCycle,
		Warnings:       warnings,
	}

	c.JSON(http.StatusCreated, response)
}

// RegisterMealPlanRoutes registers all meal plan-related routes
func RegisterMealPlanRoutes(router *gin.RouterGroup, mealPlanHandler *MealPlanHandler) {
	mealPlans := router.Group("/meal-plans")
	{
		mealPlans.POST("", mealPlanHandler.CreateMealPlan)
		mealPlans.POST("/generate", mealPlanHandler.GenerateWeeklyMealPlan) // "Fill My Week" endpoint
		mealPlans.GET("", mealPlanHandler.GetMealPlans)
		mealPlans.GET("/:id", mealPlanHandler.GetMealPlan)
		mealPlans.GET("/week/:date", mealPlanHandler.GetMealPlanByWeek)
		mealPlans.PUT("/:id", mealPlanHandler.UpdateMealPlan)
		mealPlans.DELETE("/:id", mealPlanHandler.DeleteMealPlan)
		mealPlans.PUT("/:id/entries/:day/:mealType", mealPlanHandler.UpdateMealSlot)
		mealPlans.DELETE("/:id/entries/:day/:mealType", mealPlanHandler.DeleteMealSlot)
	}
}
