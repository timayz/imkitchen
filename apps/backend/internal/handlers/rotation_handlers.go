package handlers

import (
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// ResetRotationCycle resets user's recipe rotation cycle
func ResetRotationCycle(rotationService services.RotationService) gin.HandlerFunc {
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

		// Reset the rotation cycle
		err = rotationService.ResetRotationCycle(userUUID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to reset rotation cycle",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Rotation cycle reset successfully",
			"status":  "success",
		})
	}
}

// GetRotationStats retrieves rotation statistics and history for user insights
func GetRotationStats(rotationService services.RotationService) gin.HandlerFunc {
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

		// Get rotation state for statistics
		rotationState, err := rotationService.GetRotationState(userUUID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to get rotation statistics",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		// Calculate statistics
		stats := calculateRotationStats(rotationState)

		c.JSON(http.StatusOK, gin.H{
			"message": "Rotation statistics retrieved successfully",
			"status":  "success",
			"data":    stats,
		})
	}
}

// GetVarietyScore calculates variety score for a list of recipe IDs
func GetVarietyScore(rotationService services.RotationService) gin.HandlerFunc {
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

		// Get recipe IDs from query params
		recipeIDs := c.QueryArray("recipe_ids")
		if len(recipeIDs) == 0 {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":  "No recipe IDs provided",
				"status": "error",
			})
			return
		}

		// Calculate variety score
		varietyScore, err := rotationService.GetVarietyScore(recipeIDs, userUUID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to calculate variety score",
				"status":  "error",
				"details": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message": "Variety score calculated successfully",
			"status":  "success",
			"data": gin.H{
				"recipeIds":    recipeIDs,
				"varietyScore": varietyScore,
			},
		})
	}
}

// RotationStats represents rotation statistics for user insights
type RotationStats struct {
	CycleCount         int                  `json:"cycleCount"`
	TotalRecipesUsed   int                  `json:"totalRecipesUsed"`
	LastResetDate      *string              `json:"lastResetDate"`
	MealTypeDistribution map[string]int     `json:"mealTypeDistribution"`
	ComplexityDistribution map[string]int   `json:"complexityDistribution"`
	RecentRecipes      []RecentRecipeInfo   `json:"recentRecipes"`
	VarietyMetrics     VarietyMetrics       `json:"varietyMetrics"`
}

// RecentRecipeInfo represents recent recipe usage information
type RecentRecipeInfo struct {
	RecipeID   string `json:"recipeId"`
	LastUsed   string `json:"lastUsed"`
	UsageCount int    `json:"usageCount"`
	MealType   string `json:"mealType"`
}

// VarietyMetrics represents variety and rotation metrics
type VarietyMetrics struct {
	AverageVarietyScore     float64 `json:"averageVarietyScore"`
	ConsecutiveDuplicates   int     `json:"consecutiveDuplicates"`
	MostFrequentRecipe      string  `json:"mostFrequentRecipe"`
	LeastFrequentRecipe     string  `json:"leastFrequentRecipe"`
	CuisineVariety          int     `json:"cuisineVariety"`
	ComplexityBalance       float64 `json:"complexityBalance"`
}

// calculateRotationStats calculates comprehensive statistics from rotation state
func calculateRotationStats(state *services.RotationState) *RotationStats {
	if state == nil {
		return &RotationStats{
			MealTypeDistribution:   make(map[string]int),
			ComplexityDistribution: make(map[string]int),
			RecentRecipes:         []RecentRecipeInfo{},
			VarietyMetrics: VarietyMetrics{
				AverageVarietyScore: 0.0,
			},
		}
	}

	stats := &RotationStats{
		CycleCount:             state.CycleCount,
		TotalRecipesUsed:       state.TotalRecipesUsed,
		MealTypeDistribution:   calculateMealTypeDistribution(state.MealTypeHistory),
		ComplexityDistribution: calculateComplexityDistribution(state.ComplexityHistory),
		RecentRecipes:          getRecentRecipeInfo(state.UsedRecipes),
	}

	if state.LastResetDate != nil {
		resetDateStr := state.LastResetDate.Format("2006-01-02T15:04:05Z07:00")
		stats.LastResetDate = &resetDateStr
	}

	// Calculate variety metrics
	stats.VarietyMetrics = calculateVarietyMetrics(state)

	return stats
}

// calculateMealTypeDistribution calculates how meals are distributed across meal types
func calculateMealTypeDistribution(mealTypeHistory map[string][]string) map[string]int {
	distribution := make(map[string]int)
	for mealType, recipes := range mealTypeHistory {
		distribution[mealType] = len(recipes)
	}
	return distribution
}

// calculateComplexityDistribution calculates distribution of complexity levels
func calculateComplexityDistribution(complexityHistory []string) map[string]int {
	distribution := make(map[string]int)
	for _, complexity := range complexityHistory {
		distribution[complexity]++
	}
	return distribution
}

// getRecentRecipeInfo converts recipe usage map to recent recipe info
func getRecentRecipeInfo(usedRecipes map[string]time.Time) []RecentRecipeInfo {
	var recentRecipes []RecentRecipeInfo
	
	// Convert to slice and sort by last used date (most recent first)
	for recipeID, lastUsed := range usedRecipes {
		recentRecipes = append(recentRecipes, RecentRecipeInfo{
			RecipeID: recipeID,
			LastUsed: lastUsed.Format("2006-01-02T15:04:05Z07:00"),
			// UsageCount and MealType would need additional data structure to track
		})
	}
	
	return recentRecipes
}

// calculateVarietyMetrics calculates variety and balance metrics
func calculateVarietyMetrics(state *services.RotationState) VarietyMetrics {
	// Basic implementation - would be enhanced with more sophisticated calculations
	return VarietyMetrics{
		AverageVarietyScore:   calculateAverageVariety(state),
		ConsecutiveDuplicates: 0, // Would need additional tracking
		CuisineVariety:        len(state.MealTypeHistory), // Simplified
		ComplexityBalance:     calculateComplexityBalance(state.ComplexityHistory),
	}
}

// calculateAverageVariety calculates average variety score
func calculateAverageVariety(state *services.RotationState) float64 {
	if state.TotalRecipesUsed == 0 {
		return 0.0
	}
	// Simple variety calculation based on unique recipes vs total usage
	return float64(len(state.UsedRecipes)) / float64(state.TotalRecipesUsed)
}

// calculateComplexityBalance calculates how balanced the complexity distribution is
func calculateComplexityBalance(complexityHistory []string) float64 {
	if len(complexityHistory) == 0 {
		return 0.0
	}
	
	// Calculate balance as standard deviation from ideal distribution (33% each)
	distribution := calculateComplexityDistribution(complexityHistory)
	total := len(complexityHistory)
	ideal := float64(total) / 3.0
	
	variance := 0.0
	for _, count := range distribution {
		diff := float64(count) - ideal
		variance += diff * diff
	}
	
	variance /= 3.0 // Three complexity levels
	balance := 1.0 / (1.0 + variance/float64(total))
	
	return balance
}

// ResetRotationCycleWithOptions resets user's recipe rotation cycle with options
func ResetRotationCycleWithOptions(rotationService services.RotationService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":   "Unauthorized",
				"message": "User ID not found in token",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Invalid user ID format",
				"message": "User ID format is invalid",
			})
			return
		}

		userUUID, err := uuid.Parse(userIDStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid user ID",
				"message": "User ID must be a valid UUID",
			})
			return
		}

		// Parse reset request
		var resetReq models.RotationResetRequest
		if err := c.ShouldBindJSON(&resetReq); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid request body",
				"message": err.Error(),
			})
			return
		}

		// Validate confirmation
		if !resetReq.ConfirmReset {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Reset confirmation required",
				"message": "confirmReset must be true to proceed with reset",
			})
			return
		}

		// Reset rotation cycle with options
		err = rotationService.ResetRotationCycleWithOptions(userUUID, &resetReq)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to reset rotation cycle",
				"message": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"message":   "Rotation cycle reset successfully",
			"resetAt":   time.Now().Format(time.RFC3339),
			"preserved": gin.H{
				"patterns":  resetReq.PreservePatterns,
				"favorites": resetReq.PreserveFavorites,
			},
		})
	}
}

// GetRotationAnalytics provides comprehensive rotation analytics
func GetRotationAnalytics(rotationService services.RotationService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":   "Unauthorized",
				"message": "User ID not found in token",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Invalid user ID format",
				"message": "User ID format is invalid",
			})
			return
		}

		userUUID, err := uuid.Parse(userIDStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid user ID",
				"message": "User ID must be a valid UUID",
			})
			return
		}

		// Parse query parameters
		weeks := 12 // default
		if weeksParam := c.Query("weeks"); weeksParam != "" {
			if w, err := strconv.Atoi(weeksParam); err == nil && w > 0 && w <= 52 {
				weeks = w
			}
		}

		// Get analytics
		analytics, err := rotationService.GetRotationAnalytics(userUUID, weeks)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to get rotation analytics",
				"message": err.Error(),
			})
			return
		}

		c.JSON(http.StatusOK, gin.H{
			"data": analytics,
			"metadata": gin.H{
				"generatedAt": time.Now().Format(time.RFC3339),
				"weeksAnalyzed": weeks,
			},
		})
	}
}

// ExportRotationData exports rotation data for analysis
func ExportRotationData(rotationService services.RotationService) gin.HandlerFunc {
	return func(c *gin.Context) {
		userID, exists := c.Get("UserID")
		if !exists {
			c.JSON(http.StatusUnauthorized, gin.H{
				"error":   "Unauthorized",
				"message": "User ID not found in token",
			})
			return
		}

		userIDStr, ok := userID.(string)
		if !ok {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Invalid user ID format",
				"message": "User ID format is invalid",
			})
			return
		}

		userUUID, err := uuid.Parse(userIDStr)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid user ID",
				"message": "User ID must be a valid UUID",
			})
			return
		}

		// Parse query parameters
		format := c.DefaultQuery("format", "json")
		if format != "json" && format != "csv" {
			c.JSON(http.StatusBadRequest, gin.H{
				"error":   "Invalid format",
				"message": "format must be 'json' or 'csv'",
			})
			return
		}

		// Parse date range (optional)
		now := time.Now()
		startDate := now.AddDate(-1, 0, 0) // Default: 1 year ago
		endDate := now

		if dateRange := c.Query("dateRange"); dateRange != "" {
			// Parse date range in format: "2025-01-01,2025-12-31"
			// For simplicity, using defaults here - could be enhanced
		}

		// Export data
		data, err := rotationService.ExportRotationData(userUUID, format, startDate, endDate)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Failed to export rotation data",
				"message": err.Error(),
			})
			return
		}

		// Set appropriate content type and filename
		filename := fmt.Sprintf("rotation_export_%s.%s", time.Now().Format("2006-01-02"), format)
		
		switch format {
		case "json":
			c.Header("Content-Type", "application/json")
		case "csv":
			c.Header("Content-Type", "text/csv")
		}
		
		c.Header("Content-Disposition", fmt.Sprintf("attachment; filename=%s", filename))
		c.Data(http.StatusOK, c.GetHeader("Content-Type"), data)
	}
}