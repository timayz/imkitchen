package services

import (
	"errors"
	"fmt"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

type CommunityImportService interface {
	ImportCommunityRecipe(userID, communityRecipeID uuid.UUID, request *models.RecipeImportRequest) (*models.RecipeImportResponse, error)
	GetImportHistory(userID uuid.UUID, page, limit int) ([]models.RecipeImport, int, error)
	CheckImportConflict(userID, communityRecipeID uuid.UUID) (*models.ImportConflict, error)
	GetImportStats(userID uuid.UUID) (*models.ImportStats, error)
}

type communityImportService struct {
	importRepo        repositories.RecipeImportRepository
	recipeRepo        repositories.RecipeRepository
	communityRepo     repositories.CommunityRecipeRepository
	rateLimiter       ImportRateLimiter
	validator         *validator.Validate
}

func NewCommunityImportService(
	importRepo repositories.RecipeImportRepository,
	recipeRepo repositories.RecipeRepository,
	communityRepo repositories.CommunityRecipeRepository,
	rateLimiter ImportRateLimiter,
) CommunityImportService {
	return &communityImportService{
		importRepo:    importRepo,
		recipeRepo:    recipeRepo,
		communityRepo: communityRepo,
		rateLimiter:   rateLimiter,
		validator:     validator.New(),
	}
}

// ImportCommunityRecipe imports a community recipe into a user's personal collection
func (s *communityImportService) ImportCommunityRecipe(
	userID, communityRecipeID uuid.UUID,
	request *models.RecipeImportRequest,
) (*models.RecipeImportResponse, error) {
	// Validate request
	if err := s.validator.Struct(request); err != nil {
		return nil, errors.New("validation error")
	}

	// Check rate limits (20 imports per hour per user)
	if !s.rateLimiter.Allow(fmt.Sprintf("import:%s", userID.String()), 20, time.Hour) {
		return nil, errors.New("import rate limit exceeded")
	}

	// Check if community recipe exists
	communityRecipe, err := s.communityRepo.GetByID(communityRecipeID)
	if err != nil || communityRecipe == nil {
		return nil, errors.New("community recipe not found")
	}

	// Check for existing import conflict
	conflict, err := s.CheckImportConflict(userID, communityRecipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to check import conflict: %w", err)
	}
	if conflict != nil {
		return nil, errors.New("recipe already imported")
	}

	// Begin import process
	now := time.Now()
	
	// Create the personal recipe from community recipe
	personalRecipe := &models.Recipe{
		ID:             uuid.New(),
		UserID:         userID,
		Title:          communityRecipe.Title,
		Description:    communityRecipe.Description,
		ImageURL:       communityRecipe.ImageURL,
		PrepTime:       communityRecipe.PrepTime,
		CookTime:       communityRecipe.CookTime,
		TotalTime:      communityRecipe.TotalTime,
		MealType:       communityRecipe.MealType,
		Complexity:     communityRecipe.Complexity,
		CuisineType:    communityRecipe.CuisineType,
		Servings:       communityRecipe.Servings,
		Ingredients:    communityRecipe.Ingredients,
		Instructions:   communityRecipe.Instructions,
		DietaryLabels:  communityRecipe.DietaryLabels,
		IsPublic:       false, // Personal recipes are private by default
		IsCommunity:    false,
		CreatedAt:      now,
		UpdatedAt:      now,
	}

	// Apply customizations if provided
	if request.Customizations != nil {
		if request.Customizations.Title != nil {
			personalRecipe.Title = *request.Customizations.Title
		}
		if request.Customizations.Notes != nil {
			// Add notes to description
			if personalRecipe.Description != nil {
				personalRecipe.Description = request.Customizations.Notes
			} else {
				personalRecipe.Description = request.Customizations.Notes
			}
		}
		if request.Customizations.ServingAdjustment != nil {
			personalRecipe.Servings = *request.Customizations.ServingAdjustment
			// TODO: Adjust ingredient quantities proportionally
		}
	}

	// Create the personal recipe
	createdRecipe, err := s.recipeRepo.Create(personalRecipe)
	if err != nil {
		return nil, fmt.Errorf("failed to create personal recipe: %w", err)
	}

	// Create import tracking record
	importRecord := &models.RecipeImport{
		ID:                uuid.New(),
		UserID:            userID,
		PersonalRecipeID:  createdRecipe.ID,
		CommunityRecipeID: communityRecipeID,
		ImportedAt:        now,
		PreserveAttribution: request.PreserveAttribution,
	}

	if request.PreserveAttribution {
		importRecord.OriginalContributor = communityRecipe.ContributorName
		importRecord.ImportDate = &now
	}

	err = s.importRepo.Create(importRecord)
	if err != nil {
		// Rollback personal recipe creation
		s.recipeRepo.Delete(createdRecipe.ID, userID)
		return nil, fmt.Errorf("failed to create import record: %w", err)
	}

	// Update community recipe import count
	err = s.communityRepo.IncrementImportCount(communityRecipeID)
	if err != nil {
		// Log error but don't fail the import
		// TODO: Add proper logging
	}

	// Prepare response
	response := &models.RecipeImportResponse{
		Success:          true,
		PersonalRecipeID: &createdRecipe.ID.String(),
		Message:          "Recipe successfully imported to your collection",
	}

	if request.PreserveAttribution && communityRecipe.ContributorName != nil {
		response.Attribution = &models.ImportAttribution{
			OriginalContributor: *communityRecipe.ContributorName,
			ImportDate:          now,
			CommunityMetrics: models.CommunityMetrics{
				TotalImports:  communityRecipe.ImportCount + 1, // Include this import
				AverageRating: communityRecipe.AverageRating,
			},
		}
	}

	return response, nil
}

// GetImportHistory retrieves the user's import history with pagination
func (s *communityImportService) GetImportHistory(userID uuid.UUID, page, limit int) ([]models.RecipeImport, int, error) {
	offset := (page - 1) * limit
	
	imports, err := s.importRepo.GetByUserID(userID, limit, offset)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to get import history: %w", err)
	}

	total, err := s.importRepo.CountByUserID(userID)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to count imports: %w", err)
	}

	return imports, total, nil
}

// CheckImportConflict checks if a recipe has already been imported by the user
func (s *communityImportService) CheckImportConflict(userID, communityRecipeID uuid.UUID) (*models.ImportConflict, error) {
	existingImport, err := s.importRepo.FindByCommunityRecipeID(userID, communityRecipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to check for existing import: %w", err)
	}

	if existingImport == nil {
		return nil, nil // No conflict
	}

	// Get the personal recipe details for conflict resolution
	personalRecipe, err := s.recipeRepo.GetByID(existingImport.PersonalRecipeID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get personal recipe: %w", err)
	}

	conflict := &models.ImportConflict{
		ExistingRecipeID:    existingImport.PersonalRecipeID.String(),
		ExistingRecipeTitle: personalRecipe.Title,
		ImportedAt:          existingImport.ImportedAt,
		ConflictType:        "duplicate_import",
		Resolution: models.ConflictResolution{
			Options: []string{"rename", "merge", "replace", "cancel"},
			Recommended: "rename",
		},
	}

	return conflict, nil
}

// GetImportStats returns statistics about the user's import activity
func (s *communityImportService) GetImportStats(userID uuid.UUID) (*models.ImportStats, error) {
	totalImports, err := s.importRepo.CountByUserID(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to count total imports: %w", err)
	}

	recentImports, err := s.importRepo.CountRecentImports(userID, 24*time.Hour)
	if err != nil {
		return nil, fmt.Errorf("failed to count recent imports: %w", err)
	}

	favoriteCategories, err := s.importRepo.GetTopCategories(userID, 5)
	if err != nil {
		return nil, fmt.Errorf("failed to get favorite categories: %w", err)
	}

	stats := &models.ImportStats{
		TotalImports:       totalImports,
		RecentImports:      recentImports,
		FavoriteCategories: favoriteCategories,
		ImportLimit:        20, // Per hour limit
		ImportLimitWindow:  "1h",
	}

	return stats, nil
}

// ImportRateLimiter interface for import rate limiting
type ImportRateLimiter interface {
	Allow(key string, limit int, window time.Duration) bool
}