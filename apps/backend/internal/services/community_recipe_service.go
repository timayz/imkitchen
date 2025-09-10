package services

import (
	"database/sql"
	"fmt"
	"math"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/repositories"
)

// CommunityRecipeService handles community recipe operations
type CommunityRecipeService struct {
	db              *sql.DB
	recipeRepo      *repositories.RecipeRepository
	ratingRepo      *repositories.RecipeRatingRepository
	importService   *RecipeImportService
	cacheTTL        time.Duration
}

// CommunityRecipe represents a community recipe with rating data
type CommunityRecipe struct {
	ID                      uuid.UUID  `json:"id"`
	Title                   string     `json:"title"`
	Description             *string    `json:"description,omitempty"`
	ImageURL                *string    `json:"imageUrl,omitempty"`
	PrepTime                int        `json:"prepTime"`
	CookTime                int        `json:"cookTime"`
	TotalTime               int        `json:"totalTime"`
	Complexity              string     `json:"complexity"`
	CuisineType             *string    `json:"cuisineType,omitempty"`
	MealType                []string   `json:"mealType"`
	Servings                int        `json:"servings"`
	AverageRating           float64    `json:"averageRating"`
	TotalRatings            int        `json:"totalRatings"`
	RatingDistribution      map[string]int `json:"ratingDistribution"`
	RecommendationScore     float64    `json:"recommendationScore"`
	EligibleForRecommendations bool    `json:"eligibleForRecommendations"`
	ExternalSource          *string    `json:"externalSource,omitempty"`
	CreatedAt               time.Time  `json:"createdAt"`
	UpdatedAt               time.Time  `json:"updatedAt"`
}

// CommunityRecipeFilters represents filters for community recipe search
type CommunityRecipeFilters struct {
	SearchQuery     *string   `json:"searchQuery,omitempty"`
	MinRating       *float64  `json:"minRating,omitempty"`
	MaxPrepTime     *int      `json:"maxPrepTime,omitempty"`
	MealTypes       []string  `json:"mealTypes,omitempty"`
	Complexities    []string  `json:"complexities,omitempty"`
	CuisineTypes    []string  `json:"cuisineTypes,omitempty"`
	SortBy          string    `json:"sortBy"` // rating, recent, popular, trending
	EligibleOnly    bool      `json:"eligibleOnly"` // Only recipes eligible for recommendations
}

// CommunityRecipeResponse represents paginated community recipe response
type CommunityRecipeResponse struct {
	Recipes    []*CommunityRecipe     `json:"recipes"`
	Pagination *PaginationInfo        `json:"pagination"`
	Filters    *CommunityRecipeFilters `json:"filters"`
	Aggregates *CommunityStats        `json:"aggregates"`
}

// CommunityStats represents community recipe statistics
type CommunityStats struct {
	TotalRecipes       int            `json:"totalRecipes"`
	AverageRating      float64        `json:"averageRating"`
	TopCuisines        map[string]int `json:"topCuisines"`
	ComplexityBreakdown map[string]int `json:"complexityBreakdown"`
}

// RecipeImportRequest represents a request to import external recipes
type RecipeImportRequest struct {
	Source      string `json:"source"` // spoonacular, edamam, etc.
	ExternalID  string `json:"externalId"`
	MakePublic  bool   `json:"makePublic"`
	IsCommunity bool   `json:"isCommunity"`
}

// NewCommunityRecipeService creates a new community recipe service
func NewCommunityRecipeService(db *sql.DB, importService *RecipeImportService) *CommunityRecipeService {
	return &CommunityRecipeService{
		db:            db,
		recipeRepo:    repositories.NewRecipeRepository(db),
		ratingRepo:    repositories.NewRecipeRatingRepository(db),
		importService: importService,
		cacheTTL:      time.Hour, // 1 hour cache
	}
}

// GetCommunityRecipes retrieves community recipes with filtering and pagination
func (s *CommunityRecipeService) GetCommunityRecipes(filters *CommunityRecipeFilters, page, limit int) (*CommunityRecipeResponse, error) {
	offset := (page - 1) * limit
	
	// Build query with filters
	query := s.buildCommunityRecipeQuery(filters)
	countQuery := s.buildCommunityRecipeCountQuery(filters)
	
	// Get total count
	var totalCount int
	err := s.db.QueryRow(countQuery).Scan(&totalCount)
	if err != nil {
		return nil, fmt.Errorf("failed to get community recipe count: %w", err)
	}
	
	// Get recipes
	recipes, err := s.executeCommunityRecipeQuery(query, limit, offset)
	if err != nil {
		return nil, fmt.Errorf("failed to get community recipes: %w", err)
	}
	
	// Get community stats
	stats, err := s.getCommunityStats()
	if err != nil {
		return nil, fmt.Errorf("failed to get community stats: %w", err)
	}
	
	// Build pagination info
	pagination := &PaginationInfo{
		Total:       totalCount,
		Page:        page,
		Limit:       limit,
		HasNext:     offset+limit < totalCount,
		HasPrevious: page > 1,
	}
	
	return &CommunityRecipeResponse{
		Recipes:    recipes,
		Pagination: pagination,
		Filters:    filters,
		Aggregates: stats,
	}, nil
}

// GetTrendingRecipes gets trending community recipes based on recent activity
func (s *CommunityRecipeService) GetTrendingRecipes(limit int) ([]*CommunityRecipe, error) {
	// Trending = recipes with recent ratings and high scores
	query := `
		SELECT r.id, r.title, r.description, r.image_url, r.prep_time, r.cook_time, 
			   r.complexity, r.cuisine_type, r.meal_type, r.servings,
			   r.average_rating, r.total_ratings, r.rating_distribution,
			   crr.recommendation_score, crr.eligible_for_recommendations,
			   r.external_source, r.created_at, r.updated_at
		FROM community_recipes_ranked crr
		JOIN recipes r ON crr.id = r.id
		WHERE EXISTS (
			SELECT 1 FROM recipe_ratings rr 
			WHERE rr.recipe_id = r.id 
			AND rr.created_at >= NOW() - INTERVAL '7 days'
			AND rr.moderation_status = 'approved'
		)
		ORDER BY crr.recommendation_score DESC, r.total_ratings DESC
		LIMIT $1`
	
	return s.executeCommunityRecipeQuery(query, limit, 0)
}

// GetHighlyRatedRecipes gets the highest rated community recipes
func (s *CommunityRecipeService) GetHighlyRatedRecipes(minRatings int, limit int) ([]*CommunityRecipe, error) {
	query := `
		SELECT r.id, r.title, r.description, r.image_url, r.prep_time, r.cook_time, 
			   r.complexity, r.cuisine_type, r.meal_type, r.servings,
			   r.average_rating, r.total_ratings, r.rating_distribution,
			   crr.recommendation_score, crr.eligible_for_recommendations,
			   r.external_source, r.created_at, r.updated_at
		FROM community_recipes_ranked crr
		JOIN recipes r ON crr.id = r.id
		WHERE r.total_ratings >= $1
		ORDER BY r.average_rating DESC, r.total_ratings DESC
		LIMIT $2`
	
	return s.executeCommunityRecipeQuery(query, limit, minRatings)
}

// ImportExternalRecipe imports a recipe from external API to community
func (s *CommunityRecipeService) ImportExternalRecipe(req *RecipeImportRequest) (*CommunityRecipe, error) {
	// Check if recipe already exists
	existingQuery := `SELECT id FROM recipes WHERE external_source = $1 AND external_id = $2`
	var existingID uuid.UUID
	err := s.db.QueryRow(existingQuery, req.Source, req.ExternalID).Scan(&existingID)
	if err == nil {
		// Recipe already exists, just update community flags
		return s.updateRecipeCommunityStatus(existingID, req.MakePublic, req.IsCommunity)
	}
	
	// Import new recipe using existing import service
	importedRecipe, err := s.importService.ImportFromExternalAPI(req.Source, req.ExternalID)
	if err != nil {
		return nil, fmt.Errorf("failed to import recipe: %w", err)
	}
	
	// Update community flags
	updateQuery := `
		UPDATE recipes 
		SET is_public = $2, is_community = $3, updated_at = NOW()
		WHERE id = $1`
	
	_, err = s.db.Exec(updateQuery, importedRecipe.ID, req.MakePublic, req.IsCommunity)
	if err != nil {
		return nil, fmt.Errorf("failed to update community flags: %w", err)
	}
	
	// Return as community recipe
	return s.getCommunityRecipeByID(importedRecipe.ID)
}

// PromoteToPublic promotes a user recipe to public/community status
func (s *CommunityRecipeService) PromoteToPublic(recipeID, userID uuid.UUID, makeCommunity bool) (*CommunityRecipe, error) {
	// Verify user owns the recipe
	var ownerID uuid.UUID
	err := s.db.QueryRow("SELECT user_id FROM recipes WHERE id = $1 AND deleted_at IS NULL", recipeID).Scan(&ownerID)
	if err == sql.ErrNoRows {
		return nil, ErrRecipeNotFound
	}
	if err != nil {
		return nil, fmt.Errorf("failed to check recipe ownership: %w", err)
	}
	
	if ownerID != userID {
		return nil, ErrUnauthorized
	}
	
	// Update recipe status
	updateQuery := `
		UPDATE recipes 
		SET is_public = true, is_community = $2, updated_at = NOW()
		WHERE id = $1`
	
	_, err = s.db.Exec(updateQuery, recipeID, makeCommunity)
	if err != nil {
		return nil, fmt.Errorf("failed to promote recipe: %w", err)
	}
	
	return s.getCommunityRecipeByID(recipeID)
}

// GetRecommendedRecipesForUser gets personalized recipe recommendations
func (s *CommunityRecipeService) GetRecommendedRecipesForUser(userID uuid.UUID, limit int) ([]*CommunityRecipe, error) {
	// Get user preferences and rating history to personalize recommendations
	query := `
		WITH user_preferences AS (
			-- Get user's preferred meal types and complexity from rating history
			SELECT 
				COALESCE(ARRAY_AGG(DISTINCT unnest(r.meal_type)), '{}') as preferred_meal_types,
				COALESCE(AVG(CASE WHEN rr.overall_rating >= 4 THEN 
					CASE r.complexity 
						WHEN 'simple' THEN 1 
						WHEN 'moderate' THEN 2 
						WHEN 'complex' THEN 3 
					END 
				END), 2) as avg_liked_complexity
			FROM recipe_ratings rr
			JOIN recipes r ON rr.recipe_id = r.id
			WHERE rr.user_id = $1 AND rr.overall_rating >= 4
		)
		SELECT r.id, r.title, r.description, r.image_url, r.prep_time, r.cook_time, 
			   r.complexity, r.cuisine_type, r.meal_type, r.servings,
			   r.average_rating, r.total_ratings, r.rating_distribution,
			   crr.recommendation_score, crr.eligible_for_recommendations,
			   r.external_source, r.created_at, r.updated_at
		FROM community_recipes_ranked crr
		JOIN recipes r ON crr.id = r.id
		CROSS JOIN user_preferences up
		WHERE crr.eligible_for_recommendations = true
		AND NOT EXISTS (
			-- Exclude recipes user has already rated
			SELECT 1 FROM recipe_ratings rr2 
			WHERE rr2.recipe_id = r.id AND rr2.user_id = $1
		)
		AND (
			-- Match user preferences
			r.meal_type && up.preferred_meal_types OR
			array_length(up.preferred_meal_types, 1) IS NULL
		)
		ORDER BY crr.recommendation_score DESC, r.average_rating DESC
		LIMIT $2`
	
	return s.executeCommunityRecipeQuery(query, limit, userID)
}

// buildCommunityRecipeQuery builds SQL query for community recipe search
func (s *CommunityRecipeService) buildCommunityRecipeQuery(filters *CommunityRecipeFilters) string {
	query := `
		SELECT r.id, r.title, r.description, r.image_url, r.prep_time, r.cook_time, 
			   r.complexity, r.cuisine_type, r.meal_type, r.servings,
			   r.average_rating, r.total_ratings, r.rating_distribution,
			   crr.recommendation_score, crr.eligible_for_recommendations,
			   r.external_source, r.created_at, r.updated_at
		FROM community_recipes_ranked crr
		JOIN recipes r ON crr.id = r.id
		WHERE 1=1`
	
	// Add filters
	if filters != nil {
		if filters.SearchQuery != nil && *filters.SearchQuery != "" {
			query += ` AND to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')) 
					   @@ plainto_tsquery('english', $search_query)`
		}
		
		if filters.MinRating != nil {
			query += ` AND r.average_rating >= $min_rating`
		}
		
		if filters.MaxPrepTime != nil {
			query += ` AND r.prep_time <= $max_prep_time`
		}
		
		if len(filters.MealTypes) > 0 {
			query += ` AND r.meal_type && $meal_types`
		}
		
		if len(filters.Complexities) > 0 {
			query += ` AND r.complexity = ANY($complexities)`
		}
		
		if len(filters.CuisineTypes) > 0 {
			query += ` AND r.cuisine_type = ANY($cuisine_types)`
		}
		
		if filters.EligibleOnly {
			query += ` AND crr.eligible_for_recommendations = true`
		}
	}
	
	// Add sorting
	sortBy := "crr.recommendation_score DESC"
	if filters != nil {
		switch filters.SortBy {
		case "recent":
			sortBy = "r.created_at DESC"
		case "popular":
			sortBy = "r.total_ratings DESC"
		case "rating":
			sortBy = "r.average_rating DESC"
		case "trending":
			sortBy = "crr.recommendation_score DESC, r.total_ratings DESC"
		}
	}
	
	query += ` ORDER BY ` + sortBy + ` LIMIT $limit OFFSET $offset`
	
	return query
}

// buildCommunityRecipeCountQuery builds count query for pagination
func (s *CommunityRecipeService) buildCommunityRecipeCountQuery(filters *CommunityRecipeFilters) string {
	query := `
		SELECT COUNT(*) 
		FROM community_recipes_ranked crr
		JOIN recipes r ON crr.id = r.id
		WHERE 1=1`
	
	// Add same filters as main query (simplified)
	if filters != nil {
		if filters.SearchQuery != nil && *filters.SearchQuery != "" {
			query += ` AND to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')) 
					   @@ plainto_tsquery('english', '` + *filters.SearchQuery + `')`
		}
		
		if filters.MinRating != nil {
			query += fmt.Sprintf(` AND r.average_rating >= %.2f`, *filters.MinRating)
		}
		
		if filters.MaxPrepTime != nil {
			query += fmt.Sprintf(` AND r.prep_time <= %d`, *filters.MaxPrepTime)
		}
		
		if filters.EligibleOnly {
			query += ` AND crr.eligible_for_recommendations = true`
		}
	}
	
	return query
}

// executeCommunityRecipeQuery executes query and maps results to CommunityRecipe
func (s *CommunityRecipeService) executeCommunityRecipeQuery(query string, limit int, args ...interface{}) ([]*CommunityRecipe, error) {
	// Add limit and offset to args if not already present
	finalArgs := append(args, limit)
	if limit > 0 {
		// If we're using limit, we might also have offset
		finalArgs = append(finalArgs, 0) // Default offset
	}
	
	rows, err := s.db.Query(query, finalArgs...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	
	var recipes []*CommunityRecipe
	for rows.Next() {
		recipe := &CommunityRecipe{}
		var mealTypeJSON []byte
		var ratingDistJSON []byte
		
		err := rows.Scan(
			&recipe.ID, &recipe.Title, &recipe.Description, &recipe.ImageURL,
			&recipe.PrepTime, &recipe.CookTime, &recipe.Complexity, &recipe.CuisineType,
			&mealTypeJSON, &recipe.Servings, &recipe.AverageRating, &recipe.TotalRatings,
			&ratingDistJSON, &recipe.RecommendationScore, &recipe.EligibleForRecommendations,
			&recipe.ExternalSource, &recipe.CreatedAt, &recipe.UpdatedAt,
		)
		if err != nil {
			return nil, err
		}
		
		// Parse meal types JSON array
		var mealTypes []string
		if len(mealTypeJSON) > 0 {
			if err := json.Unmarshal(mealTypeJSON, &mealTypes); err == nil {
				recipe.MealType = mealTypes
			}
		}
		
		// Parse rating distribution JSON
		if len(ratingDistJSON) > 0 {
			if err := json.Unmarshal(ratingDistJSON, &recipe.RatingDistribution); err != nil {
				recipe.RatingDistribution = map[string]int{
					"1": 0, "2": 0, "3": 0, "4": 0, "5": 0,
				}
			}
		} else {
			recipe.RatingDistribution = map[string]int{
				"1": 0, "2": 0, "3": 0, "4": 0, "5": 0,
			}
		}
		
		recipe.TotalTime = recipe.PrepTime + recipe.CookTime
		recipes = append(recipes, recipe)
	}
	
	return recipes, rows.Err()
}

// getCommunityRecipeByID gets a single community recipe by ID
func (s *CommunityRecipeService) getCommunityRecipeByID(id uuid.UUID) (*CommunityRecipe, error) {
	query := `
		SELECT r.id, r.title, r.description, r.image_url, r.prep_time, r.cook_time, 
			   r.complexity, r.cuisine_type, r.meal_type, r.servings,
			   r.average_rating, r.total_ratings, r.rating_distribution,
			   crr.recommendation_score, crr.eligible_for_recommendations,
			   r.external_source, r.created_at, r.updated_at
		FROM community_recipes_ranked crr
		JOIN recipes r ON crr.id = r.id
		WHERE r.id = $1`
	
	recipes, err := s.executeCommunityRecipeQuery(query, 1, id)
	if err != nil {
		return nil, err
	}
	if len(recipes) == 0 {
		return nil, ErrRecipeNotFound
	}
	
	return recipes[0], nil
}

// updateRecipeCommunityStatus updates a recipe's community status
func (s *CommunityRecipeService) updateRecipeCommunityStatus(recipeID uuid.UUID, isPublic, isCommunity bool) (*CommunityRecipe, error) {
	query := `
		UPDATE recipes 
		SET is_public = $2, is_community = $3, updated_at = NOW()
		WHERE id = $1`
	
	_, err := s.db.Exec(query, recipeID, isPublic, isCommunity)
	if err != nil {
		return nil, err
	}
	
	return s.getCommunityRecipeByID(recipeID)
}

// getCommunityStats gets community recipe statistics
func (s *CommunityRecipeService) getCommunityStats() (*CommunityStats, error) {
	query := `
		SELECT 
			COUNT(*) as total_recipes,
			AVG(average_rating) as avg_rating,
			jsonb_object_agg(cuisine_type, cuisine_count) as top_cuisines,
			jsonb_object_agg(complexity, complexity_count) as complexity_breakdown
		FROM (
			SELECT 
				r.cuisine_type,
				r.complexity,
				r.average_rating,
				COUNT(*) OVER (PARTITION BY r.cuisine_type) as cuisine_count,
				COUNT(*) OVER (PARTITION BY r.complexity) as complexity_count
			FROM recipes r
			WHERE r.is_community = true AND r.is_public = true AND r.deleted_at IS NULL
		) stats`
	
	var totalRecipes int
	var avgRating sql.NullFloat64
	var cuisinesJSON, complexityJSON []byte
	
	err := s.db.QueryRow(query).Scan(&totalRecipes, &avgRating, &cuisinesJSON, &complexityJSON)
	if err != nil {
		return nil, err
	}
	
	stats := &CommunityStats{
		TotalRecipes:        totalRecipes,
		AverageRating:      avgRating.Float64,
		TopCuisines:        make(map[string]int),
		ComplexityBreakdown: make(map[string]int),
	}
	
	// Parse JSON data
	if len(cuisinesJSON) > 0 {
		json.Unmarshal(cuisinesJSON, &stats.TopCuisines)
	}
	
	if len(complexityJSON) > 0 {
		json.Unmarshal(complexityJSON, &stats.ComplexityBreakdown)
	}
	
	return stats, nil
}