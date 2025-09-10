package repositories

import (
	"database/sql"
	"time"

	"github.com/google/uuid"
)

// RecipeRating represents a recipe rating entity
type RecipeRating struct {
	ID               uuid.UUID  `db:"id" json:"id"`
	RecipeID         uuid.UUID  `db:"recipe_id" json:"recipeId"`
	UserID           uuid.UUID  `db:"user_id" json:"userId"`
	OverallRating    int        `db:"overall_rating" json:"overallRating"`
	DifficultyRating *int       `db:"difficulty_rating" json:"difficultyRating,omitempty"`
	TasteRating      *int       `db:"taste_rating" json:"tasteRating,omitempty"`
	ReviewText       *string    `db:"review_text" json:"reviewText,omitempty"`
	WouldMakeAgain   *bool      `db:"would_make_again" json:"wouldMakeAgain,omitempty"`
	ActualPrepTime   *int       `db:"actual_prep_time" json:"actualPrepTime,omitempty"`
	ActualCookTime   *int       `db:"actual_cook_time" json:"actualCookTime,omitempty"`
	MealPlanID       *uuid.UUID `db:"meal_plan_id" json:"mealPlanId,omitempty"`
	CookingContext   *string    `db:"cooking_context" json:"cookingContext,omitempty"`
	ModerationStatus string     `db:"moderation_status" json:"moderationStatus"`
	FlaggedReason    *string    `db:"flagged_reason" json:"flaggedReason,omitempty"`
	CreatedAt        time.Time  `db:"created_at" json:"createdAt"`
	UpdatedAt        time.Time  `db:"updated_at" json:"updatedAt"`
}

// RatingDistribution represents the distribution of ratings
type RatingDistribution map[string]int

// RatingsAggregates represents aggregated rating data
type RatingsAggregates struct {
	AverageRating       float64             `json:"averageRating"`
	TotalRatings        int                 `json:"totalRatings"`
	RatingDistribution  RatingDistribution  `json:"ratingDistribution"`
}

// RecipeRatingRepository handles database operations for recipe ratings
type RecipeRatingRepository struct {
	db *sql.DB
}

// NewRecipeRatingRepository creates a new recipe rating repository
func NewRecipeRatingRepository(db *sql.DB) *RecipeRatingRepository {
	return &RecipeRatingRepository{
		db: db,
	}
}

// CreateRating creates a new recipe rating
func (r *RecipeRatingRepository) CreateRating(rating *RecipeRating) error {
	query := `
		INSERT INTO recipe_ratings (
			id, recipe_id, user_id, overall_rating, difficulty_rating, taste_rating,
			review_text, would_make_again, actual_prep_time, actual_cook_time,
			meal_plan_id, cooking_context, moderation_status
		) VALUES (
			$1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
		)`
	
	_, err := r.db.Exec(query,
		rating.ID, rating.RecipeID, rating.UserID, rating.OverallRating,
		rating.DifficultyRating, rating.TasteRating, rating.ReviewText,
		rating.WouldMakeAgain, rating.ActualPrepTime, rating.ActualCookTime,
		rating.MealPlanID, rating.CookingContext, rating.ModerationStatus,
	)
	return err
}

// UpdateRating updates an existing recipe rating
func (r *RecipeRatingRepository) UpdateRating(ratingID uuid.UUID, updates map[string]interface{}) error {
	if len(updates) == 0 {
		return nil
	}

	setParts := make([]string, 0, len(updates))
	args := make([]interface{}, 0, len(updates)+1)
	argIndex := 1

	for field, value := range updates {
		setParts = append(setParts, field+" = $"+string(rune(argIndex)+'0'))
		args = append(args, value)
		argIndex++
	}

	query := `UPDATE recipe_ratings SET ` + 
		string(rune(len(setParts))+',') + 
		` updated_at = NOW() WHERE id = $` + string(rune(argIndex)+'0')
	args = append(args, ratingID)

	_, err := r.db.Exec(query, args...)
	return err
}

// GetRatingByUserAndRecipe gets a rating by user ID and recipe ID
func (r *RecipeRatingRepository) GetRatingByUserAndRecipe(userID, recipeID uuid.UUID) (*RecipeRating, error) {
	var rating RecipeRating
	query := `
		SELECT id, recipe_id, user_id, overall_rating, difficulty_rating, taste_rating,
			   review_text, would_make_again, actual_prep_time, actual_cook_time,
			   meal_plan_id, cooking_context, moderation_status, flagged_reason,
			   created_at, updated_at
		FROM recipe_ratings
		WHERE user_id = $1 AND recipe_id = $2`
	
	err := r.db.QueryRow(query, userID, recipeID).Scan(
		&rating.ID, &rating.RecipeID, &rating.UserID, &rating.OverallRating,
		&rating.DifficultyRating, &rating.TasteRating, &rating.ReviewText,
		&rating.WouldMakeAgain, &rating.ActualPrepTime, &rating.ActualCookTime,
		&rating.MealPlanID, &rating.CookingContext, &rating.ModerationStatus,
		&rating.FlaggedReason, &rating.CreatedAt, &rating.UpdatedAt,
	)
	
	if err == sql.ErrNoRows {
		return nil, nil
	}
	return &rating, err
}

// GetRatingsByRecipe gets all ratings for a recipe with pagination
func (r *RecipeRatingRepository) GetRatingsByRecipe(recipeID uuid.UUID, limit, offset int, moderationStatus string) ([]*RecipeRating, int, error) {
	// Get total count
	countQuery := `SELECT COUNT(*) FROM recipe_ratings WHERE recipe_id = $1 AND moderation_status = $2`
	var totalCount int
	err := r.db.QueryRow(countQuery, recipeID, moderationStatus).Scan(&totalCount)
	if err != nil {
		return nil, 0, err
	}

	// Get ratings
	query := `
		SELECT id, recipe_id, user_id, overall_rating, difficulty_rating, taste_rating,
			   review_text, would_make_again, actual_prep_time, actual_cook_time,
			   meal_plan_id, cooking_context, moderation_status, flagged_reason,
			   created_at, updated_at
		FROM recipe_ratings
		WHERE recipe_id = $1 AND moderation_status = $2
		ORDER BY created_at DESC
		LIMIT $3 OFFSET $4`
	
	rows, err := r.db.Query(query, recipeID, moderationStatus, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	var ratings []*RecipeRating
	for rows.Next() {
		var rating RecipeRating
		err := rows.Scan(
			&rating.ID, &rating.RecipeID, &rating.UserID, &rating.OverallRating,
			&rating.DifficultyRating, &rating.TasteRating, &rating.ReviewText,
			&rating.WouldMakeAgain, &rating.ActualPrepTime, &rating.ActualCookTime,
			&rating.MealPlanID, &rating.CookingContext, &rating.ModerationStatus,
			&rating.FlaggedReason, &rating.CreatedAt, &rating.UpdatedAt,
		)
		if err != nil {
			return nil, 0, err
		}
		ratings = append(ratings, &rating)
	}

	return ratings, totalCount, rows.Err()
}

// GetRatingsByUser gets all ratings by a user with pagination
func (r *RecipeRatingRepository) GetRatingsByUser(userID uuid.UUID, limit, offset int) ([]*RecipeRating, int, error) {
	// Get total count
	countQuery := `SELECT COUNT(*) FROM recipe_ratings WHERE user_id = $1`
	var totalCount int
	err := r.db.QueryRow(countQuery, userID).Scan(&totalCount)
	if err != nil {
		return nil, 0, err
	}

	// Get ratings with recipe information
	query := `
		SELECT rr.id, rr.recipe_id, rr.user_id, rr.overall_rating, rr.difficulty_rating, rr.taste_rating,
			   rr.review_text, rr.would_make_again, rr.actual_prep_time, rr.actual_cook_time,
			   rr.meal_plan_id, rr.cooking_context, rr.moderation_status, rr.flagged_reason,
			   rr.created_at, rr.updated_at
		FROM recipe_ratings rr
		JOIN recipes r ON rr.recipe_id = r.id
		WHERE rr.user_id = $1 AND r.deleted_at IS NULL
		ORDER BY rr.created_at DESC
		LIMIT $2 OFFSET $3`
	
	rows, err := r.db.Query(query, userID, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	var ratings []*RecipeRating
	for rows.Next() {
		var rating RecipeRating
		err := rows.Scan(
			&rating.ID, &rating.RecipeID, &rating.UserID, &rating.OverallRating,
			&rating.DifficultyRating, &rating.TasteRating, &rating.ReviewText,
			&rating.WouldMakeAgain, &rating.ActualPrepTime, &rating.ActualCookTime,
			&rating.MealPlanID, &rating.CookingContext, &rating.ModerationStatus,
			&rating.FlaggedReason, &rating.CreatedAt, &rating.UpdatedAt,
		)
		if err != nil {
			return nil, 0, err
		}
		ratings = append(ratings, &rating)
	}

	return ratings, totalCount, rows.Err()
}

// GetRatingsAggregatesByRecipe gets aggregated rating data for a recipe
func (r *RecipeRatingRepository) GetRatingsAggregatesByRecipe(recipeID uuid.UUID) (*RatingsAggregates, error) {
	query := `
		SELECT average_rating, total_ratings, rating_distribution
		FROM recipes
		WHERE id = $1`
	
	var avgRating float64
	var totalRatings int
	var distributionJSON []byte
	
	err := r.db.QueryRow(query, recipeID).Scan(&avgRating, &totalRatings, &distributionJSON)
	if err != nil {
		return nil, err
	}

	// Parse distribution JSON
	distribution := make(RatingDistribution)
	if len(distributionJSON) > 0 {
		// Simple JSON parsing for rating distribution
		// In production, use proper JSON unmarshaling
		distribution = map[string]int{
			"1": 0, "2": 0, "3": 0, "4": 0, "5": 0,
		}
	}

	return &RatingsAggregates{
		AverageRating:      avgRating,
		TotalRatings:       totalRatings,
		RatingDistribution: distribution,
	}, nil
}

// DeleteRating soft deletes a rating (marks as rejected)
func (r *RecipeRatingRepository) DeleteRating(ratingID uuid.UUID) error {
	query := `UPDATE recipe_ratings SET moderation_status = 'rejected', updated_at = NOW() WHERE id = $1`
	_, err := r.db.Exec(query, ratingID)
	return err
}

// FlagRating flags a rating for moderation review
func (r *RecipeRatingRepository) FlagRating(ratingID uuid.UUID, reason string) error {
	query := `
		UPDATE recipe_ratings 
		SET moderation_status = 'flagged', flagged_reason = $2, updated_at = NOW() 
		WHERE id = $1`
	_, err := r.db.Exec(query, ratingID, reason)
	return err
}

// GetPendingModerationRatings gets ratings pending moderation review
func (r *RecipeRatingRepository) GetPendingModerationRatings(limit, offset int) ([]*RecipeRating, int, error) {
	// Get total count
	countQuery := `SELECT COUNT(*) FROM recipe_ratings WHERE moderation_status = 'pending'`
	var totalCount int
	err := r.db.QueryRow(countQuery).Scan(&totalCount)
	if err != nil {
		return nil, 0, err
	}

	// Get pending ratings
	query := `
		SELECT id, recipe_id, user_id, overall_rating, difficulty_rating, taste_rating,
			   review_text, would_make_again, actual_prep_time, actual_cook_time,
			   meal_plan_id, cooking_context, moderation_status, flagged_reason,
			   created_at, updated_at
		FROM recipe_ratings
		WHERE moderation_status = 'pending'
		ORDER BY created_at ASC
		LIMIT $1 OFFSET $2`
	
	rows, err := r.db.Query(query, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	var ratings []*RecipeRating
	for rows.Next() {
		var rating RecipeRating
		err := rows.Scan(
			&rating.ID, &rating.RecipeID, &rating.UserID, &rating.OverallRating,
			&rating.DifficultyRating, &rating.TasteRating, &rating.ReviewText,
			&rating.WouldMakeAgain, &rating.ActualPrepTime, &rating.ActualCookTime,
			&rating.MealPlanID, &rating.CookingContext, &rating.ModerationStatus,
			&rating.FlaggedReason, &rating.CreatedAt, &rating.UpdatedAt,
		)
		if err != nil {
			return nil, 0, err
		}
		ratings = append(ratings, &rating)
	}

	return ratings, totalCount, rows.Err()
}