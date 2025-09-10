package repositories

import (
	"fmt"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"gorm.io/gorm"
)

// PreferenceRepository handles user preference data access
type PreferenceRepository struct {
	DB *gorm.DB
}

// NewPreferenceRepository creates a new preference repository
func NewPreferenceRepository(db *gorm.DB) *PreferenceRepository {
	return &PreferenceRepository{DB: db}
}

// GetUserPreferences retrieves user's core preferences
func (pr *PreferenceRepository) GetUserPreferences(userID uuid.UUID) (*models.CoreUserPreferences, error) {
	var user models.User
	
	err := pr.DB.Select("max_cook_time, preferred_meal_complexity").
		Where("id = ? AND deleted_at IS NULL", userID).
		First(&user).Error
	
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, fmt.Errorf("user not found")
		}
		return nil, fmt.Errorf("failed to get user preferences: %w", err)
	}

	preferences := &models.CoreUserPreferences{
		MaxCookTime:         user.MaxCookTime,
		PreferredComplexity: user.PreferredMealComplexity,
	}

	// Apply defaults if values are not set
	if preferences.MaxCookTime == 0 {
		preferences.MaxCookTime = 60 // Default 60 minutes
	}
	if preferences.PreferredComplexity == "" {
		preferences.PreferredComplexity = "moderate" // Default complexity
	}

	return preferences, nil
}

// UpdateUserPreferences updates user's core preferences
func (pr *PreferenceRepository) UpdateUserPreferences(userID uuid.UUID, preferences *models.CoreUserPreferences) error {
	// Validate the user exists
	var userExists bool
	err := pr.DB.Model(&models.User{}).
		Select("count(*) > 0").
		Where("id = ? AND deleted_at IS NULL", userID).
		Find(&userExists).Error
	
	if err != nil {
		return fmt.Errorf("failed to check user existence: %w", err)
	}
	if !userExists {
		return fmt.Errorf("user not found")
	}

	// Update user preferences
	updateData := map[string]interface{}{
		"max_cook_time":             preferences.MaxCookTime,
		"preferred_meal_complexity": preferences.PreferredComplexity,
		"updated_at":                "now()",
	}

	result := pr.DB.Model(&models.User{}).
		Where("id = ? AND deleted_at IS NULL", userID).
		Updates(updateData)

	if result.Error != nil {
		return fmt.Errorf("failed to update user preferences: %w", result.Error)
	}

	if result.RowsAffected == 0 {
		return fmt.Errorf("no user found to update")
	}

	return nil
}

// GetUser retrieves user by ID (helper method)
func (pr *PreferenceRepository) GetUser(userID uuid.UUID) (*models.User, error) {
	var user models.User
	
	err := pr.DB.Where("id = ? AND deleted_at IS NULL", userID).First(&user).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, fmt.Errorf("user not found")
		}
		return nil, fmt.Errorf("failed to get user: %w", err)
	}
	
	return &user, nil
}

// ValidatePreferences validates preference data before saving
func (pr *PreferenceRepository) ValidatePreferences(preferences *models.CoreUserPreferences) error {
	// Validate MaxCookTime range (15-180 minutes)
	if preferences.MaxCookTime < 15 || preferences.MaxCookTime > 180 {
		return fmt.Errorf("max cook time must be between 15 and 180 minutes")
	}

	// Validate PreferredComplexity values
	validComplexities := map[string]bool{
		"simple":   true,
		"moderate": true,
		"complex":  true,
	}
	
	if !validComplexities[preferences.PreferredComplexity] {
		return fmt.Errorf("preferred complexity must be one of: simple, moderate, complex")
	}

	return nil
}

// GetUserWeeklyPatterns retrieves all weekly patterns for a user
func (pr *PreferenceRepository) GetUserWeeklyPatterns(userID uuid.UUID) ([]models.UserWeeklyPattern, error) {
	var patterns []models.UserWeeklyPattern
	
	err := pr.DB.Where("user_id = ?", userID).Find(&patterns).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get user weekly patterns: %w", err)
	}
	
	return patterns, nil
}

// CreateUserWeeklyPattern creates a new weekly pattern for a user
func (pr *PreferenceRepository) CreateUserWeeklyPattern(userID uuid.UUID, pattern *models.UserWeeklyPattern) (*models.UserWeeklyPattern, error) {
	pattern.UserID = userID
	
	err := pr.DB.Create(pattern).Error
	if err != nil {
		return nil, fmt.Errorf("failed to create weekly pattern: %w", err)
	}
	
	return pattern, nil
}

// UpdateUserWeeklyPattern updates an existing weekly pattern
func (pr *PreferenceRepository) UpdateUserWeeklyPattern(userID uuid.UUID, patternID uuid.UUID, updates *models.UserWeeklyPattern) (*models.UserWeeklyPattern, error) {
	var pattern models.UserWeeklyPattern
	
	// Find the pattern and verify ownership
	err := pr.DB.Where("id = ? AND user_id = ?", patternID, userID).First(&pattern).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, fmt.Errorf("weekly pattern not found or access denied")
		}
		return nil, fmt.Errorf("failed to find weekly pattern: %w", err)
	}
	
	// Update fields
	pattern.DayOfWeek = updates.DayOfWeek
	pattern.MaxPrepTime = updates.MaxPrepTime
	pattern.PreferredComplexity = updates.PreferredComplexity
	pattern.IsWeekendPattern = updates.IsWeekendPattern
	
	err = pr.DB.Save(&pattern).Error
	if err != nil {
		return nil, fmt.Errorf("failed to update weekly pattern: %w", err)
	}
	
	return &pattern, nil
}

// DeleteUserWeeklyPattern deletes a weekly pattern
func (pr *PreferenceRepository) DeleteUserWeeklyPattern(userID uuid.UUID, patternID uuid.UUID) error {
	result := pr.DB.Where("id = ? AND user_id = ?", patternID, userID).Delete(&models.UserWeeklyPattern{})
	
	if result.Error != nil {
		return fmt.Errorf("failed to delete weekly pattern: %w", result.Error)
	}
	
	if result.RowsAffected == 0 {
		return fmt.Errorf("weekly pattern not found or access denied")
	}
	
	return nil
}

// UpsertUserWeeklyPatterns replaces all weekly patterns for a user
func (pr *PreferenceRepository) UpsertUserWeeklyPatterns(userID uuid.UUID, patterns []models.UserWeeklyPattern) ([]models.UserWeeklyPattern, error) {
	// Start a transaction
	tx := pr.DB.Begin()
	
	// Delete existing patterns for the user
	if err := tx.Where("user_id = ?", userID).Delete(&models.UserWeeklyPattern{}).Error; err != nil {
		tx.Rollback()
		return nil, fmt.Errorf("failed to delete existing patterns: %w", err)
	}
	
	// Create new patterns
	var createdPatterns []models.UserWeeklyPattern
	for _, pattern := range patterns {
		pattern.UserID = userID
		pattern.ID = uuid.New() // Ensure new UUID
		
		if err := tx.Create(&pattern).Error; err != nil {
			tx.Rollback()
			return nil, fmt.Errorf("failed to create pattern: %w", err)
		}
		
		createdPatterns = append(createdPatterns, pattern)
	}
	
	// Commit transaction
	if err := tx.Commit().Error; err != nil {
		return nil, fmt.Errorf("failed to commit transaction: %w", err)
	}
	
	return createdPatterns, nil
}

// ValidateWeeklyPattern validates weekly pattern data
func (pr *PreferenceRepository) ValidateWeeklyPattern(pattern *models.UserWeeklyPattern) error {
	// Validate day of week (0-6)
	if pattern.DayOfWeek < 0 || pattern.DayOfWeek > 6 {
		return fmt.Errorf("day of week must be between 0 (Sunday) and 6 (Saturday)")
	}
	
	// Validate max prep time
	if pattern.MaxPrepTime < 5 || pattern.MaxPrepTime > 300 {
		return fmt.Errorf("max prep time must be between 5 and 300 minutes")
	}
	
	// Validate preferred complexity
	validComplexities := map[string]bool{
		"simple":   true,
		"moderate": true,
		"complex":  true,
	}
	
	if !validComplexities[pattern.PreferredComplexity] {
		return fmt.Errorf("preferred complexity must be one of: simple, moderate, complex")
	}
	
	return nil
}

// GetUserFavorites retrieves user's favorite recipes with pagination
func (pr *PreferenceRepository) GetUserFavorites(userID uuid.UUID, page, limit int) ([]models.UserRecipeFavorite, int64, error) {
	var favorites []models.UserRecipeFavorite
	var total int64
	
	// Count total favorites
	countQuery := pr.DB.Model(&models.UserRecipeFavorite{}).Where("user_id = ?", userID)
	if err := countQuery.Count(&total).Error; err != nil {
		return nil, 0, fmt.Errorf("failed to count user favorites: %w", err)
	}
	
	// Get paginated favorites with recipe details
	offset := (page - 1) * limit
	err := pr.DB.Preload("Recipe").
		Where("user_id = ?", userID).
		Order("favorited_at DESC").
		Offset(offset).
		Limit(limit).
		Find(&favorites).Error
	
	if err != nil {
		return nil, 0, fmt.Errorf("failed to get user favorites: %w", err)
	}
	
	return favorites, total, nil
}

// AddUserFavorite marks a recipe as favorite for a user
func (pr *PreferenceRepository) AddUserFavorite(userID, recipeID uuid.UUID) (*models.UserRecipeFavorite, error) {
	// Check if favorite already exists
	var existing models.UserRecipeFavorite
	err := pr.DB.Where("user_id = ? AND recipe_id = ?", userID, recipeID).First(&existing).Error
	
	if err == nil {
		// Already exists
		return nil, fmt.Errorf("recipe is already favorited")
	}
	
	if err != gorm.ErrRecordNotFound {
		// Database error
		return nil, fmt.Errorf("failed to check existing favorite: %w", err)
	}
	
	// Create new favorite
	favorite := &models.UserRecipeFavorite{
		UserID:           userID,
		RecipeID:         recipeID,
		WeightMultiplier: 1.5, // Default multiplier
	}
	
	err = pr.DB.Create(favorite).Error
	if err != nil {
		return nil, fmt.Errorf("failed to create favorite: %w", err)
	}
	
	return favorite, nil
}

// RemoveUserFavorite removes a recipe from user's favorites
func (pr *PreferenceRepository) RemoveUserFavorite(userID, recipeID uuid.UUID) error {
	result := pr.DB.Where("user_id = ? AND recipe_id = ?", userID, recipeID).
		Delete(&models.UserRecipeFavorite{})
	
	if result.Error != nil {
		return fmt.Errorf("failed to remove favorite: %w", result.Error)
	}
	
	if result.RowsAffected == 0 {
		return fmt.Errorf("favorite not found")
	}
	
	return nil
}

// IsUserFavorite checks if a recipe is favorited by a user
func (pr *PreferenceRepository) IsUserFavorite(userID, recipeID uuid.UUID) (bool, error) {
	var count int64
	err := pr.DB.Model(&models.UserRecipeFavorite{}).
		Where("user_id = ? AND recipe_id = ?", userID, recipeID).
		Count(&count).Error
	
	if err != nil {
		return false, fmt.Errorf("failed to check favorite status: %w", err)
	}
	
	return count > 0, nil
}

// GetFavoriteRecipeIDs returns a slice of favorite recipe IDs for a user
func (pr *PreferenceRepository) GetFavoriteRecipeIDs(userID uuid.UUID) ([]string, error) {
	var favorites []models.UserRecipeFavorite
	
	err := pr.DB.Select("recipe_id").
		Where("user_id = ?", userID).
		Find(&favorites).Error
	
	if err != nil {
		return nil, fmt.Errorf("failed to get favorite recipe IDs: %w", err)
	}
	
	recipeIDs := make([]string, len(favorites))
	for i, favorite := range favorites {
		recipeIDs[i] = favorite.RecipeID.String()
	}
	
	return recipeIDs, nil
}

// UpdateFavoriteMultiplier updates the weight multiplier for a favorite
func (pr *PreferenceRepository) UpdateFavoriteMultiplier(userID, recipeID uuid.UUID, multiplier float64) error {
	// Validate multiplier range (1.0 - 3.0)
	if multiplier < 1.0 || multiplier > 3.0 {
		return fmt.Errorf("weight multiplier must be between 1.0 and 3.0")
	}
	
	result := pr.DB.Model(&models.UserRecipeFavorite{}).
		Where("user_id = ? AND recipe_id = ?", userID, recipeID).
		Update("weight_multiplier", multiplier)
	
	if result.Error != nil {
		return fmt.Errorf("failed to update favorite multiplier: %w", result.Error)
	}
	
	if result.RowsAffected == 0 {
		return fmt.Errorf("favorite not found")
	}
	
	return nil
}

// CreateResetLog creates a new rotation reset log entry
func (pr *PreferenceRepository) CreateResetLog(resetLog *models.RotationResetLog) error {
	result := pr.DB.Create(resetLog)
	if result.Error != nil {
		return fmt.Errorf("failed to create reset log: %w", result.Error)
	}
	
	return nil
}