package services

import (
	"database/sql"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/repositories"
)

var (
	ErrDuplicateRating   = errors.New("user has already rated this recipe")
	ErrRecipeNotFound    = errors.New("recipe not found")
	ErrRatingNotFound    = errors.New("rating not found")
	ErrInvalidRating     = errors.New("rating must be between 1 and 5 stars")
	ErrUnauthorized      = errors.New("unauthorized to perform this action")
	ErrModerationFailed  = errors.New("content moderation failed")
)

// RecipeRatingService handles business logic for recipe ratings
type RecipeRatingService struct {
	ratingRepo     *repositories.RecipeRatingRepository
	moderationSvc  *ContentModerationService
	db             *sql.DB
}

// RatingSubmissionRequest represents a rating submission
type RatingSubmissionRequest struct {
	RecipeID         uuid.UUID `json:"recipeId"`
	OverallRating    int       `json:"overallRating"`
	DifficultyRating *int      `json:"difficultyRating,omitempty"`
	TasteRating      *int      `json:"tasteRating,omitempty"`
	ReviewText       *string   `json:"reviewText,omitempty"`
	WouldMakeAgain   *bool     `json:"wouldMakeAgain,omitempty"`
	ActualPrepTime   *int      `json:"actualPrepTime,omitempty"`
	ActualCookTime   *int      `json:"actualCookTime,omitempty"`
	CookingContext   *string   `json:"cookingContext,omitempty"`
}

// RatingUpdateRequest represents a rating update
type RatingUpdateRequest struct {
	OverallRating    *int    `json:"overallRating,omitempty"`
	DifficultyRating *int    `json:"difficultyRating,omitempty"`
	TasteRating      *int    `json:"tasteRating,omitempty"`
	ReviewText       *string `json:"reviewText,omitempty"`
	WouldMakeAgain   *bool   `json:"wouldMakeAgain,omitempty"`
	ActualPrepTime   *int    `json:"actualPrepTime,omitempty"`
	ActualCookTime   *int    `json:"actualCookTime,omitempty"`
	CookingContext   *string `json:"cookingContext,omitempty"`
}

// PaginatedRatingsResponse represents paginated ratings response
type PaginatedRatingsResponse struct {
	Ratings    []*repositories.RecipeRating         `json:"ratings"`
	Pagination *PaginationInfo                      `json:"pagination"`
	Aggregates *repositories.RatingsAggregates      `json:"aggregates"`
}

// PaginationInfo represents pagination metadata
type RatingPaginationInfo struct {
	Total       int  `json:"total"`
	Page        int  `json:"page"`
	Limit       int  `json:"limit"`
	HasNext     bool `json:"hasNext"`
	HasPrevious bool `json:"hasPrevious"`
}

// NewRecipeRatingService creates a new recipe rating service
func NewRecipeRatingService(db *sql.DB) *RecipeRatingService {
	return &RecipeRatingService{
		ratingRepo:    repositories.NewRecipeRatingRepository(db),
		moderationSvc: NewContentModerationService(),
		db:            db,
	}
}

// SubmitRating submits a new recipe rating
func (s *RecipeRatingService) SubmitRating(userID uuid.UUID, req *RatingSubmissionRequest) (*repositories.RecipeRating, error) {
	// Validate rating values
	if err := s.validateRatingValues(req.OverallRating, req.DifficultyRating, req.TasteRating); err != nil {
		return nil, err
	}

	// Check if recipe exists
	if err := s.validateRecipeExists(req.RecipeID); err != nil {
		return nil, err
	}

	// Check for duplicate rating
	existingRating, err := s.ratingRepo.GetRatingByUserAndRecipe(userID, req.RecipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to check existing rating: %w", err)
	}
	if existingRating != nil {
		return nil, ErrDuplicateRating
	}

	// Moderate review text if provided
	var moderationResult *ModerationResult
	if req.ReviewText != nil && *req.ReviewText != "" {
		moderationResult = s.moderationSvc.ModerateReviewText(*req.ReviewText)
	}

	// Create rating entity
	rating := &repositories.RecipeRating{
		ID:               uuid.New(),
		RecipeID:         req.RecipeID,
		UserID:           userID,
		OverallRating:    req.OverallRating,
		DifficultyRating: req.DifficultyRating,
		TasteRating:      req.TasteRating,
		WouldMakeAgain:   req.WouldMakeAgain,
		ActualPrepTime:   req.ActualPrepTime,
		ActualCookTime:   req.ActualCookTime,
		CookingContext:   req.CookingContext,
		ModerationStatus: "approved",
		CreatedAt:        time.Now(),
		UpdatedAt:        time.Now(),
	}

	// Set review text and moderation status
	if moderationResult != nil {
		sanitizedText := moderationResult.SanitizedContent
		rating.ReviewText = &sanitizedText
		rating.ModerationStatus = moderationResult.ModerationStatus
		if moderationResult.FlaggedReason != "" {
			rating.FlaggedReason = &moderationResult.FlaggedReason
		}
	}

	// Save rating
	if err := s.ratingRepo.CreateRating(rating); err != nil {
		return nil, fmt.Errorf("failed to create rating: %w", err)
	}

	return rating, nil
}

// UpdateRating updates an existing recipe rating
func (s *RecipeRatingService) UpdateRating(userID, ratingID uuid.UUID, req *RatingUpdateRequest) (*repositories.RecipeRating, error) {
	// Get existing rating
	existingRating, err := s.ratingRepo.GetRatingByUserAndRecipe(userID, ratingID)
	if err != nil {
		return nil, fmt.Errorf("failed to get existing rating: %w", err)
	}
	if existingRating == nil {
		return nil, ErrRatingNotFound
	}

	// Validate rating values if provided
	if req.OverallRating != nil || req.DifficultyRating != nil || req.TasteRating != nil {
		overallRating := existingRating.OverallRating
		if req.OverallRating != nil {
			overallRating = *req.OverallRating
		}
		if err := s.validateRatingValues(overallRating, req.DifficultyRating, req.TasteRating); err != nil {
			return nil, err
		}
	}

	// Prepare update map
	updates := make(map[string]interface{})
	
	if req.OverallRating != nil {
		updates["overall_rating"] = *req.OverallRating
	}
	if req.DifficultyRating != nil {
		updates["difficulty_rating"] = *req.DifficultyRating
	}
	if req.TasteRating != nil {
		updates["taste_rating"] = *req.TasteRating
	}
	if req.WouldMakeAgain != nil {
		updates["would_make_again"] = *req.WouldMakeAgain
	}
	if req.ActualPrepTime != nil {
		updates["actual_prep_time"] = *req.ActualPrepTime
	}
	if req.ActualCookTime != nil {
		updates["actual_cook_time"] = *req.ActualCookTime
	}
	if req.CookingContext != nil {
		updates["cooking_context"] = *req.CookingContext
	}

	// Handle review text update with moderation
	if req.ReviewText != nil {
		var moderationResult *ModerationResult
		if *req.ReviewText != "" {
			moderationResult = s.moderationSvc.ModerateReviewText(*req.ReviewText)
			updates["review_text"] = moderationResult.SanitizedContent
			updates["moderation_status"] = moderationResult.ModerationStatus
			if moderationResult.FlaggedReason != "" {
				updates["flagged_reason"] = moderationResult.FlaggedReason
			}
		} else {
			updates["review_text"] = nil
			updates["moderation_status"] = "approved"
			updates["flagged_reason"] = nil
		}
	}

	// Update rating
	if err := s.ratingRepo.UpdateRating(existingRating.ID, updates); err != nil {
		return nil, fmt.Errorf("failed to update rating: %w", err)
	}

	// Return updated rating
	return s.ratingRepo.GetRatingByUserAndRecipe(userID, existingRating.RecipeID)
}

// GetRatingsByRecipe gets all ratings for a recipe with pagination
func (s *RecipeRatingService) GetRatingsByRecipe(recipeID uuid.UUID, page, limit int) (*PaginatedRatingsResponse, error) {
	offset := (page - 1) * limit
	
	// Get ratings
	ratings, totalCount, err := s.ratingRepo.GetRatingsByRecipe(recipeID, limit, offset, "approved")
	if err != nil {
		return nil, fmt.Errorf("failed to get ratings: %w", err)
	}

	// Get aggregates
	aggregates, err := s.ratingRepo.GetRatingsAggregatesByRecipe(recipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get rating aggregates: %w", err)
	}

	// Build pagination info
	pagination := &PaginationInfo{
		Total:       totalCount,
		Page:        page,
		Limit:       limit,
		HasNext:     offset+limit < totalCount,
		HasPrevious: page > 1,
	}

	return &PaginatedRatingsResponse{
		Ratings:    ratings,
		Pagination: pagination,
		Aggregates: aggregates,
	}, nil
}

// GetUserRatingHistory gets a user's rating history with pagination
func (s *RecipeRatingService) GetUserRatingHistory(userID uuid.UUID, page, limit int) (*PaginatedRatingsResponse, error) {
	offset := (page - 1) * limit
	
	ratings, totalCount, err := s.ratingRepo.GetRatingsByUser(userID, limit, offset)
	if err != nil {
		return nil, fmt.Errorf("failed to get user ratings: %w", err)
	}

	pagination := &PaginationInfo{
		Total:       totalCount,
		Page:        page,
		Limit:       limit,
		HasNext:     offset+limit < totalCount,
		HasPrevious: page > 1,
	}

	return &PaginatedRatingsResponse{
		Ratings:    ratings,
		Pagination: pagination,
	}, nil
}

// GetUserRatingForRecipe gets a user's rating for a specific recipe
func (s *RecipeRatingService) GetUserRatingForRecipe(userID, recipeID uuid.UUID) (*repositories.RecipeRating, error) {
	return s.ratingRepo.GetRatingByUserAndRecipe(userID, recipeID)
}

// DeleteRating soft deletes a user's rating
func (s *RecipeRatingService) DeleteRating(userID, recipeID uuid.UUID) error {
	// Get existing rating
	rating, err := s.ratingRepo.GetRatingByUserAndRecipe(userID, recipeID)
	if err != nil {
		return fmt.Errorf("failed to get rating: %w", err)
	}
	if rating == nil {
		return ErrRatingNotFound
	}

	// Soft delete rating
	return s.ratingRepo.DeleteRating(rating.ID)
}

// FlagRating flags a rating for moderation review
func (s *RecipeRatingService) FlagRating(ratingID uuid.UUID, reason string) error {
	return s.ratingRepo.FlagRating(ratingID, reason)
}

// GetPendingModerationRatings gets ratings pending moderation (admin only)
func (s *RecipeRatingService) GetPendingModerationRatings(page, limit int) (*PaginatedRatingsResponse, error) {
	offset := (page - 1) * limit
	
	ratings, totalCount, err := s.ratingRepo.GetPendingModerationRatings(limit, offset)
	if err != nil {
		return nil, fmt.Errorf("failed to get pending ratings: %w", err)
	}

	pagination := &PaginationInfo{
		Total:       totalCount,
		Page:        page,
		Limit:       limit,
		HasNext:     offset+limit < totalCount,
		HasPrevious: page > 1,
	}

	return &PaginatedRatingsResponse{
		Ratings:    ratings,
		Pagination: pagination,
	}, nil
}

// validateRatingValues validates rating values
func (s *RecipeRatingService) validateRatingValues(overallRating int, difficultyRating, tasteRating *int) error {
	if overallRating < 1 || overallRating > 5 {
		return ErrInvalidRating
	}
	if difficultyRating != nil && (*difficultyRating < 1 || *difficultyRating > 5) {
		return ErrInvalidRating
	}
	if tasteRating != nil && (*tasteRating < 1 || *tasteRating > 5) {
		return ErrInvalidRating
	}
	return nil
}

// validateRecipeExists checks if a recipe exists
func (s *RecipeRatingService) validateRecipeExists(recipeID uuid.UUID) error {
	query := `SELECT 1 FROM recipes WHERE id = $1 AND deleted_at IS NULL`
	var exists int
	err := s.db.QueryRow(query, recipeID).Scan(&exists)
	if err == sql.ErrNoRows {
		return ErrRecipeNotFound
	}
	return err
}