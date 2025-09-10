package services

import (
	"encoding/json"
	"fmt"
	"strings"

	"github.com/imkitchen/backend/internal/clients"
)

type UserService struct {
	supabaseClient        *clients.SupabaseClient
	supabaseServiceClient *clients.SupabaseClient
}

type UserProfile struct {
	ID                       string                 `json:"id"`
	Email                    string                 `json:"email"`
	EmailVerified            bool                   `json:"email_verified"`
	FirstName                string                 `json:"first_name"`
	LastName                 string                 `json:"last_name"`
	AvatarURL                string                 `json:"avatar_url"`
	DietaryRestrictions      []string               `json:"dietary_restrictions"`
	CookingSkillLevel        string                 `json:"cooking_skill_level"`
	PreferredMealComplexity  string                 `json:"preferred_meal_complexity"`
	PreferenceLearningData   map[string]interface{} `json:"preference_learning_data"`
	CreatedAt                string                 `json:"created_at"`
	UpdatedAt                string                 `json:"updated_at"`
}

type UpdateProfileRequest struct {
	FirstName               *string                `json:"first_name,omitempty" validate:"omitempty,min=1,max=100"`
	LastName                *string                `json:"last_name,omitempty" validate:"omitempty,min=1,max=100"`
	AvatarURL               *string                `json:"avatar_url,omitempty" validate:"omitempty,url"`
	DietaryRestrictions     *[]string              `json:"dietary_restrictions,omitempty"`
	CookingSkillLevel       *string                `json:"cooking_skill_level,omitempty" validate:"omitempty,oneof=beginner intermediate advanced"`
	PreferredMealComplexity *string                `json:"preferred_meal_complexity,omitempty" validate:"omitempty,oneof=simple moderate complex"`
	PreferenceLearningData  *map[string]interface{} `json:"preference_learning_data,omitempty"`
}

type ChangeEmailRequest struct {
	NewEmail string `json:"new_email" binding:"required,email" validate:"required,email,max=255"`
}

type ExportDataResponse struct {
	Profile UserProfile `json:"profile"`
	Data    interface{} `json:"data"`
}

// NewUserService creates a new user service
func NewUserService() (*UserService, error) {
	supabaseClient, err := clients.NewSupabaseClient()
	if err != nil {
		return nil, fmt.Errorf("failed to create Supabase client: %w", err)
	}

	supabaseServiceClient, err := clients.NewSupabaseServiceClient()
	if err != nil {
		return nil, fmt.Errorf("failed to create Supabase service client: %w", err)
	}

	return &UserService{
		supabaseClient:        supabaseClient,
		supabaseServiceClient: supabaseServiceClient,
	}, nil
}

// GetUserProfile retrieves user profile by ID
func (s *UserService) GetUserProfile(userID string) (*UserProfile, error) {
	// Get user profile from users table
	var userProfile []map[string]interface{}
	
	err := s.supabaseServiceClient.Client.DB.From("users").
		Select("*").
		Eq("id", userID).
		Execute(&userProfile)
	
	if err != nil {
		return nil, fmt.Errorf("failed to get user profile: %w", err)
	}

	if len(userProfile) == 0 {
		return nil, fmt.Errorf("user not found")
	}

	profile := convertToUserProfile(userProfile[0])
	return profile, nil
}

// UpdateUserProfile updates user profile
func (s *UserService) UpdateUserProfile(userID string, req UpdateProfileRequest) (*UserProfile, error) {
	updateData := make(map[string]interface{})

	if req.FirstName != nil {
		updateData["first_name"] = *req.FirstName
	}
	if req.LastName != nil {
		updateData["last_name"] = *req.LastName
	}
	if req.AvatarURL != nil {
		updateData["avatar_url"] = *req.AvatarURL
	}
	if req.DietaryRestrictions != nil {
		updateData["dietary_restrictions"] = *req.DietaryRestrictions
	}
	if req.CookingSkillLevel != nil {
		updateData["cooking_skill_level"] = *req.CookingSkillLevel
	}
	if req.PreferredMealComplexity != nil {
		updateData["preferred_meal_complexity"] = *req.PreferredMealComplexity
	}
	if req.PreferenceLearningData != nil {
		updateData["preference_learning_data"] = *req.PreferenceLearningData
	}

	updateData["updated_at"] = "NOW()"

	var updatedProfile []map[string]interface{}
	err := s.supabaseServiceClient.Client.DB.From("users").
		Update(updateData).
		Eq("id", userID).
		Select("*").
		Execute(&updatedProfile)

	if err != nil {
		return nil, fmt.Errorf("failed to update user profile: %w", err)
	}

	if len(updatedProfile) == 0 {
		return nil, fmt.Errorf("user not found or update failed")
	}

	profile := convertToUserProfile(updatedProfile[0])
	return profile, nil
}

// ChangeEmail initiates email change verification
func (s *UserService) ChangeEmail(userID string, req ChangeEmailRequest, accessToken string) error {
	// Set the access token for the client
	s.supabaseClient.Client.Auth.SetAuth(accessToken)

	// Update email with Supabase Auth (requires email confirmation)
	_, err := s.supabaseClient.Client.Auth.UpdateUser(map[string]interface{}{
		"email": req.NewEmail,
	})

	if err != nil {
		return fmt.Errorf("failed to initiate email change: %w", err)
	}

	return nil
}

// DeleteUserAccount deletes user account and exports data
func (s *UserService) DeleteUserAccount(userID string, accessToken string) (*ExportDataResponse, error) {
	// First, export user data
	exportData, err := s.ExportUserData(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to export user data: %w", err)
	}

	// Delete from users table
	var deletedUser []map[string]interface{}
	err = s.supabaseServiceClient.Client.DB.From("users").
		Delete().
		Eq("id", userID).
		Select("*").
		Execute(&deletedUser)

	if err != nil {
		return nil, fmt.Errorf("failed to delete user profile: %w", err)
	}

	// Delete from Supabase Auth
	s.supabaseClient.Client.Auth.SetAuth(accessToken)
	_, err = s.supabaseClient.Client.Auth.UpdateUser(map[string]interface{}{
		"email": fmt.Sprintf("deleted_%s@deleted.local", userID),
	})

	if err != nil {
		return nil, fmt.Errorf("failed to delete auth user: %w", err)
	}

	return exportData, nil
}

// ExportUserData exports user data in JSON format
func (s *UserService) ExportUserData(userID string) (*ExportDataResponse, error) {
	// Get user profile
	profile, err := s.GetUserProfile(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user profile for export: %w", err)
	}

	// TODO: Add other user data exports (recipes, meal plans, etc.)
	// For now, just return profile data
	exportData := &ExportDataResponse{
		Profile: *profile,
		Data: map[string]interface{}{
			"recipes":    []interface{}{}, // Placeholder for future recipe data
			"meal_plans": []interface{}{}, // Placeholder for future meal plan data
			"preferences": profile.PreferenceLearningData,
		},
	}

	return exportData, nil
}

// Helper function to convert database result to UserProfile
func convertToUserProfile(data map[string]interface{}) *UserProfile {
	profile := &UserProfile{}

	if id, ok := data["id"].(string); ok {
		profile.ID = id
	}
	if email, ok := data["email"].(string); ok {
		profile.Email = email
	}
	if emailVerified, ok := data["email_verified"].(bool); ok {
		profile.EmailVerified = emailVerified
	}
	if firstName, ok := data["first_name"].(string); ok {
		profile.FirstName = firstName
	}
	if lastName, ok := data["last_name"].(string); ok {
		profile.LastName = lastName
	}
	if avatarURL, ok := data["avatar_url"].(string); ok {
		profile.AvatarURL = avatarURL
	}
	if cookingSkillLevel, ok := data["cooking_skill_level"].(string); ok {
		profile.CookingSkillLevel = cookingSkillLevel
	}
	if preferredMealComplexity, ok := data["preferred_meal_complexity"].(string); ok {
		profile.PreferredMealComplexity = preferredMealComplexity
	}
	if createdAt, ok := data["created_at"].(string); ok {
		profile.CreatedAt = createdAt
	}
	if updatedAt, ok := data["updated_at"].(string); ok {
		profile.UpdatedAt = updatedAt
	}

	// Handle dietary restrictions array
	if dietaryRestrictionsRaw, ok := data["dietary_restrictions"]; ok {
		switch v := dietaryRestrictionsRaw.(type) {
		case []interface{}:
			restrictions := make([]string, len(v))
			for i, item := range v {
				if str, ok := item.(string); ok {
					restrictions[i] = str
				}
			}
			profile.DietaryRestrictions = restrictions
		case []string:
			profile.DietaryRestrictions = v
		case string:
			// Handle JSON string
			var restrictions []string
			if err := json.Unmarshal([]byte(v), &restrictions); err == nil {
				profile.DietaryRestrictions = restrictions
			}
		}
	}

	// Handle preference learning data
	if preferenceLearningDataRaw, ok := data["preference_learning_data"]; ok {
		switch v := preferenceLearningDataRaw.(type) {
		case map[string]interface{}:
			profile.PreferenceLearningData = v
		case string:
			// Handle JSON string
			var learningData map[string]interface{}
			if err := json.Unmarshal([]byte(v), &learningData); err == nil {
				profile.PreferenceLearningData = learningData
			}
		}
	}

	if profile.DietaryRestrictions == nil {
		profile.DietaryRestrictions = []string{}
	}
	if profile.PreferenceLearningData == nil {
		profile.PreferenceLearningData = make(map[string]interface{})
	}

	return profile
}

// ValidateUpdateProfileRequest validates update profile request
func ValidateUpdateProfileRequest(req UpdateProfileRequest) string {
	if req.FirstName != nil && (len(*req.FirstName) < 1 || len(*req.FirstName) > 100) {
		return "First name must be between 1 and 100 characters"
	}
	if req.LastName != nil && (len(*req.LastName) < 1 || len(*req.LastName) > 100) {
		return "Last name must be between 1 and 100 characters"
	}
	if req.CookingSkillLevel != nil {
		validSkills := []string{"beginner", "intermediate", "advanced"}
		if !userContains(validSkills, *req.CookingSkillLevel) {
			return "Cooking skill level must be one of: beginner, intermediate, advanced"
		}
	}
	if req.PreferredMealComplexity != nil {
		validComplexity := []string{"simple", "moderate", "complex"}
		if !userContains(validComplexity, *req.PreferredMealComplexity) {
			return "Preferred meal complexity must be one of: simple, moderate, complex"
		}
	}
	if req.DietaryRestrictions != nil {
		validRestrictions := []string{
			"vegetarian", "vegan", "gluten-free", "dairy-free", "nut-free",
			"shellfish-free", "egg-free", "soy-free", "low-carb", "keto",
			"paleo", "halal", "kosher",
		}
		for _, restriction := range *req.DietaryRestrictions {
			if !userContains(validRestrictions, strings.ToLower(restriction)) {
				return fmt.Sprintf("Invalid dietary restriction: %s", restriction)
			}
		}
	}
	return ""
}

// Helper function to check if slice contains string
func userContains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}