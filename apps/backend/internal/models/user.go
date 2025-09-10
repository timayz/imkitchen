package models

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/lib/pq"
)

// User represents a user in the system
type User struct {
	ID                    uuid.UUID       `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	Email                 string          `json:"email" gorm:"size:255;uniqueIndex;not null" validate:"required,email,max=255"`
	EmailVerified         bool            `json:"emailVerified" gorm:"column:email_verified;default:false"`
	EncryptedPassword     string          `json:"-" gorm:"column:encrypted_password;size:255"`

	// Profile Information
	Username              string          `json:"username" gorm:"size:100;uniqueIndex"`
	DisplayName           string          `json:"displayName" gorm:"column:display_name;size:100"`
	FirstName             string          `json:"firstName" gorm:"column:first_name;size:100"`
	LastName              string          `json:"lastName" gorm:"column:last_name;size:100"`
	AvatarURL             string          `json:"avatarUrl" gorm:"column:avatar_url;type:text"`

	// Preferences
	DietaryRestrictions   pq.StringArray  `json:"dietaryRestrictions" gorm:"column:dietary_restrictions;type:text[];default:'{}'"`
	Allergies             pq.StringArray  `json:"allergies" gorm:"type:text[];default:'{}'"`
	CookingSkillLevel     string          `json:"cookingSkillLevel" gorm:"column:cooking_skill_level;size:20" validate:"oneof=beginner intermediate advanced"`
	PreferredMealComplexity string        `json:"preferredMealComplexity" gorm:"column:preferred_meal_complexity;size:20" validate:"oneof=simple moderate complex"`
	MaxCookTime           int             `json:"maxCookTime" gorm:"column:max_cook_time;default:60"` // minutes

	// Learning Algorithm Data
	RotationResetCount    int             `json:"rotationResetCount" gorm:"column:rotation_reset_count;default:0"`
	PreferenceLearningData json.RawMessage `json:"preferenceLearningData" gorm:"column:preference_learning_data;type:jsonb;default:'{}'"`

	// Additional Profile Fields
	Bio                   *string         `json:"bio,omitempty" gorm:"column:bio;type:text"`
	Location              *string         `json:"location,omitempty" gorm:"column:location;size:100"`
	Website               *string         `json:"website,omitempty" gorm:"column:website;type:text"`

	// Metadata
	CreatedAt             time.Time       `json:"createdAt" gorm:"column:created_at;default:now()"`
	UpdatedAt             time.Time       `json:"updatedAt" gorm:"column:updated_at;default:now()"`
	LastActiveAt          *time.Time      `json:"lastActiveAt" gorm:"column:last_active_at"`
	DeletedAt             *time.Time      `json:"deletedAt" gorm:"column:deleted_at"`
}

// UserPreferences holds user's dietary and cooking preferences for meal planning
type UserPreferences struct {
	DietaryRestrictions     []string       `json:"dietaryRestrictions"`
	CookingSkillLevel       string         `json:"cookingSkillLevel"`    // beginner, intermediate, advanced
	PreferredMealComplexity string         `json:"preferredMealComplexity"` // simple, moderate, complex
	MaxPrepTimePerMeal      int            `json:"maxPrepTimePerMeal"`
	WeeklyAvailability      map[string]int `json:"weeklyAvailability"` // day -> available minutes
	CuisinePreferences      []string       `json:"cuisinePreferences"`
	AvoidIngredients        []string       `json:"avoidIngredients"`
}

// TableName specifies the table name for GORM
func (User) TableName() string {
	return "users"
}