package repositories

import (
	"time"
	
	"github.com/google/uuid"
	"gorm.io/gorm"
	
	"github.com/imkitchen/backend/internal/models"
)

type RecipeImportRepository interface {
	Create(recipeImport *models.RecipeImport) error
	GetByID(id uuid.UUID) (*models.RecipeImport, error)
	GetByUserID(userID uuid.UUID, limit, offset int) ([]models.RecipeImport, error)
	CountByUserID(userID uuid.UUID) (int, error)
	CountRecentImports(userID uuid.UUID, window time.Duration) (int, error)
	FindByCommunityRecipeID(userID, communityRecipeID uuid.UUID) (*models.RecipeImport, error)
	GetTopCategories(userID uuid.UUID, limit int) ([]models.CategoryStat, error)
	Update(recipeImport *models.RecipeImport) error
	Delete(id uuid.UUID) error
}

type recipeImportRepository struct {
	db *gorm.DB
}

func NewRecipeImportRepository(db *gorm.DB) RecipeImportRepository {
	return &recipeImportRepository{db: db}
}

// Create creates a new recipe import record
func (r *recipeImportRepository) Create(recipeImport *models.RecipeImport) error {
	return r.db.Create(recipeImport).Error
}

// GetByID retrieves a recipe import by its ID
func (r *recipeImportRepository) GetByID(id uuid.UUID) (*models.RecipeImport, error) {
	var recipeImport models.RecipeImport
	err := r.db.Where("id = ?", id).First(&recipeImport).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, nil
		}
		return nil, err
	}
	return &recipeImport, nil
}

// GetByUserID retrieves recipe imports for a specific user with pagination
func (r *recipeImportRepository) GetByUserID(userID uuid.UUID, limit, offset int) ([]models.RecipeImport, error) {
	var imports []models.RecipeImport
	
	err := r.db.Where("user_id = ?", userID).
		Preload("PersonalRecipe").
		Preload("CommunityRecipe").
		Order("imported_at DESC").
		Limit(limit).
		Offset(offset).
		Find(&imports).Error
	
	if err != nil {
		return nil, err
	}
	
	return imports, nil
}

// CountByUserID counts total imports for a user
func (r *recipeImportRepository) CountByUserID(userID uuid.UUID) (int, error) {
	var count int64
	err := r.db.Model(&models.RecipeImport{}).Where("user_id = ?", userID).Count(&count).Error
	return int(count), err
}

// CountRecentImports counts imports within a specific time window
func (r *recipeImportRepository) CountRecentImports(userID uuid.UUID, window time.Duration) (int, error) {
	cutoff := time.Now().Add(-window)
	var count int64
	
	err := r.db.Model(&models.RecipeImport{}).
		Where("user_id = ? AND imported_at > ?", userID, cutoff).
		Count(&count).Error
	
	return int(count), err
}

// FindByCommunityRecipeID finds an existing import of a specific community recipe by a user
func (r *recipeImportRepository) FindByCommunityRecipeID(userID, communityRecipeID uuid.UUID) (*models.RecipeImport, error) {
	var recipeImport models.RecipeImport
	
	err := r.db.Where("user_id = ? AND community_recipe_id = ?", userID, communityRecipeID).
		First(&recipeImport).Error
	
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, nil
		}
		return nil, err
	}
	
	return &recipeImport, nil
}

// GetTopCategories retrieves the user's most imported recipe categories
func (r *recipeImportRepository) GetTopCategories(userID uuid.UUID, limit int) ([]models.CategoryStat, error) {
	var results []models.CategoryStat
	
	// This query joins with community recipes to get meal types and aggregates them
	err := r.db.Raw(`
		SELECT 
			unnest(cr.meal_type) as category,
			COUNT(*) as count
		FROM recipe_imports ri
		JOIN community_recipes cr ON ri.community_recipe_id = cr.id
		WHERE ri.user_id = ?
		GROUP BY unnest(cr.meal_type)
		ORDER BY count DESC
		LIMIT ?
	`, userID, limit).Scan(&results).Error
	
	return results, err
}

// Update updates an existing recipe import record
func (r *recipeImportRepository) Update(recipeImport *models.RecipeImport) error {
	return r.db.Save(recipeImport).Error
}

// Delete removes a recipe import record
func (r *recipeImportRepository) Delete(id uuid.UUID) error {
	return r.db.Where("id = ?", id).Delete(&models.RecipeImport{}).Error
}

// CommunityRecipeRepository interface for community recipe operations
type CommunityRecipeRepository interface {
	GetByID(id uuid.UUID) (*models.CommunityRecipe, error)
	IncrementImportCount(id uuid.UUID) error
	GetTrending(limit int) ([]models.CommunityRecipe, error)
	GetHighlyRated(minRating float64, limit int) ([]models.CommunityRecipe, error)
	Search(filters *CommunityRecipeFilters, page, limit int) ([]models.CommunityRecipe, int, error)
}

type communityRecipeRepository struct {
	db *gorm.DB
}

func NewCommunityRecipeRepository(db *gorm.DB) CommunityRecipeRepository {
	return &communityRecipeRepository{db: db}
}

// GetByID retrieves a community recipe by ID
func (r *communityRecipeRepository) GetByID(id uuid.UUID) (*models.CommunityRecipe, error) {
	var recipe models.CommunityRecipe
	err := r.db.Where("id = ?", id).First(&recipe).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, nil
		}
		return nil, err
	}
	return &recipe, nil
}

// IncrementImportCount increments the import count for a community recipe
func (r *communityRecipeRepository) IncrementImportCount(id uuid.UUID) error {
	return r.db.Model(&models.CommunityRecipe{}).
		Where("id = ?", id).
		UpdateColumn("import_count", gorm.Expr("import_count + 1")).Error
}

// GetTrending retrieves trending community recipes
func (r *communityRecipeRepository) GetTrending(limit int) ([]models.CommunityRecipe, error) {
	var recipes []models.CommunityRecipe
	
	err := r.db.Where("is_trending = true").
		Order("trending_score DESC, created_at DESC").
		Limit(limit).
		Find(&recipes).Error
	
	return recipes, err
}

// GetHighlyRated retrieves highly-rated community recipes
func (r *communityRecipeRepository) GetHighlyRated(minRating float64, limit int) ([]models.CommunityRecipe, error) {
	var recipes []models.CommunityRecipe
	
	err := r.db.Where("average_rating >= ? AND total_ratings >= 3", minRating).
		Order("average_rating DESC, total_ratings DESC").
		Limit(limit).
		Find(&recipes).Error
	
	return recipes, err
}

// Search searches community recipes with filters
func (r *communityRecipeRepository) Search(filters *CommunityRecipeFilters, page, limit int) ([]models.CommunityRecipe, int, error) {
	var recipes []models.CommunityRecipe
	var total int64
	
	query := r.db.Model(&models.CommunityRecipe{})
	
	// Apply filters
	if filters.SearchQuery != nil && *filters.SearchQuery != "" {
		searchTerm := "%" + *filters.SearchQuery + "%"
		query = query.Where("title ILIKE ? OR description ILIKE ?", searchTerm, searchTerm)
	}
	
	if filters.MinRating != nil {
		query = query.Where("average_rating >= ?", *filters.MinRating)
	}
	
	if filters.MaxPrepTime != nil {
		query = query.Where("prep_time <= ?", *filters.MaxPrepTime)
	}
	
	if filters.MealTypes != nil && len(filters.MealTypes) > 0 {
		query = query.Where("meal_type && ?", filters.MealTypes)
	}
	
	if filters.Complexities != nil && len(filters.Complexities) > 0 {
		query = query.Where("complexity IN ?", filters.Complexities)
	}
	
	if filters.Tags != nil && len(filters.Tags) > 0 {
		query = query.Where("user_tags && ?", filters.Tags)
	}
	
	// Count total
	query.Count(&total)
	
	// Apply sorting
	switch filters.SortBy {
	case "rating":
		query = query.Order("average_rating DESC, total_ratings DESC")
	case "recent":
		query = query.Order("created_at DESC")
	case "popular":
		query = query.Order("import_count DESC")
	case "trending":
		query = query.Order("trending_score DESC")
	default:
		query = query.Order("created_at DESC")
	}
	
	// Apply pagination
	offset := (page - 1) * limit
	err := query.Limit(limit).Offset(offset).Find(&recipes).Error
	
	return recipes, int(total), err
}

// CommunityRecipeFilters represents search and filter options for community recipes
type CommunityRecipeFilters struct {
	SearchQuery   *string  `json:"searchQuery,omitempty"`
	SortBy        string   `json:"sortBy" validate:"oneof=rating recent popular trending"`
	MinRating     *float64 `json:"minRating,omitempty" validate:"omitempty,min=1,max=5"`
	MaxPrepTime   *int     `json:"maxPrepTime,omitempty" validate:"omitempty,min=0"`
	MealTypes     []string `json:"mealTypes,omitempty"`
	Complexities  []string `json:"complexities,omitempty" validate:"omitempty,dive,oneof=simple moderate complex"`
	CuisineTypes  []string `json:"cuisineTypes,omitempty"`
	DietaryLabels []string `json:"dietaryLabels,omitempty"`
	Tags          []string `json:"tags,omitempty"`
}

// Add the filters to the models file
func init() {
	// This will be moved to the models package
}