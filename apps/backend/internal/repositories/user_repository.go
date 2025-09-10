package repositories

import (
	"encoding/json"
	"fmt"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
)

// UserRepository defines methods for user data access
type UserRepository interface {
	GetByID(id uuid.UUID) (*models.User, error)
	Update(user *models.User) (*models.User, error)
	UpdatePreferenceLearningData(id uuid.UUID, data json.RawMessage) error
	UpdateRotationResetCount(id uuid.UUID, count int) error
	GetUserPreferences(id uuid.UUID) (*models.UserPreferences, error)
}

// UserPreferences type alias for backward compatibility
type UserPreferences = models.UserPreferences

type userRepository struct {
	db *gorm.DB
}

// NewUserRepository creates a new user repository
func NewUserRepository(db *gorm.DB) UserRepository {
	return &userRepository{db: db}
}

// GetByID retrieves a user by ID
func (r *userRepository) GetByID(id uuid.UUID) (*models.User, error) {
	var user models.User
	err := r.db.Where("id = ? AND deleted_at IS NULL", id).First(&user).Error
	if err != nil {
		return nil, err
	}
	return &user, nil
}

// Update updates a user record
func (r *userRepository) Update(user *models.User) (*models.User, error) {
	err := r.db.Save(user).Error
	if err != nil {
		return nil, err
	}
	return user, nil
}

// UpdatePreferenceLearningData updates the user's preference learning data
func (r *userRepository) UpdatePreferenceLearningData(id uuid.UUID, data json.RawMessage) error {
	return r.db.Model(&models.User{}).
		Where("id = ? AND deleted_at IS NULL", id).
		Update("preference_learning_data", data).Error
}

// UpdateRotationResetCount updates the user's rotation reset count
func (r *userRepository) UpdateRotationResetCount(id uuid.UUID, count int) error {
	return r.db.Model(&models.User{}).
		Where("id = ? AND deleted_at IS NULL", id).
		Update("rotation_reset_count", count).Error
}

// GetUserPreferences gets user preferences optimized for meal planning
func (r *userRepository) GetUserPreferences(id uuid.UUID) (*models.UserPreferences, error) {
	user, err := r.GetByID(id)
	if err != nil {
		return nil, fmt.Errorf("failed to get user: %w", err)
	}

	preferences := &models.UserPreferences{
		DietaryRestrictions:     user.DietaryRestrictions,
		CookingSkillLevel:       user.CookingSkillLevel,
		PreferredMealComplexity: user.PreferredMealComplexity,
		MaxPrepTimePerMeal:      user.MaxCookTime, // Use MaxCookTime from user
		WeeklyAvailability:      make(map[string]int),
		CuisinePreferences:      make([]string, 0),
		AvoidIngredients:        make([]string, 0),
	}

	// Parse preference learning data for additional preferences
	if len(user.PreferenceLearningData) > 0 {
		var learningData map[string]interface{}
		if err := json.Unmarshal(user.PreferenceLearningData, &learningData); err == nil {
			// Extract max prep time preference
			if maxPrepTime, exists := learningData["maxPrepTimePerMeal"]; exists {
				if prepTime, ok := maxPrepTime.(float64); ok {
					preferences.MaxPrepTimePerMeal = int(prepTime)
				}
			}

			// Extract cuisine preferences
			if cuisinePrefs, exists := learningData["cuisinePreferences"]; exists {
				if cuisines, ok := cuisinePrefs.([]interface{}); ok {
					for _, cuisine := range cuisines {
						if cuisineStr, ok := cuisine.(string); ok {
							preferences.CuisinePreferences = append(preferences.CuisinePreferences, cuisineStr)
						}
					}
				}
			}

			// Extract ingredients to avoid
			if avoidIngredients, exists := learningData["avoidIngredients"]; exists {
				if ingredients, ok := avoidIngredients.([]interface{}); ok {
					for _, ingredient := range ingredients {
						if ingredientStr, ok := ingredient.(string); ok {
							preferences.AvoidIngredients = append(preferences.AvoidIngredients, ingredientStr)
						}
					}
				}
			}
		}
	}

	// Set default weekly availability if empty
	if len(preferences.WeeklyAvailability) == 0 {
		days := []string{"monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"}
		for _, day := range days {
			preferences.WeeklyAvailability[day] = 90 // Default 90 minutes per day
		}
	}

	// Adjust max prep time based on cooking skill level
	switch user.CookingSkillLevel {
	case "beginner":
		if preferences.MaxPrepTimePerMeal > 45 {
			preferences.MaxPrepTimePerMeal = 45 // Limit beginners to 45 minutes
		}
	case "advanced":
		if preferences.MaxPrepTimePerMeal < 30 {
			preferences.MaxPrepTimePerMeal = 90 // Allow advanced cooks more time
		}
	}

	return preferences, nil
}