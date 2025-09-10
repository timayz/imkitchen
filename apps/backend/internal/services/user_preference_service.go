package services

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// UserPreferenceService manages advanced user cooking preferences
type UserPreferenceService interface {
	GetUserPreferences(userID uuid.UUID) (*models.UserPreferences, error)
	UpdateUserPreferences(userID uuid.UUID, preferences *UpdatePreferencesRequest) (*models.UserPreferences, error)
	GetPreferenceHistory(userID uuid.UUID) ([]*PreferenceHistoryEntry, error)
	ValidatePreferences(preferences *UpdatePreferencesRequest) []string
	GetDefaultPreferences() *models.UserPreferences
}

// UpdatePreferencesRequest represents the request payload for updating user preferences
type UpdatePreferencesRequest struct {
	MaxPrepTimePerMeal      *int               `json:"maxPrepTimePerMeal,omitempty" validate:"omitempty,min=5,max=300"`
	WeeklyAvailability      map[string]int     `json:"weeklyAvailability,omitempty" validate:"omitempty"`
	PreferredMealComplexity *string            `json:"preferredMealComplexity,omitempty" validate:"omitempty,oneof=simple moderate complex"`
	CuisinePreferences      []string           `json:"cuisinePreferences,omitempty"`
	AvoidIngredients        []string           `json:"avoidIngredients,omitempty"`
	DietaryRestrictions     []string           `json:"dietaryRestrictions,omitempty"`
	CookingSkillLevel       *string            `json:"cookingSkillLevel,omitempty" validate:"omitempty,oneof=beginner intermediate advanced"`
}

// PreferenceHistoryEntry tracks changes to user preferences over time
type PreferenceHistoryEntry struct {
	ID              uuid.UUID                `json:"id"`
	UserID          uuid.UUID                `json:"userId"`
	PreviousPrefs   *models.UserPreferences  `json:"previousPreferences"`
	NewPrefs        *models.UserPreferences  `json:"newPreferences"`
	ChangedFields   []string                 `json:"changedFields"`
	ChangeReason    string                   `json:"changeReason"`
	CreatedAt       time.Time                `json:"createdAt"`
}

type userPreferenceService struct {
	userRepo     repositories.UserRepository
	cacheService CacheServiceInterface
}

// NewUserPreferenceService creates a new user preference service
func NewUserPreferenceService(userRepo repositories.UserRepository, cacheService CacheServiceInterface) UserPreferenceService {
	return &userPreferenceService{
		userRepo:     userRepo,
		cacheService: cacheService,
	}
}

// GetUserPreferences retrieves current user preferences
func (s *userPreferenceService) GetUserPreferences(userID uuid.UUID) (*models.UserPreferences, error) {
	// Try to get from cache first
	cacheKey := fmt.Sprintf("user_preferences:%s", userID.String())
	if s.cacheService != nil {
		if cached := s.cacheService.Get(cacheKey); cached != "" {
			var preferences models.UserPreferences
			if err := json.Unmarshal([]byte(cached), &preferences); err == nil {
				return &preferences, nil
			}
		}
	}

	// Get user from database
	user, err := s.userRepo.GetByID(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user: %w", err)
	}

	// Extract preferences from user model
	preferences := s.extractPreferencesFromUser(user)

	// Cache the preferences
	if s.cacheService != nil {
		if prefData, err := json.Marshal(preferences); err == nil {
			s.cacheService.Set(cacheKey, string(prefData), time.Hour) // Cache for 1 hour
		}
	}

	return preferences, nil
}

// UpdateUserPreferences updates user cooking preferences
func (s *userPreferenceService) UpdateUserPreferences(userID uuid.UUID, req *UpdatePreferencesRequest) (*models.UserPreferences, error) {
	// Validate the request
	if errors := s.ValidatePreferences(req); len(errors) > 0 {
		return nil, fmt.Errorf("validation failed: %v", errors)
	}

	// Get current user
	user, err := s.userRepo.GetByID(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user: %w", err)
	}

	// Get current preferences for history tracking
	currentPrefs := s.extractPreferencesFromUser(user)

	// Update user model with new preferences
	s.updateUserFromPreferences(user, req)

	// Save updated user
	updatedUser, err := s.userRepo.Update(user)
	if err != nil {
		return nil, fmt.Errorf("failed to update user preferences: %w", err)
	}

	// Extract new preferences
	newPrefs := s.extractPreferencesFromUser(updatedUser)

	// Track preference history
	s.trackPreferenceChange(userID, currentPrefs, newPrefs, "user_update")

	// Clear cache
	cacheKey := fmt.Sprintf("user_preferences:%s", userID.String())
	if s.cacheService != nil {
		s.cacheService.Delete(cacheKey)
	}

	return newPrefs, nil
}

// GetPreferenceHistory retrieves user preference change history
func (s *userPreferenceService) GetPreferenceHistory(userID uuid.UUID) ([]*PreferenceHistoryEntry, error) {
	// For now, return empty history - would be implemented with a separate history table
	return []*PreferenceHistoryEntry{}, nil
}

// ValidatePreferences validates preference update request
func (s *userPreferenceService) ValidatePreferences(req *UpdatePreferencesRequest) []string {
	var errors []string

	// Validate MaxPrepTimePerMeal
	if req.MaxPrepTimePerMeal != nil {
		if *req.MaxPrepTimePerMeal < 5 || *req.MaxPrepTimePerMeal > 300 {
			errors = append(errors, "maxPrepTimePerMeal must be between 5 and 300 minutes")
		}
	}

	// Validate WeeklyAvailability
	if req.WeeklyAvailability != nil {
		validDays := map[string]bool{
			"monday": true, "tuesday": true, "wednesday": true, "thursday": true,
			"friday": true, "saturday": true, "sunday": true,
		}
		for day, minutes := range req.WeeklyAvailability {
			if !validDays[day] {
				errors = append(errors, fmt.Sprintf("invalid day in weeklyAvailability: %s", day))
			}
			if minutes < 0 || minutes > 480 { // Max 8 hours
				errors = append(errors, fmt.Sprintf("invalid minutes for %s: must be between 0 and 480", day))
			}
		}
	}

	// Validate PreferredMealComplexity
	if req.PreferredMealComplexity != nil {
		validComplexities := map[string]bool{"simple": true, "moderate": true, "complex": true}
		if !validComplexities[*req.PreferredMealComplexity] {
			errors = append(errors, "preferredMealComplexity must be one of: simple, moderate, complex")
		}
	}

	// Validate CookingSkillLevel
	if req.CookingSkillLevel != nil {
		validSkills := map[string]bool{"beginner": true, "intermediate": true, "advanced": true}
		if !validSkills[*req.CookingSkillLevel] {
			errors = append(errors, "cookingSkillLevel must be one of: beginner, intermediate, advanced")
		}
	}

	return errors
}

// GetDefaultPreferences returns sensible default preferences
func (s *userPreferenceService) GetDefaultPreferences() *models.UserPreferences {
	return &models.UserPreferences{
		DietaryRestrictions:     []string{},
		CookingSkillLevel:       "intermediate",
		PreferredMealComplexity: "moderate",
		MaxPrepTimePerMeal:      45,
		WeeklyAvailability: map[string]int{
			"monday":    30,
			"tuesday":   30,
			"wednesday": 30,
			"thursday":  30,
			"friday":    45,
			"saturday":  60,
			"sunday":    60,
		},
		CuisinePreferences: []string{},
		AvoidIngredients:   []string{},
	}
}

// Helper method to extract preferences from user model
func (s *userPreferenceService) extractPreferencesFromUser(user *models.User) *models.UserPreferences {
	// Parse preference learning data for weekly availability
	weeklyAvailability := map[string]int{
		"monday": 30, "tuesday": 30, "wednesday": 30, "thursday": 30,
		"friday": 45, "saturday": 60, "sunday": 60,
	}

	if user.PreferenceLearningData != nil {
		var learningData map[string]interface{}
		if err := json.Unmarshal(user.PreferenceLearningData, &learningData); err == nil {
			if availability, ok := learningData["weeklyAvailability"].(map[string]interface{}); ok {
				for day, minutes := range availability {
					if minutesFloat, ok := minutes.(float64); ok {
						weeklyAvailability[day] = int(minutesFloat)
					}
				}
			}
		}
	}

	return &models.UserPreferences{
		DietaryRestrictions:     user.DietaryRestrictions,
		CookingSkillLevel:       user.CookingSkillLevel,
		PreferredMealComplexity: user.PreferredMealComplexity,
		MaxPrepTimePerMeal:      user.MaxCookTime, // Use existing field as default
		WeeklyAvailability:      weeklyAvailability,
		CuisinePreferences:      []string{}, // Default empty, could be stored in learning data
		AvoidIngredients:        user.Allergies,
	}
}

// Helper method to update user model from preferences
func (s *userPreferenceService) updateUserFromPreferences(user *models.User, req *UpdatePreferencesRequest) {
	if req.CookingSkillLevel != nil {
		user.CookingSkillLevel = *req.CookingSkillLevel
	}
	if req.PreferredMealComplexity != nil {
		user.PreferredMealComplexity = *req.PreferredMealComplexity
	}
	if req.MaxPrepTimePerMeal != nil {
		user.MaxCookTime = *req.MaxPrepTimePerMeal
	}
	if req.DietaryRestrictions != nil {
		user.DietaryRestrictions = req.DietaryRestrictions
	}

	// Update preference learning data
	var learningData map[string]interface{}
	if user.PreferenceLearningData != nil {
		json.Unmarshal(user.PreferenceLearningData, &learningData)
	} else {
		learningData = make(map[string]interface{})
	}

	if req.WeeklyAvailability != nil {
		learningData["weeklyAvailability"] = req.WeeklyAvailability
	}
	if req.CuisinePreferences != nil {
		learningData["cuisinePreferences"] = req.CuisinePreferences
	}

	// Save updated learning data
	updatedData, _ := json.Marshal(learningData)
	user.PreferenceLearningData = updatedData
}

// Helper method to track preference changes
func (s *userPreferenceService) trackPreferenceChange(userID uuid.UUID, oldPrefs, newPrefs *models.UserPreferences, reason string) {
	// For now, just log the change - in a production system this would save to a history table
	// This method is a placeholder for future preference history tracking
}