package services

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/url"

	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

type RecipeService interface {
	CreateRecipe(userID uuid.UUID, input *models.CreateRecipeInput) (*models.Recipe, error)
	GetRecipe(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error)
	GetUserRecipes(userID uuid.UUID, limit, offset int) ([]models.Recipe, error)
	UpdateRecipe(id uuid.UUID, userID uuid.UUID, input *models.UpdateRecipeInput) (*models.Recipe, error)
	DeleteRecipe(id uuid.UUID, userID uuid.UUID) error
	SearchRecipes(userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error)
	ImportRecipe(userID uuid.UUID, input *models.ImportRecipeInput) (*models.ImportRecipeResult, error)
}

type recipeService struct {
	repo          repositories.RecipeRepository
	importService RecipeImportService
	validator     *validator.Validate
}

func NewRecipeService(repo repositories.RecipeRepository) RecipeService {
	return &recipeService{
		repo:          repo,
		importService: NewRecipeImportService(),
		validator:     validator.New(),
	}
}

func (s *recipeService) CreateRecipe(userID uuid.UUID, input *models.CreateRecipeInput) (*models.Recipe, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	// Additional business logic validation
	if err := s.validateRecipeBusinessRules(input); err != nil {
		return nil, err
	}

	// Convert ingredients and instructions to JSON
	ingredientsJSON, err := json.Marshal(input.Ingredients)
	if err != nil {
		return nil, fmt.Errorf("error marshaling ingredients: %w", err)
	}

	instructionsJSON, err := json.Marshal(input.Instructions)
	if err != nil {
		return nil, fmt.Errorf("error marshaling instructions: %w", err)
	}

	// Create recipe model
	recipe := &models.Recipe{
		ID:            uuid.New(),
		UserID:        userID,
		Title:         input.Title,
		Description:   input.Description,
		PrepTime:      input.PrepTime,
		CookTime:      input.CookTime,
		MealType:      input.MealType,
		Complexity:    input.Complexity,
		CuisineType:   input.CuisineType,
		Servings:      input.Servings,
		Ingredients:   ingredientsJSON,
		Instructions:  instructionsJSON,
		DietaryLabels: input.DietaryLabels,
		ImageURL:      input.ImageURL,
		SourceURL:     input.SourceURL,
	}

	// Save to database
	if err := s.repo.Create(recipe); err != nil {
		return nil, fmt.Errorf("error creating recipe: %w", err)
	}

	return recipe, nil
}

func (s *recipeService) GetRecipe(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error) {
	recipe, err := s.repo.GetByID(id, userID)
	if err != nil {
		return nil, fmt.Errorf("error getting recipe: %w", err)
	}
	return recipe, nil
}

func (s *recipeService) GetUserRecipes(userID uuid.UUID, limit, offset int) ([]models.Recipe, error) {
	recipes, err := s.repo.GetByUserID(userID, limit, offset)
	if err != nil {
		return nil, fmt.Errorf("error getting user recipes: %w", err)
	}
	return recipes, nil
}

func (s *recipeService) UpdateRecipe(id uuid.UUID, userID uuid.UUID, input *models.UpdateRecipeInput) (*models.Recipe, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	// Additional business logic validation for updates
	if err := s.validateUpdateBusinessRules(input); err != nil {
		return nil, err
	}

	// Update recipe
	recipe, err := s.repo.Update(id, userID, input)
	if err != nil {
		return nil, fmt.Errorf("error updating recipe: %w", err)
	}

	return recipe, nil
}

func (s *recipeService) DeleteRecipe(id uuid.UUID, userID uuid.UUID) error {
	// Check if recipe exists and belongs to user before deletion
	_, err := s.repo.GetByID(id, userID)
	if err != nil {
		return fmt.Errorf("recipe not found: %w", err)
	}

	if err := s.repo.Delete(id, userID); err != nil {
		return fmt.Errorf("error deleting recipe: %w", err)
	}

	return nil
}

func (s *recipeService) SearchRecipes(userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {
	// Set defaults if not provided
	if params.Page == 0 {
		params.Page = 1
	}
	if params.Limit == 0 {
		params.Limit = 20
	}
	if params.SortBy == "" {
		params.SortBy = "created_at"
	}
	if params.SortOrder == "" {
		params.SortOrder = "desc"
	}

	// Validate search parameters
	if err := s.validator.Struct(params); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	return s.repo.Search(userID, params)
}

func (s *recipeService) ImportRecipe(userID uuid.UUID, input *models.ImportRecipeInput) (*models.ImportRecipeResult, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   recipeStringPtr(fmt.Sprintf("validation error: %v", err)),
		}, nil
	}

	// Check if we've already imported this recipe
	parsedURL, _ := url.Parse(input.URL)
	if parsedURL != nil {
		if existingRecipe, err := s.repo.GetByExternalSource("user_import", input.URL); err == nil && existingRecipe != nil {
			return &models.ImportRecipeResult{
				Success:  true,
				Recipe:   existingRecipe,
				Warnings: []string{"Recipe was already imported from this URL"},
			}, nil
		}
	}

	// Use the import service to parse the recipe
	importResult, err := s.importService.ImportFromURL(input)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   recipeStringPtr(fmt.Sprintf("import failed: %v", err)),
		}, nil
	}

	if !importResult.Success {
		return importResult, nil
	}

	// If import was successful, create the recipe
	htmlContent, err := s.importService.(*recipeImportService).fetchURL(input.URL)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   recipeStringPtr(fmt.Sprintf("failed to fetch URL: %v", err)),
		}, nil
	}

	recipeInput, err := s.importService.ParseRecipeFromHTML(htmlContent, input.URL)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   recipeStringPtr(fmt.Sprintf("failed to parse recipe: %v", err)),
		}, nil
	}

	// Apply overrides if provided
	if input.OverrideFields != nil {
		s.applyImportOverrides(recipeInput, input.OverrideFields)
	}

	// Set source information
	recipeInput.SourceURL = &input.URL

	// Create the recipe
	recipe, err := s.CreateRecipe(userID, recipeInput)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   recipeStringPtr(fmt.Sprintf("failed to create recipe: %v", err)),
		}, nil
	}

	// Mark as imported
	recipe.ExternalSource = recipeStringPtr("user_import")
	recipe.ExternalID = &input.URL

	// Update the recipe with external source info
	updateInput := &models.UpdateRecipeInput{}
	if err := s.repo.Update(recipe.ID, userID, updateInput); err != nil {
		// Log error but don't fail the import
	}

	return &models.ImportRecipeResult{
		Success:  true,
		Recipe:   recipe,
		Warnings: importResult.Warnings,
	}, nil
}

// validateRecipeBusinessRules performs additional business logic validation
func (s *recipeService) validateRecipeBusinessRules(input *models.CreateRecipeInput) error {
	// Ensure at least one ingredient
	if len(input.Ingredients) == 0 {
		return errors.New("recipe must have at least one ingredient")
	}

	// Ensure at least one instruction
	if len(input.Instructions) == 0 {
		return errors.New("recipe must have at least one instruction")
	}

	// Validate instruction step numbers are sequential
	for i, instruction := range input.Instructions {
		if instruction.StepNumber != i+1 {
			return fmt.Errorf("instruction step numbers must be sequential, expected %d but got %d", i+1, instruction.StepNumber)
		}
	}

	// Validate ingredient amounts are positive
	for _, ingredient := range input.Ingredients {
		if ingredient.Amount <= 0 {
			return fmt.Errorf("ingredient %s must have a positive amount", ingredient.Name)
		}
	}

	// Validate total time doesn't exceed reasonable limits (16 hours)
	if input.PrepTime+input.CookTime > 960 {
		return errors.New("total recipe time cannot exceed 16 hours")
	}

	return nil
}

// validateUpdateBusinessRules performs validation for update operations
func (s *recipeService) validateUpdateBusinessRules(input *models.UpdateRecipeInput) error {
	// Similar validation for updates, but with optional fields
	if input.Ingredients != nil {
		if len(*input.Ingredients) == 0 {
			return errors.New("recipe must have at least one ingredient")
		}
		for _, ingredient := range *input.Ingredients {
			if ingredient.Amount <= 0 {
				return fmt.Errorf("ingredient %s must have a positive amount", ingredient.Name)
			}
		}
	}

	if input.Instructions != nil {
		if len(*input.Instructions) == 0 {
			return errors.New("recipe must have at least one instruction")
		}
		for i, instruction := range *input.Instructions {
			if instruction.StepNumber != i+1 {
				return fmt.Errorf("instruction step numbers must be sequential, expected %d but got %d", i+1, instruction.StepNumber)
			}
		}
	}

	// Validate total time if both times are provided
	if input.PrepTime != nil && input.CookTime != nil {
		if *input.PrepTime+*input.CookTime > 960 {
			return errors.New("total recipe time cannot exceed 16 hours")
		}
	}

	return nil
}

// applyImportOverrides applies override fields during import
func (s *recipeService) applyImportOverrides(recipe *models.CreateRecipeInput, overrides *models.CreateRecipeInput) {
	if overrides.Title != "" {
		recipe.Title = overrides.Title
	}
	if overrides.Description != nil {
		recipe.Description = overrides.Description
	}
	if overrides.PrepTime > 0 {
		recipe.PrepTime = overrides.PrepTime
	}
	if overrides.CookTime > 0 {
		recipe.CookTime = overrides.CookTime
	}
	if len(overrides.MealType) > 0 {
		recipe.MealType = overrides.MealType
	}
	if overrides.Complexity != "" {
		recipe.Complexity = overrides.Complexity
	}
	if overrides.CuisineType != nil {
		recipe.CuisineType = overrides.CuisineType
	}
	if overrides.Servings > 0 {
		recipe.Servings = overrides.Servings
	}
	if len(overrides.Ingredients) > 0 {
		recipe.Ingredients = overrides.Ingredients
	}
	if len(overrides.Instructions) > 0 {
		recipe.Instructions = overrides.Instructions
	}
	if len(overrides.DietaryLabels) > 0 {
		recipe.DietaryLabels = overrides.DietaryLabels
	}
	if overrides.ImageURL != nil {
		recipe.ImageURL = overrides.ImageURL
	}
}

// Helper function to create string pointer
func recipeStringPtr(s string) *string {
	return &s
}