package services

import (
	"context"
	"fmt"
	"regexp"
	"strings"
	"time"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

type TagManagementService struct {
	tagRepo    *repositories.TagManagementRepository
	recipeRepo *repositories.RecipeRepository
	aiService  *AIContentService // For intelligent tag suggestions
}

func NewTagManagementService(tagRepo *repositories.TagManagementRepository, recipeRepo *repositories.RecipeRepository, aiService *AIContentService) *TagManagementService {
	return &TagManagementService{
		tagRepo:    tagRepo,
		recipeRepo: recipeRepo,
		aiService:  aiService,
	}
}

type TagManagementSuggestion struct {
	Tag        string  `json:"tag"`
	Confidence float64 `json:"confidence"`
	UsageCount int     `json:"usage_count"`
	Category   string  `json:"category"`
}

type PopularTag struct {
	Tag         string `json:"tag"`
	UsageCount  int    `json:"usage_count"`
	Category    string `json:"category"`
	TrendingUp  bool   `json:"trending_up"`
	Description string `json:"description,omitempty"`
}

type CommunityTag struct {
	Tag        string  `json:"tag"`
	VoteCount  int     `json:"vote_count"`
	UserVoted  bool    `json:"user_voted"`
	Confidence float64 `json:"confidence"`
}

type TagStat struct {
	UsageCount int     `json:"usage_count"`
	Trending   bool    `json:"trending"`
	Category   string  `json:"category"`
	Confidence float64 `json:"confidence"`
}

type InvalidTagResult struct {
	Tag    string `json:"tag"`
	Reason string `json:"reason"`
}

var (
	validTagPattern = regexp.MustCompile(`^[a-z0-9_\-\s]+$`)
	bannedWords     = []string{"spam", "fake", "test", "admin", "bot", "system"}
)

func (s *TagManagementService) GetTagSuggestions(userID, query, recipeID string, exclude []string, limit int) ([]TagManagementSuggestion, error) {
	ctx := context.Background()
	
	// Get recipe context for intelligent suggestions
	var recipeContext *models.Recipe
	var err error
	if recipeID != "" {
		recipeContext, err = s.recipeRepo.GetByID(ctx, recipeID)
		if err != nil {
			return nil, fmt.Errorf("failed to get recipe context: %w", err)
		}
	}

	// Get database suggestions based on existing tags
	dbSuggestions, err := s.tagRepo.GetTagSuggestions(ctx, query, exclude, limit)
	if err != nil {
		return nil, fmt.Errorf("failed to get database tag suggestions: %w", err)
	}

	// Get AI-powered suggestions if recipe context is available
	var aiSuggestions []TagManagementSuggestion
	if recipeContext != nil {
		aiSuggestions, err = s.getAITagSuggestions(recipeContext, query, exclude)
		if err != nil {
			// Log error but don't fail the request
			fmt.Printf("Warning: AI tag suggestions failed: %v\n", err)
		}
	}

	// Merge and rank suggestions
	merged := s.mergeTagSuggestions(dbSuggestions, aiSuggestions, limit)
	
	// Filter out banned or invalid suggestions
	var filtered []TagManagementSuggestion
	for _, suggestion := range merged {
		if s.isValidTag(suggestion.Tag) {
			filtered = append(filtered, suggestion)
		}
	}

	if len(filtered) > limit {
		filtered = filtered[:limit]
	}

	return filtered, nil
}

func (s *TagManagementService) GetPopularTags(userID string, limit int, categoryFilter, timePeriod string) ([]PopularTag, error) {
	ctx := context.Background()
	
	var since *time.Time
	switch timePeriod {
	case "day":
		t := time.Now().AddDate(0, 0, -1)
		since = &t
	case "week":
		t := time.Now().AddDate(0, 0, -7)
		since = &t
	case "month":
		t := time.Now().AddDate(0, -1, 0)
		since = &t
	default:
		// "all" - no time filter
	}

	tags, err := s.tagRepo.GetPopularTags(ctx, limit, categoryFilter, since)
	if err != nil {
		return nil, fmt.Errorf("failed to get popular tags: %w", err)
	}

	// Add trending indicators
	for i := range tags {
		trending, err := s.tagRepo.IsTagTrending(ctx, tags[i].Tag, time.Now().AddDate(0, 0, -7))
		if err != nil {
			// Log warning but continue
			fmt.Printf("Warning: failed to check trending status for tag %s: %v\n", tags[i].Tag, err)
		}
		tags[i].TrendingUp = trending
	}

	return tags, nil
}

func (s *TagManagementService) ValidateTags(userID string, tags []string) ([]string, []InvalidTagResult, error) {
	var validTags []string
	var invalidTags []InvalidTagResult

	for _, tag := range tags {
		cleaned := strings.TrimSpace(strings.ToLower(tag))
		if cleaned == "" {
			invalidTags = append(invalidTags, InvalidTagResult{
				Tag:    tag,
				Reason: "Tag cannot be empty",
			})
			continue
		}

		if len(cleaned) > 30 {
			invalidTags = append(invalidTags, InvalidTagResult{
				Tag:    tag,
				Reason: "Tag too long (max 30 characters)",
			})
			continue
		}

		if len(cleaned) < 2 {
			invalidTags = append(invalidTags, InvalidTagResult{
				Tag:    tag,
				Reason: "Tag too short (min 2 characters)",
			})
			continue
		}

		if !s.isValidTag(cleaned) {
			invalidTags = append(invalidTags, InvalidTagResult{
				Tag:    tag,
				Reason: "Tag contains invalid characters or banned words",
			})
			continue
		}

		validTags = append(validTags, cleaned)
	}

	return validTags, invalidTags, nil
}

func (s *TagManagementService) UpdateRecipeTags(userID, recipeID string, tags []string, action string) ([]string, error) {
	ctx := context.Background()

	// Verify user owns the recipe
	recipe, err := s.recipeRepo.GetByID(ctx, recipeID)
	if err != nil {
		return nil, fmt.Errorf("recipe not found: %w", err)
	}

	if recipe.UserID != userID {
		return nil, fmt.Errorf("permission denied: user does not own this recipe")
	}

	// Validate tags
	validTags, invalidTags, err := s.ValidateTags(userID, tags)
	if err != nil {
		return nil, fmt.Errorf("failed to validate tags: %w", err)
	}

	if len(invalidTags) > 0 {
		return nil, fmt.Errorf("invalid tags provided: %v", invalidTags)
	}

	// Apply the action
	var updatedTags []string
	switch action {
	case "add":
		updatedTags, err = s.tagRepo.AddTagsToRecipe(ctx, recipeID, validTags)
	case "remove":
		updatedTags, err = s.tagRepo.RemoveTagsFromRecipe(ctx, recipeID, validTags)
	case "replace":
		updatedTags, err = s.tagRepo.ReplaceRecipeTags(ctx, recipeID, validTags)
	default:
		return nil, fmt.Errorf("invalid action: %s", action)
	}

	if err != nil {
		return nil, fmt.Errorf("failed to update recipe tags: %w", err)
	}

	// Update tag usage statistics
	go func() {
		if err := s.tagRepo.UpdateTagUsageStats(context.Background(), validTags); err != nil {
			fmt.Printf("Warning: failed to update tag usage stats: %v\n", err)
		}
	}()

	return updatedTags, nil
}

func (s *TagManagementService) GetRecipeTags(userID, recipeID string) ([]string, []CommunityTag, map[string]TagStat, error) {
	ctx := context.Background()

	// Get recipe and verify access
	recipe, err := s.recipeRepo.GetByID(ctx, recipeID)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("recipe not found: %w", err)
	}

	// Get user tags (recipe owner's tags)
	userTags, err := s.tagRepo.GetRecipeUserTags(ctx, recipeID)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to get user tags: %w", err)
	}

	// Get community tags (if recipe is public)
	var communityTags []CommunityTag
	if recipe.IsPublic {
		communityTags, err = s.tagRepo.GetRecipeCommunityTags(ctx, recipeID, userID)
		if err != nil {
			return nil, nil, nil, fmt.Errorf("failed to get community tags: %w", err)
		}
	}

	// Get tag statistics
	allTags := make([]string, 0, len(userTags)+len(communityTags))
	allTags = append(allTags, userTags...)
	for _, ct := range communityTags {
		allTags = append(allTags, ct.Tag)
	}

	tagStats, err := s.tagRepo.GetTagStats(ctx, allTags)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to get tag stats: %w", err)
	}

	return userTags, communityTags, tagStats, nil
}

func (s *TagManagementService) VoteOnTag(userID, recipeID, tag, action string) (int, bool, error) {
	ctx := context.Background()

	// Verify recipe exists and is public
	recipe, err := s.recipeRepo.GetByID(ctx, recipeID)
	if err != nil {
		return 0, false, fmt.Errorf("recipe not found: %w", err)
	}

	if !recipe.IsPublic {
		return 0, false, fmt.Errorf("cannot vote on tags for private recipes")
	}

	// Validate tag
	if !s.isValidTag(tag) {
		return 0, false, fmt.Errorf("invalid tag: %s", tag)
	}

	// Apply vote
	var voteCount int
	var userVoted bool
	switch action {
	case "upvote":
		voteCount, userVoted, err = s.tagRepo.UpvoteTag(ctx, recipeID, userID, tag)
	case "downvote":
		voteCount, userVoted, err = s.tagRepo.DownvoteTag(ctx, recipeID, userID, tag)
	case "remove":
		voteCount, userVoted, err = s.tagRepo.RemoveVote(ctx, recipeID, userID, tag)
	default:
		return 0, false, fmt.Errorf("invalid vote action: %s", action)
	}

	if err != nil {
		return 0, false, fmt.Errorf("failed to vote on tag: %w", err)
	}

	// Update tag confidence score based on votes
	go func() {
		if err := s.tagRepo.UpdateTagConfidence(context.Background(), recipeID, tag); err != nil {
			fmt.Printf("Warning: failed to update tag confidence: %v\n", err)
		}
	}()

	return voteCount, userVoted, nil
}

// Helper methods

func (s *TagManagementService) isValidTag(tag string) bool {
	if tag == "" || len(tag) < 2 || len(tag) > 30 {
		return false
	}

	// Check for valid characters (alphanumeric, underscores, hyphens, spaces)
	if !validTagPattern.MatchString(tag) {
		return false
	}

	// Check for banned words
	loweredTag := strings.ToLower(tag)
	for _, banned := range bannedWords {
		if strings.Contains(loweredTag, banned) {
			return false
		}
	}

	// Check for reasonable content (not just special characters)
	if strings.Trim(tag, "_- ") == "" {
		return false
	}

	return true
}

func (s *TagManagementService) getAITagSuggestions(recipe *models.Recipe, query string, exclude []string) ([]TagManagementSuggestion, error) {
	if s.aiService == nil {
		return nil, nil
	}

	// Create recipe context for AI analysis
	context := fmt.Sprintf(
		"Recipe: %s\nDescription: %s\nIngredients: %v\nMeal Types: %v\nComplexity: %s\nCuisine: %s",
		recipe.Title,
		recipe.Description,
		recipe.Ingredients,
		recipe.MealType,
		recipe.Complexity,
		recipe.CuisineType,
	)

	// Request AI tag suggestions
	suggestions, err := s.aiService.GenerateTagSuggestions(context, query, exclude)
	if err != nil {
		return nil, fmt.Errorf("AI tag suggestion failed: %w", err)
	}

	// Convert to our format and validate
	var validSuggestions []TagManagementSuggestion
	for _, suggestion := range suggestions {
		if s.isValidTag(suggestion.Tag) {
			validSuggestions = append(validSuggestions, TagManagementSuggestion{
				Tag:        suggestion.Tag,
				Confidence: suggestion.Confidence,
				UsageCount: 0, // AI suggestions don't have usage count
				Category:   suggestion.Category,
			})
		}
	}

	return validSuggestions, nil
}

func (s *TagManagementService) mergeTagSuggestions(dbSuggestions, aiSuggestions []TagManagementSuggestion, limit int) []TagManagementSuggestion {
	// Create a map to merge suggestions and avoid duplicates
	suggestionMap := make(map[string]TagManagementSuggestion)

	// Add database suggestions (prioritize higher usage count)
	for _, suggestion := range dbSuggestions {
		suggestionMap[suggestion.Tag] = suggestion
	}

	// Add AI suggestions (if not already present or if higher confidence)
	for _, suggestion := range aiSuggestions {
		existing, exists := suggestionMap[suggestion.Tag]
		if !exists || suggestion.Confidence > existing.Confidence {
			// Keep usage count from database if available
			if exists {
				suggestion.UsageCount = existing.UsageCount
			}
			suggestionMap[suggestion.Tag] = suggestion
		}
	}

	// Convert back to slice and sort by combined score
	var merged []TagManagementSuggestion
	for _, suggestion := range suggestionMap {
		// Calculate combined score: confidence + usage_count normalized
		score := suggestion.Confidence + float64(suggestion.UsageCount)*0.01
		suggestion.Confidence = score
		merged = append(merged, suggestion)
	}

	// Sort by score descending
	for i := 0; i < len(merged)-1; i++ {
		for j := i + 1; j < len(merged); j++ {
			if merged[i].Confidence < merged[j].Confidence {
				merged[i], merged[j] = merged[j], merged[i]
			}
		}
	}

	if len(merged) > limit {
		merged = merged[:limit]
	}

	return merged
}