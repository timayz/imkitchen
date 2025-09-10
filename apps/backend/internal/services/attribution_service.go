package services

import (
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

type AttributionService struct {
	attributionRepo repositories.AttributionRepository
	userRepo        repositories.UserRepository
	recipeRepo      repositories.RecipeRepository
	metricsRepo     repositories.MetricsRepository
}

func NewAttributionService(
	attributionRepo repositories.AttributionRepository,
	userRepo repositories.UserRepository,
	recipeRepo repositories.RecipeRepository,
	metricsRepo repositories.MetricsRepository,
) *AttributionService {
	return &AttributionService{
		attributionRepo: attributionRepo,
		userRepo:        userRepo,
		recipeRepo:      recipeRepo,
		metricsRepo:     metricsRepo,
	}
}

// RecipeAttribution type alias for consistency with models package
type RecipeAttribution = models.RecipeAttribution

// Type aliases for consistency with models package
type CommunityMetrics = models.CommunityMetrics
type RecipeChainLink = models.RecipeChainLink
type EngagementStats = models.EngagementStats
type ImportCustomizations = models.ImportCustomizations

// Type aliases for repository types
type ContributorBadge = repositories.ContributorBadge
type ContributorAchievement = repositories.ContributorAchievement

// Type aliases for metrics
type MetricsOverview = repositories.RecipeOverview
type PopularityMetrics = repositories.RecipePopularity
type EngagementMetrics = repositories.RecipeEngagement
type GeographicMetrics = repositories.RecipeGeographic

type ContributorProfile struct {
	ID            string                   `json:"id"`
	Username      string                   `json:"username"`
	DisplayName   string                   `json:"display_name"`
	AvatarURL     *string                  `json:"avatar_url"`
	TotalRecipes  int                      `json:"total_recipes"`
	AverageRating float64                  `json:"average_rating"`
	TotalImports  int                      `json:"total_imports"`
	JoinedAt      time.Time                `json:"joined_at"`
	Badges        []ContributorBadge       `json:"badges"`
	Achievements  []ContributorAchievement `json:"achievements"`
	Bio           *string                  `json:"bio"`
	Location      *string                  `json:"location"`
	Website       *string                  `json:"website"`
}


type CommunityMetricsData struct {
	Overview     *MetricsOverview     `json:"overview,omitempty"`
	Popularity   *PopularityMetrics   `json:"popularity,omitempty"`
	Engagement   *EngagementMetrics   `json:"engagement,omitempty"`
	Geographic   *GeographicMetrics   `json:"geographic,omitempty"`
	Achievements []ContributorAchievement `json:"achievements,omitempty"`
}


type RegionMetric struct {
	Name  string `json:"name"`
	Flag  string `json:"flag"`
	Count int    `json:"count"`
}

type MetricsTimeframe string

const (
	TimeframeDay     MetricsTimeframe = "day"
	TimeframeWeek    MetricsTimeframe = "week"
	TimeframeMonth   MetricsTimeframe = "month"
	TimeframeQuarter MetricsTimeframe = "quarter"
	TimeframeYear    MetricsTimeframe = "year"
	TimeframeAll     MetricsTimeframe = "all"
)

func (s *AttributionService) CreateRecipeAttribution(ctx context.Context, req *models.RecipeImportRequest, personalRecipeID string) (*RecipeAttribution, error) {
	// Convert string ID to UUID
	recipeUUID, err := uuid.Parse(req.CommunityRecipeID)
	if err != nil {
		return nil, fmt.Errorf("invalid recipe ID format: %w", err)
	}

	// Get the original recipe details
	originalRecipe, err := s.recipeRepo.GetCommunityRecipeByID(ctx, recipeUUID)
	if err != nil {
		return nil, fmt.Errorf("failed to get original recipe: %w", err)
	}

	// Get contributor details
	contributor, err := s.userRepo.GetByID(originalRecipe.UserID)
	if err != nil {
		return nil, fmt.Errorf("failed to get contributor: %w", err)
	}

	// Create attribution record
	attribution := &models.RecipeAttribution{
		RecipeID:              personalRecipeID,
		OriginalContributorID: originalRecipe.UserID.String(),
		OriginalContributor:   contributor.Username,
		ImportDate:            time.Now(),
		PreserveAttribution:   req.PreserveAttribution,
		Customizations:        extractRecipeImportCustomizations(req.Customizations),
	}

	if err := s.attributionRepo.Create(ctx, attribution); err != nil {
		return nil, fmt.Errorf("failed to create attribution record: %w", err)
	}

	// Build recipe chain
	chain, err := s.buildRecipeChain(ctx, req.CommunityRecipeID)
	if err != nil {
		// Log warning but don't fail the request
		fmt.Printf("Warning: failed to build recipe chain: %v\n", err)
	}

	// Get community metrics
	communityMetrics, err := s.getCommunityMetrics(ctx, req.CommunityRecipeID)
	if err != nil {
		// Log warning but don't fail the request
		fmt.Printf("Warning: failed to get community metrics: %v\n", err)
		communityMetrics = &CommunityMetrics{}
	}

	// Increment import count
	go func() {
		if err := s.attributionRepo.IncrementImportCount(context.Background(), req.CommunityRecipeID); err != nil {
			fmt.Printf("Warning: failed to increment import count: %v\n", err)
		}
	}()

	return &RecipeAttribution{
		ID:                    attribution.ID,
		RecipeID:              personalRecipeID,
		OriginalContributorID: originalRecipe.UserID.String(),
		OriginalContributor:   contributor.Username,
		ImportDate:            attribution.ImportDate,
		PreserveAttribution:   attribution.PreserveAttribution,
		Customizations:        attribution.Customizations,
		CommunityMetrics:      *communityMetrics,
		RecipeChain:          chain,
	}, nil
}

func (s *AttributionService) GetRecipeAttribution(ctx context.Context, recipeID string) (*RecipeAttribution, error) {
	attribution, err := s.attributionRepo.GetByRecipeID(ctx, recipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get attribution: %w", err)
	}

	// Convert string ID to UUID
	contributorUUID, err := uuid.Parse(attribution.OriginalContributorID)
	if err != nil {
		return nil, fmt.Errorf("invalid contributor ID format: %w", err)
	}

	// Get contributor details
	contributor, err := s.userRepo.GetByID(contributorUUID)
	if err != nil {
		return nil, fmt.Errorf("failed to get contributor: %w", err)
	}

	// Build recipe chain
	chain, err := s.buildRecipeChain(ctx, attribution.RecipeID)
	if err != nil {
		fmt.Printf("Warning: failed to build recipe chain: %v\n", err)
	}

	// Get community metrics
	communityMetrics, err := s.getCommunityMetrics(ctx, attribution.RecipeID)
	if err != nil {
		fmt.Printf("Warning: failed to get community metrics: %v\n", err)
		communityMetrics = &CommunityMetrics{}
	}

	// Get engagement stats
	engagementStats, err := s.getEngagementStats(ctx, recipeID)
	if err != nil {
		fmt.Printf("Warning: failed to get engagement stats: %v\n", err)
	}

	return &RecipeAttribution{
		ID:                    attribution.ID,
		RecipeID:              recipeID,
		OriginalContributorID: attribution.OriginalContributorID,
		OriginalContributor:   contributor.Username,
		ImportDate:            attribution.ImportDate,
		PreserveAttribution:   attribution.PreserveAttribution,
		Customizations:        attribution.Customizations,
		CommunityMetrics:      *communityMetrics,
		RecipeChain:          chain,
		EngagementStats:      engagementStats,
	}, nil
}

func (s *AttributionService) GetContributorProfile(ctx context.Context, contributorID string) (*ContributorProfile, error) {
	// Convert string ID to UUID
	userUUID, err := uuid.Parse(contributorID)
	if err != nil {
		return nil, fmt.Errorf("invalid contributor ID format: %w", err)
	}
	
	user, err := s.userRepo.GetByID(userUUID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user: %w", err)
	}

	// Get contributor stats
	stats, err := s.attributionRepo.GetContributorStats(ctx, contributorID)
	if err != nil {
		return nil, fmt.Errorf("failed to get contributor stats: %w", err)
	}

	// Get badges and achievements
	badges, err := s.attributionRepo.GetContributorBadges(ctx, contributorID)
	if err != nil {
		fmt.Printf("Warning: failed to get contributor badges: %v\n", err)
		badges = []ContributorBadge{}
	}

	achievements, err := s.attributionRepo.GetContributorAchievements(ctx, contributorID)
	if err != nil {
		fmt.Printf("Warning: failed to get contributor achievements: %v\n", err)
		achievements = []ContributorAchievement{}
	}

	avatarURL := &user.AvatarURL
	
	return &ContributorProfile{
		ID:            user.ID.String(),
		Username:      user.Username,
		DisplayName:   user.DisplayName,
		AvatarURL:     avatarURL,
		TotalRecipes:  stats.TotalRecipes,
		AverageRating: stats.AverageRating,
		TotalImports:  stats.TotalImports,
		JoinedAt:      user.CreatedAt,
		Badges:        badges,
		Achievements:  achievements,
		Bio:           user.Bio,
		Location:      user.Location,
		Website:       user.Website,
	}, nil
}

func (s *AttributionService) GetRecipeMetrics(ctx context.Context, recipeID string, timeframe MetricsTimeframe) (*CommunityMetricsData, error) {
	// Get basic metrics
	overview, err := s.metricsRepo.GetRecipeOverview(ctx, recipeID, string(timeframe))
	if err != nil {
		return nil, fmt.Errorf("failed to get recipe overview: %w", err)
	}

	// Get popularity metrics
	popularity, err := s.metricsRepo.GetRecipePopularity(ctx, recipeID, string(timeframe))
	if err != nil {
		fmt.Printf("Warning: failed to get popularity metrics: %v\n", err)
	}

	// Get engagement metrics
	engagement, err := s.metricsRepo.GetRecipeEngagement(ctx, recipeID, string(timeframe))
	if err != nil {
		fmt.Printf("Warning: failed to get engagement metrics: %v\n", err)
	}

	// Get geographic metrics
	geographic, err := s.metricsRepo.GetRecipeGeographic(ctx, recipeID, string(timeframe))
	if err != nil {
		fmt.Printf("Warning: failed to get geographic metrics: %v\n", err)
	}

	return &CommunityMetricsData{
		Overview:   overview,
		Popularity: popularity,
		Engagement: engagement,
		Geographic: geographic,
	}, nil
}

func (s *AttributionService) GetContributorMetrics(ctx context.Context, contributorID string, timeframe MetricsTimeframe) (*CommunityMetricsData, error) {
	// Get contributor overview
	overview, err := s.metricsRepo.GetContributorOverview(ctx, contributorID, string(timeframe))
	if err != nil {
		return nil, fmt.Errorf("failed to get contributor overview: %w", err)
	}

	// Get achievements
	achievements, err := s.attributionRepo.GetContributorAchievements(ctx, contributorID)
	if err != nil {
		fmt.Printf("Warning: failed to get achievements: %v\n", err)
	}

	// Convert contributor overview to metrics overview format
	metricsOverview := &MetricsOverview{
		TotalViews:   overview.TotalViews,
		TotalImports: overview.TotalImports,
		TotalRatings: overview.TotalRecipes,
		AvgRating:    overview.AverageRating,
	}
	
	return &CommunityMetricsData{
		Overview:     metricsOverview,
		Achievements: achievements,
	}, nil
}

func (s *AttributionService) AwardAchievement(ctx context.Context, contributorID, achievementType string, metadata map[string]interface{}) error {
	achievement := s.createAchievement(achievementType, metadata)
	if achievement == nil {
		return fmt.Errorf("unknown achievement type: %s", achievementType)
	}

	return s.attributionRepo.CreateAchievement(ctx, contributorID, achievement)
}

// Helper methods

func (s *AttributionService) buildRecipeChain(ctx context.Context, recipeID string) ([]RecipeChainLink, error) {
	chain, err := s.attributionRepo.GetRecipeChain(ctx, recipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get recipe chain: %w", err)
	}

	var links []RecipeChainLink
	for _, link := range chain {
		// Convert string ID to UUID
		contributorUUID, err := uuid.Parse(link.ContributorID)
		if err != nil {
			fmt.Printf("Warning: invalid contributor ID %s: %v\n", link.ContributorID, err)
			continue
		}
		
		contributor, err := s.userRepo.GetByID(contributorUUID)
		if err != nil {
			fmt.Printf("Warning: failed to get contributor %s: %v\n", link.ContributorID, err)
			continue
		}

		links = append(links, RecipeChainLink{
			ContributorID:   link.ContributorID,
			ContributorName: contributor.Username,
			Contribution:    link.Contribution,
			Timestamp:       link.Timestamp,
		})
	}

	return links, nil
}

func (s *AttributionService) getCommunityMetrics(ctx context.Context, recipeID string) (*CommunityMetrics, error) {
	// TODO: Implement full community metrics retrieval
	// For now, return default values
	return &CommunityMetrics{
		TotalImports:  0,
		AverageRating: 0.0,
		TotalRatings:  0,
		TrendingScore: 0.0,
	}, nil
}

func (s *AttributionService) getEngagementStats(ctx context.Context, recipeID string) (*EngagementStats, error) {
	// TODO: Implement full engagement stats retrieval
	// For now, return default values
	return &EngagementStats{
		Views:    0,
		Likes:    0,
		Saves:    0,
		Shares:   0,
		Comments: 0,
	}, nil
}

func (s *AttributionService) createAchievement(achievementType string, metadata map[string]interface{}) *ContributorAchievement {
	achievements := map[string]ContributorAchievement{
		"first_recipe": {
			Name:        "First Recipe",
			Description: "Shared your first recipe with the community",
			Category:    "milestone",
			Progress:    1,
			MaxProgress: 1,
		},
		"popular_recipe": {
			Name:        "Popular Recipe",
			Description: "One of your recipes reached 100+ imports",
			Category:    "popularity", 
			Progress:    1,
			MaxProgress: 1,
		},
		"community_favorite": {
			Name:        "Community Favorite",
			Description: "Recipe voted as community favorite",
			Category:    "recognition",
			Progress:    1,
			MaxProgress: 1,
		},
		"prolific_creator": {
			Name:        "Prolific Creator", 
			Description: "Shared 50+ recipes with the community",
			Category:    "milestone",
			Progress:    50,
			MaxProgress: 50,
		},
		"trending_chef": {
			Name:        "Trending Chef",
			Description: "Had a recipe trending for a full week",
			Category:    "trending",
			Progress:    1,
			MaxProgress: 1,
		},
	}

	achievement, exists := achievements[achievementType]
	if !exists {
		return nil
	}

	achievement.ID = fmt.Sprintf("%s_%d", achievementType, time.Now().Unix())
	unlockedAt := time.Now()
	achievement.UnlockedAt = &unlockedAt

	return &achievement
}

func extractCustomizations(customizations *models.RecipeCustomizations) []string {
	if customizations == nil {
		return []string{}
	}

	var changes []string
	if customizations.CustomTitle != nil {
		changes = append(changes, "title")
	}
	if customizations.CustomNotes != nil {
		changes = append(changes, "notes")
	}
	if customizations.ServingAdjustment != nil {
		changes = append(changes, "servings")
	}

	return changes
}

func extractRecipeImportCustomizations(customizations *models.RecipeImportCustomizations) []string {
	if customizations == nil {
		return []string{}
	}

	var changes []string
	if customizations.Title != nil {
		changes = append(changes, "title")
	}
	if customizations.Notes != nil {
		changes = append(changes, "notes")
	}
	if customizations.ServingAdjustment != nil {
		changes = append(changes, "servings")
	}

	return changes
}