package services

import (
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// PreferenceService handles business logic for user preferences
type PreferenceService struct {
	repo *repositories.PreferenceRepository
}

// NewPreferenceService creates a new preference service
func NewPreferenceService(repo *repositories.PreferenceRepository) *PreferenceService {
	return &PreferenceService{repo: repo}
}

// GetUserPreferences retrieves user's core preferences with defaults
func (ps *PreferenceService) GetUserPreferences(userID uuid.UUID) (*models.CoreUserPreferences, error) {
	if userID == uuid.Nil {
		return nil, fmt.Errorf("invalid user ID")
	}

	preferences, err := ps.repo.GetUserPreferences(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user preferences: %w", err)
	}

	// Ensure sensible defaults
	ps.applyDefaults(preferences)

	return preferences, nil
}

// UpdateUserPreferences updates user's core preferences with validation
func (ps *PreferenceService) UpdateUserPreferences(userID uuid.UUID, preferences *models.CoreUserPreferences) error {
	if userID == uuid.Nil {
		return fmt.Errorf("invalid user ID")
	}

	if preferences == nil {
		return fmt.Errorf("preferences cannot be nil")
	}

	// Validate preference data
	if err := ps.validatePreferences(preferences); err != nil {
		return fmt.Errorf("preference validation failed: %w", err)
	}

	// Apply defaults for any missing values
	ps.applyDefaults(preferences)

	// Update in repository
	if err := ps.repo.UpdateUserPreferences(userID, preferences); err != nil {
		return fmt.Errorf("failed to update user preferences: %w", err)
	}

	return nil
}

// applyDefaults ensures preferences have sensible default values
func (ps *PreferenceService) applyDefaults(preferences *models.CoreUserPreferences) {
	if preferences.MaxCookTime <= 0 {
		preferences.MaxCookTime = 60 // Default 60 minutes
	}
	if preferences.PreferredComplexity == "" {
		preferences.PreferredComplexity = "moderate" // Default complexity
	}
}

// validatePreferences validates preference data with business rules
func (ps *PreferenceService) validatePreferences(preferences *models.CoreUserPreferences) error {
	// Validate MaxCookTime range (15-180 minutes as per story requirements)
	if preferences.MaxCookTime != 0 && (preferences.MaxCookTime < 15 || preferences.MaxCookTime > 180) {
		return fmt.Errorf("max cook time must be between 15 and 180 minutes, got %d", preferences.MaxCookTime)
	}

	// Validate PreferredComplexity values
	if preferences.PreferredComplexity != "" {
		validComplexities := map[string]bool{
			"simple":   true,
			"moderate": true,
			"complex":  true,
		}
		
		if !validComplexities[preferences.PreferredComplexity] {
			return fmt.Errorf("preferred complexity must be one of: simple, moderate, complex, got '%s'", preferences.PreferredComplexity)
		}
	}

	return nil
}

// GetUserPreferencesWithMetadata retrieves preferences with additional user metadata
func (ps *PreferenceService) GetUserPreferencesWithMetadata(userID uuid.UUID) (map[string]interface{}, error) {
	preferences, err := ps.GetUserPreferences(userID)
	if err != nil {
		return nil, err
	}

	user, err := ps.repo.GetUser(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user metadata: %w", err)
	}

	result := map[string]interface{}{
		"preferences": preferences,
		"metadata": map[string]interface{}{
			"cookingSkillLevel": user.CookingSkillLevel,
			"lastUpdated":      user.UpdatedAt.Format(time.RFC3339),
		},
	}

	return result, nil
}

// ResetUserPreferences resets user preferences to default values
func (ps *PreferenceService) ResetUserPreferences(userID uuid.UUID) error {
	defaultPreferences := &models.CoreUserPreferences{
		MaxCookTime:         60,        // Default 60 minutes
		PreferredComplexity: "moderate", // Default complexity
	}

	return ps.UpdateUserPreferences(userID, defaultPreferences)
}