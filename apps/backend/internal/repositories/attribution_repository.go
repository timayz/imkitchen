package repositories

import (
	"context"
	"time"
	
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
)

// AttributionRepository handles recipe attribution data
type AttributionRepository interface {
	Create(ctx context.Context, attribution *models.RecipeAttribution) error
	GetByRecipeID(ctx context.Context, recipeID string) (*models.RecipeAttribution, error)
	IncrementImportCount(ctx context.Context, recipeID string) error
	GetContributorStats(ctx context.Context, contributorID string) (*ContributorStats, error)
	GetContributorBadges(ctx context.Context, contributorID string) ([]ContributorBadge, error)
	GetContributorAchievements(ctx context.Context, contributorID string) ([]ContributorAchievement, error)
	CreateAchievement(ctx context.Context, contributorID string, achievement *ContributorAchievement) error
	GetRecipeChain(ctx context.Context, recipeID string) ([]models.RecipeChainLink, error)
	UpdateAttribution(ctx context.Context, id uuid.UUID, attribution *models.RecipeAttribution) error
	DeleteAttribution(ctx context.Context, id uuid.UUID) error
}

// ContributorStats represents statistics for a contributor
type ContributorStats struct {
	TotalRecipes     int     `json:"total_recipes"`
	TotalImports     int     `json:"total_imports"`
	AverageRating    float64 `json:"average_rating"`
	PopularityScore  float64 `json:"popularity_score"`
}

// Attribution type alias for backward compatibility
type Attribution = models.RecipeAttribution

// ContributorBadge represents a contributor badge
type ContributorBadge struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	Emoji       string    `json:"emoji"`
	EarnedAt    time.Time `json:"earned_at"`
}

// ContributorAchievement represents a contributor achievement
type ContributorAchievement struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	Category    string    `json:"category"`
	Progress    int       `json:"progress"`
	MaxProgress int       `json:"max_progress"`
	UnlockedAt  *time.Time `json:"unlocked_at,omitempty"`
}