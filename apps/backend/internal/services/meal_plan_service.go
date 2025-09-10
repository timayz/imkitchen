package services

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

type MealPlanService interface {
	CreateMealPlan(userID uuid.UUID, input *models.CreateMealPlanInput) (*models.MealPlan, error)
	GetMealPlan(id uuid.UUID, userID uuid.UUID) (*models.MealPlanResponse, error)
	GetUserMealPlans(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlanResponse, error)
	GetMealPlanByWeek(userID uuid.UUID, weekStart time.Time) (*models.MealPlanResponse, error)
	UpdateMealPlan(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlanResponse, error)
	UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlanResponse, error)
	DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlanResponse, error)
	DeleteMealPlan(id uuid.UUID, userID uuid.UUID) error
	
	// New meal substitution and flexibility methods
	UpdateMealEntry(mealPlanID uuid.UUID, entryID string, userID uuid.UUID, input *models.MealEntryUpdateRequest) (*models.MealPlanResponse, error)
	ReorderMeals(mealPlanID uuid.UUID, userID uuid.UUID, input *models.MealReorderRequest) (*models.MealPlanResponse, error)
	GetSwapSuggestions(mealPlanID uuid.UUID, entryID string, userID uuid.UUID) ([]models.Recipe, error)
	GetChangeHistory(mealPlanID uuid.UUID, userID uuid.UUID, filters *models.ChangeHistoryFilters) (*models.ChangeHistoryResponse, error)
	UndoLastChange(mealPlanID uuid.UUID, userID uuid.UUID) (*models.MealPlanResponse, error)
	RedoLastUndo(mealPlanID uuid.UUID, userID uuid.UUID) (*models.MealPlanResponse, error)
	
	// Community-enhanced meal planning
	GetCommunityRecommendedMeals(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.Recipe, error)
	GenerateRatingAwareMealPlan(userID uuid.UUID, preferences *models.MealPlanPreferences) (*models.MealPlan, error)
}

type mealPlanService struct {
	repo                repositories.MealPlanRepository
	recipeRepo          repositories.RecipeRepository
	changeHistoryRepo   repositories.MealPlanChangeHistoryRepository
	communityService    *CommunityRecipeService
	validator           *validator.Validate
}

func NewMealPlanService(repo repositories.MealPlanRepository, recipeRepo repositories.RecipeRepository, changeHistoryRepo repositories.MealPlanChangeHistoryRepository, communityService *CommunityRecipeService) MealPlanService {
	return &mealPlanService{
		repo:                repo,
		recipeRepo:          recipeRepo,
		changeHistoryRepo:   changeHistoryRepo,
		communityService:    communityService,
		validator:           validator.New(),
	}
}

func (s *mealPlanService) CreateMealPlan(userID uuid.UUID, input *models.CreateMealPlanInput) (*models.MealPlan, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	// Check if meal plan already exists for this week
	existing, err := s.repo.GetByWeekStart(userID, input.WeekStartDate)
	if err == nil && existing != nil {
		return nil, fmt.Errorf("meal plan already exists for week starting %s", input.WeekStartDate.Format("2006-01-02"))
	}

	// Create meal plan model
	mealPlan := &models.MealPlan{
		ID:             uuid.New(),
		UserID:         userID,
		WeekStartDate:  input.WeekStartDate,
		GenerationType: input.GenerationType,
		GeneratedAt:    time.Now(),
		IsActive:       true,
		Status:         "active",
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
	}

	// Convert meals to JSONB format
	mealsJSON, err := json.Marshal(input.Meals)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal meals: %w", err)
	}
	mealPlan.Meals = mealsJSON

	// Calculate total estimated time
	mealPlan.TotalEstimatedTime = s.calculateTotalTime(&input.Meals, userID)

	// Create meal plan
	if err := s.repo.Create(mealPlan); err != nil {
		return nil, fmt.Errorf("failed to create meal plan: %w", err)
	}

	return mealPlan, nil
}

// GetCommunityRecommendedMeals gets community recipes recommended for meal planning
func (s *mealPlanService) GetCommunityRecommendedMeals(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.Recipe, error) {
	if s.communityService == nil {
		// Fallback to personal recipes if community service not available
		return s.recipeRepo.GetSimilarRecipes(userID, &models.RecipeFilters{
			MealType:       filters.MealType,
			Complexity:     filters.Complexity,
			MaxPrepTime:    filters.MaxPrepTime,
			MaxCookTime:    filters.MaxCookTime,
			CuisineType:    filters.CuisineType,
		})
	}

	// Get personalized recommendations from community
	communityRecipes, err := s.communityService.GetRecommendedRecipesForUser(userID, 20)
	if err != nil {
		// Fallback to highly rated recipes if personalized recommendations fail
		communityRecipes, err = s.communityService.GetHighlyRatedRecipes(3, 20)
		if err != nil {
			return nil, fmt.Errorf("failed to get community recommendations: %w", err)
		}
	}

	// Convert community recipes to recipe models
	var recipes []models.Recipe
	for _, cr := range communityRecipes {
		// Apply filters
		if filters.MealType != nil && !s.containsAnyMealType(cr.MealType, *filters.MealType) {
			continue
		}
		if filters.Complexity != nil && cr.Complexity != *filters.Complexity {
			continue
		}
		if filters.MaxPrepTime != nil && cr.PrepTime > *filters.MaxPrepTime {
			continue
		}
		if filters.MaxCookTime != nil && cr.CookTime > *filters.MaxCookTime {
			continue
		}
		if filters.CuisineType != nil && (cr.CuisineType == nil || *cr.CuisineType != *filters.CuisineType) {
			continue
		}

		recipe := models.Recipe{
			ID:              cr.ID,
			Title:           cr.Title,
			Description:     cr.Description,
			ImageURL:        cr.ImageURL,
			PrepTime:        cr.PrepTime,
			CookTime:        cr.CookTime,
			TotalTime:       cr.TotalTime,
			Complexity:      cr.Complexity,
			CuisineType:     cr.CuisineType,
			MealType:        cr.MealType,
			Servings:        cr.Servings,
			AverageRating:   cr.AverageRating,
			TotalRatings:    cr.TotalRatings,
			ExternalSource:  cr.ExternalSource,
			CreatedAt:       cr.CreatedAt,
			UpdatedAt:       cr.UpdatedAt,
		}
		recipes = append(recipes, recipe)
	}

	return recipes, nil
}

// GenerateRatingAwareMealPlan generates a meal plan that considers community ratings
func (s *mealPlanService) GenerateRatingAwareMealPlan(userID uuid.UUID, preferences *models.MealPlanPreferences) (*models.MealPlan, error) {
	// Get user's dietary preferences and constraints
	mealPlanFilters := &models.MealPlanFilters{
		MealType:    preferences.PreferredMealTypes,
		Complexity:  preferences.PreferredComplexity,
		MaxPrepTime: preferences.MaxPrepTime,
		MaxCookTime: preferences.MaxCookTime,
		CuisineType: preferences.PreferredCuisineType,
	}

	// Get community recommended recipes
	recommendedRecipes, err := s.GetCommunityRecommendedMeals(userID, mealPlanFilters)
	if err != nil {
		return nil, fmt.Errorf("failed to get community recommendations: %w", err)
	}

	// Generate meal plan with rating-aware recipe selection
	meals := make(map[string]map[string]*models.MealEntry)
	days := []string{"monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"}
	mealTypes := []string{"breakfast", "lunch", "dinner"}

	if preferences.IncludeSnacks {
		mealTypes = append(mealTypes, "snack")
	}

	recipeIndex := 0
	for _, day := range days {
		meals[day] = make(map[string]*models.MealEntry)
		
		for _, mealType := range mealTypes {
			// Skip if user doesn't want this meal type on this day
			if !s.shouldIncludeMeal(day, mealType, preferences) {
				continue
			}

			// Select recipe with preference for highly rated community recipes
			var selectedRecipe *models.Recipe
			
			// First try to find a community recipe that matches this meal type
			for i := recipeIndex; i < len(recommendedRecipes); i++ {
				recipe := &recommendedRecipes[i]
				if s.containsString(recipe.MealType, mealType) {
					selectedRecipe = recipe
					recipeIndex = (i + 1) % len(recommendedRecipes)
					break
				}
			}

			// If no community recipe found, use personal recipes as fallback
			if selectedRecipe == nil {
				personalFilters := &models.RecipeFilters{
					MealType: &[]string{mealType},
				}
				personalRecipes, err := s.recipeRepo.GetSimilarRecipes(userID, personalFilters)
				if err == nil && len(personalRecipes) > 0 {
					selectedRecipe = &personalRecipes[0]
				}
			}

			if selectedRecipe != nil {
				meals[day][mealType] = &models.MealEntry{
					ID:       fmt.Sprintf("%s_%s_%s", day, mealType, selectedRecipe.ID.String()),
					RecipeID: selectedRecipe.ID,
					Day:      day,
					MealType: mealType,
					Servings: selectedRecipe.Servings,
					IsLocked: false,
					Notes:    fmt.Sprintf("Community rating: %.1f/5 (%d reviews)", selectedRecipe.AverageRating, selectedRecipe.TotalRatings),
				}
			}
		}
	}

	// Create meal plan input
	mealPlanInput := &models.CreateMealPlanInput{
		WeekStartDate:  preferences.WeekStartDate,
		GenerationType: "rating_aware_automatic",
		Meals:          meals,
	}

	// Create the meal plan
	return s.CreateMealPlan(userID, mealPlanInput)
}

// Helper methods
func (s *mealPlanService) containsAnyMealType(recipeMealTypes []string, filterMealTypes []string) bool {
	for _, rmt := range recipeMealTypes {
		for _, fmt := range filterMealTypes {
			if rmt == fmt {
				return true
			}
		}
	}
	return false
}

func (s *mealPlanService) containsString(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}

func (s *mealPlanService) shouldIncludeMeal(day, mealType string, preferences *models.MealPlanPreferences) bool {
	// Default to including all meals unless preferences specify otherwise
	if preferences.ExcludedMeals == nil {
		return true
	}

	excludedKey := fmt.Sprintf("%s_%s", day, mealType)
	for _, excluded := range preferences.ExcludedMeals {
		if excluded == excludedKey {
			return false
		}
	}

	return true
}

func (s *mealPlanService) GetMealPlan(id uuid.UUID, userID uuid.UUID) (*models.MealPlanResponse, error) {
	mealPlan, err := s.repo.GetByID(id, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plan: %w", err)
	}

	// Populate with recipe details
	response, err := s.populateMealPlanWithRecipes(mealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

func (s *mealPlanService) GetUserMealPlans(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlanResponse, error) {
	mealPlans, err := s.repo.GetByUserID(userID, filters)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plans: %w", err)
	}

	responses := make([]models.MealPlanResponse, len(mealPlans))
	for i, mealPlan := range mealPlans {
		response, err := s.populateMealPlanWithRecipes(&mealPlan)
		if err != nil {
			return nil, fmt.Errorf("failed to populate meal plan %s with recipes: %w", mealPlan.ID, err)
		}
		responses[i] = *response
	}

	return responses, nil
}

func (s *mealPlanService) GetMealPlanByWeek(userID uuid.UUID, weekStart time.Time) (*models.MealPlanResponse, error) {
	mealPlan, err := s.repo.GetByWeekStart(userID, weekStart)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plan for week: %w", err)
	}

	// Populate with recipe details
	response, err := s.populateMealPlanWithRecipes(mealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

func (s *mealPlanService) UpdateMealPlan(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlanResponse, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	// Update meal plan
	mealPlan, err := s.repo.Update(id, userID, input)
	if err != nil {
		return nil, fmt.Errorf("failed to update meal plan: %w", err)
	}

	// Populate with recipe details
	response, err := s.populateMealPlanWithRecipes(mealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

func (s *mealPlanService) UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlanResponse, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	// Validate meal type
	if err := s.validateMealType(mealType); err != nil {
		return nil, err
	}

	// Validate day
	if err := s.validateDay(day); err != nil {
		return nil, err
	}

	// If recipe ID is provided, validate it exists and user has access
	if input.RecipeID != nil && *input.RecipeID != "" {
		recipeUUID, err := uuid.Parse(*input.RecipeID)
		if err != nil {
			return nil, fmt.Errorf("invalid recipe ID format: %w", err)
		}

		_, err = s.recipeRepo.GetByID(recipeUUID, userID)
		if err != nil {
			return nil, fmt.Errorf("recipe not found or access denied: %w", err)
		}
	}

	// Update meal slot
	mealPlan, err := s.repo.UpdateMealSlot(mealPlanID, userID, day, mealType, input)
	if err != nil {
		return nil, fmt.Errorf("failed to update meal slot: %w", err)
	}

	// Populate with recipe details
	response, err := s.populateMealPlanWithRecipes(mealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

func (s *mealPlanService) DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlanResponse, error) {
	// Validate meal type
	if err := s.validateMealType(mealType); err != nil {
		return nil, err
	}

	// Validate day
	if err := s.validateDay(day); err != nil {
		return nil, err
	}

	// Delete meal slot
	mealPlan, err := s.repo.DeleteMealSlot(mealPlanID, userID, day, mealType)
	if err != nil {
		return nil, fmt.Errorf("failed to delete meal slot: %w", err)
	}

	// Populate with recipe details
	response, err := s.populateMealPlanWithRecipes(mealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

func (s *mealPlanService) DeleteMealPlan(id uuid.UUID, userID uuid.UUID) error {
	return s.repo.Delete(id, userID)
}

// Helper method to populate meal plan with recipe details
func (s *mealPlanService) populateMealPlanWithRecipes(mealPlan *models.MealPlan) (*models.MealPlanResponse, error) {
	response := &models.MealPlanResponse{
		MealPlan: *mealPlan,
	}

	// Parse meals from JSONB
	var weeklyMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &weeklyMeals); err != nil {
		return nil, fmt.Errorf("failed to unmarshal meals: %w", err)
	}

	// Populate recipe details for each meal slot
	populatedWeek := models.WeeklyMealsWithRecipes{}

	populatedWeek.Monday = s.populateDayMealsWithRecipes(weeklyMeals.Monday, mealPlan.UserID)
	populatedWeek.Tuesday = s.populateDayMealsWithRecipes(weeklyMeals.Tuesday, mealPlan.UserID)
	populatedWeek.Wednesday = s.populateDayMealsWithRecipes(weeklyMeals.Wednesday, mealPlan.UserID)
	populatedWeek.Thursday = s.populateDayMealsWithRecipes(weeklyMeals.Thursday, mealPlan.UserID)
	populatedWeek.Friday = s.populateDayMealsWithRecipes(weeklyMeals.Friday, mealPlan.UserID)
	populatedWeek.Saturday = s.populateDayMealsWithRecipes(weeklyMeals.Saturday, mealPlan.UserID)
	populatedWeek.Sunday = s.populateDayMealsWithRecipes(weeklyMeals.Sunday, mealPlan.UserID)

	response.PopulatedMeals = populatedWeek
	return response, nil
}

// Helper method to populate a day's meals with recipe details
func (s *mealPlanService) populateDayMealsWithRecipes(dayMeals []models.MealSlot, userID uuid.UUID) []models.MealSlotWithRecipe {
	populated := make([]models.MealSlotWithRecipe, len(dayMeals))

	for i, meal := range dayMeals {
		populated[i] = models.MealSlotWithRecipe{
			MealSlot: meal,
		}

		// If recipe ID is present, fetch the recipe details
		if meal.RecipeID != nil && *meal.RecipeID != "" {
			recipeUUID, err := uuid.Parse(*meal.RecipeID)
			if err == nil {
				recipe, err := s.recipeRepo.GetByID(recipeUUID, userID)
				if err == nil {
					populated[i].Recipe = recipe
				}
			}
		}
	}

	return populated
}

// Helper method to calculate total estimated cooking time for a meal plan
func (s *mealPlanService) calculateTotalTime(meals *models.WeeklyMeals, userID uuid.UUID) int {
	totalTime := 0

	// Helper function to calculate time for a day's meals
	calculateDayTime := func(dayMeals []models.MealSlot) {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil && *meal.RecipeID != "" {
				recipeUUID, err := uuid.Parse(*meal.RecipeID)
				if err == nil {
					recipe, err := s.recipeRepo.GetByID(recipeUUID, userID)
					if err == nil {
						totalTime += recipe.PrepTime + recipe.CookTime
					}
				}
			}
		}
	}

	calculateDayTime(meals.Monday)
	calculateDayTime(meals.Tuesday)
	calculateDayTime(meals.Wednesday)
	calculateDayTime(meals.Thursday)
	calculateDayTime(meals.Friday)
	calculateDayTime(meals.Saturday)
	calculateDayTime(meals.Sunday)

	return totalTime
}

// Helper method to validate meal type
func (s *mealPlanService) validateMealType(mealType string) error {
	validMealTypes := map[string]bool{
		"breakfast": true,
		"lunch":     true,
		"dinner":    true,
	}

	if !validMealTypes[mealType] {
		return fmt.Errorf("invalid meal type: %s, must be one of: breakfast, lunch, dinner", mealType)
	}

	return nil
}

// Helper method to validate day
func (s *mealPlanService) validateDay(day string) error {
	validDays := map[string]bool{
		"monday":    true,
		"tuesday":   true,
		"wednesday": true,
		"thursday":  true,
		"friday":    true,
		"saturday":  true,
		"sunday":    true,
	}

	if !validDays[day] {
		return fmt.Errorf("invalid day: %s, must be one of: monday, tuesday, wednesday, thursday, friday, saturday, sunday", day)
	}

	return nil
}

// UpdateMealEntry updates a specific meal entry with substitution tracking
func (s *mealPlanService) UpdateMealEntry(mealPlanID uuid.UUID, entryID string, userID uuid.UUID, input *models.MealEntryUpdateRequest) (*models.MealPlanResponse, error) {
	// Validate input
	if err := s.validator.Struct(input); err != nil {
		return nil, fmt.Errorf("validation error: %w", err)
	}

	// Get current meal plan to capture before state
	mealPlan, err := s.repo.GetByID(mealPlanID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plan: %w", err)
	}

	// Parse current meals to find the entry
	var currentMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &currentMeals); err != nil {
		return nil, fmt.Errorf("failed to unmarshal current meals: %w", err)
	}

	// Find and update the specific meal entry
	var foundSlot *models.MealSlot
	var day string
	var slotIndex int

	// Check all days to find the entry
	days := map[string]*[]models.MealSlot{
		"monday":    &currentMeals.Monday,
		"tuesday":   &currentMeals.Tuesday,
		"wednesday": &currentMeals.Wednesday,
		"thursday":  &currentMeals.Thursday,
		"friday":    &currentMeals.Friday,
		"saturday":  &currentMeals.Saturday,
		"sunday":    &currentMeals.Sunday,
	}

	for dayName, dayMeals := range days {
		for i, slot := range *dayMeals {
			if slot.RecipeID != nil && *slot.RecipeID == entryID {
				foundSlot = &slot
				day = dayName
				slotIndex = i
				break
			}
		}
		if foundSlot != nil {
			break
		}
	}

	if foundSlot == nil {
		return nil, fmt.Errorf("meal entry not found: %s", entryID)
	}

	// Check if meal is locked
	if foundSlot.IsLocked {
		return nil, fmt.Errorf("cannot modify locked meal entry")
	}

	// Validate new recipe exists and user has access
	newRecipeUUID, err := uuid.Parse(input.RecipeID)
	if err != nil {
		return nil, fmt.Errorf("invalid recipe ID format: %w", err)
	}

	newRecipe, err := s.recipeRepo.GetByID(newRecipeUUID, userID)
	if err != nil {
		return nil, fmt.Errorf("recipe not found or access denied: %w", err)
	}

	// Capture before state
	beforeState := models.ChangeState{
		EntryID: &entryID,
		MealSlots: []models.MealSlot{*foundSlot},
		Metadata: map[string]interface{}{
			"day":       day,
			"slotIndex": slotIndex,
		},
	}

	// Update the meal slot
	foundSlot.RecipeID = &input.RecipeID
	if input.IsLocked != nil {
		foundSlot.IsLocked = *input.IsLocked
	}

	// Update the day's meals
	*days[day] = append((*days[day])[:slotIndex], append([]models.MealSlot{*foundSlot}, (*days[day])[slotIndex+1:]...)...)

	// Capture after state
	afterState := models.ChangeState{
		EntryID: &entryID,
		MealSlots: []models.MealSlot{*foundSlot},
		Metadata: map[string]interface{}{
			"day":       day,
			"slotIndex": slotIndex,
		},
	}

	// Update meal plan in database
	mealsJSON, err := json.Marshal(currentMeals)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal updated meals: %w", err)
	}

	updateInput := &models.UpdateMealPlanInput{
		Meals: &currentMeals,
	}

	updatedMealPlan, err := s.repo.Update(mealPlanID, userID, updateInput)
	if err != nil {
		return nil, fmt.Errorf("failed to update meal plan: %w", err)
	}

	// Record change history
	beforeStateJSON, _ := json.Marshal(beforeState)
	afterStateJSON, _ := json.Marshal(afterState)

	changeHistory := &models.MealPlanChangeHistory{
		MealPlanID:   mealPlanID,
		UserID:       userID,
		ChangeType:   "substitution",
		BeforeState:  beforeStateJSON,
		AfterState:   afterStateJSON,
		ChangeReason: &input.ChangeReason,
	}

	if err := s.changeHistoryRepo.Create(changeHistory); err != nil {
		// Log error but don't fail the operation
		fmt.Printf("Warning: failed to record change history: %v\n", err)
	}

	// Clean up old changes (keep max 20)
	s.changeHistoryRepo.DeleteOldChanges(mealPlanID, userID, 20)

	// Populate response with recipe details
	response, err := s.populateMealPlanWithRecipes(updatedMealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

// GetSwapSuggestions provides recipe suggestions for meal substitution
func (s *mealPlanService) GetSwapSuggestions(mealPlanID uuid.UUID, entryID string, userID uuid.UUID) ([]models.Recipe, error) {
	// Get current meal plan to understand the context
	mealPlan, err := s.repo.GetByID(mealPlanID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plan: %w", err)
	}

	// Parse meals to find the current recipe
	var currentMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &currentMeals); err != nil {
		return nil, fmt.Errorf("failed to unmarshal meals: %w", err)
	}

	// Find the current recipe and meal type
	var currentRecipeID string
	var mealType string

	found := false
	allSlots := s.getAllMealSlots(currentMeals)
	for _, slot := range allSlots {
		if slot.RecipeID != nil && *slot.RecipeID == entryID {
			currentRecipeID = *slot.RecipeID
			mealType = slot.MealType
			found = true
			break
		}
	}

	if !found {
		return nil, fmt.Errorf("meal entry not found: %s", entryID)
	}

	// Get current recipe details
	currentRecipeUUID, err := uuid.Parse(currentRecipeID)
	if err != nil {
		return nil, fmt.Errorf("invalid current recipe ID: %w", err)
	}

	currentRecipe, err := s.recipeRepo.GetByID(currentRecipeUUID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get current recipe: %w", err)
	}

	// Get recipe suggestions based on similarity
	suggestions, err := s.recipeRepo.GetSimilarRecipes(userID, &models.RecipeFilters{
		MealType:   &mealType,
		MaxPrepTime: &currentRecipe.PrepTime,
		Complexity: &currentRecipe.Complexity,
		ExcludeIDs: []uuid.UUID{currentRecipeUUID},
	})

	if err != nil {
		return nil, fmt.Errorf("failed to get recipe suggestions: %w", err)
	}

	return suggestions, nil
}

// GetChangeHistory retrieves the change history for a meal plan
func (s *mealPlanService) GetChangeHistory(mealPlanID uuid.UUID, userID uuid.UUID, filters *models.ChangeHistoryFilters) (*models.ChangeHistoryResponse, error) {
	changes, err := s.changeHistoryRepo.GetByMealPlanID(mealPlanID, userID, filters)
	if err != nil {
		return nil, fmt.Errorf("failed to get change history: %w", err)
	}

	total, err := s.changeHistoryRepo.CountChanges(mealPlanID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to count changes: %w", err)
	}

	response := &models.ChangeHistoryResponse{
		Changes: changes,
		CanUndo: len(changes) > 0,
		CanRedo: false, // TODO: Implement redo functionality
		Total:   int(total),
	}

	return response, nil
}

// UndoLastChange undoes the most recent change to the meal plan
func (s *mealPlanService) UndoLastChange(mealPlanID uuid.UUID, userID uuid.UUID) (*models.MealPlanResponse, error) {
	// Get the latest change
	changes, err := s.changeHistoryRepo.GetLatestChanges(mealPlanID, userID, 1)
	if err != nil {
		return nil, fmt.Errorf("failed to get latest changes: %w", err)
	}

	if len(changes) == 0 {
		return nil, fmt.Errorf("no changes to undo")
	}

	latestChange := changes[0]

	// Parse the before state
	var beforeState models.ChangeState
	if err := json.Unmarshal(latestChange.BeforeState, &beforeState); err != nil {
		return nil, fmt.Errorf("failed to parse before state: %w", err)
	}

	// Get current meal plan
	mealPlan, err := s.repo.GetByID(mealPlanID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plan: %w", err)
	}

	// Apply the before state to restore the meal plan
	var currentMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &currentMeals); err != nil {
		return nil, fmt.Errorf("failed to unmarshal current meals: %w", err)
	}

	// Restore the meal slots based on the change type
	if err := s.restoreMealState(&currentMeals, &beforeState, latestChange.ChangeType); err != nil {
		return nil, fmt.Errorf("failed to restore meal state: %w", err)
	}

	// Update the meal plan
	updateInput := &models.UpdateMealPlanInput{
		Meals: &currentMeals,
	}

	updatedMealPlan, err := s.repo.Update(mealPlanID, userID, updateInput)
	if err != nil {
		return nil, fmt.Errorf("failed to update meal plan: %w", err)
	}

	// Populate response
	response, err := s.populateMealPlanWithRecipes(updatedMealPlan)
	if err != nil {
		return nil, fmt.Errorf("failed to populate meal plan with recipes: %w", err)
	}

	return response, nil
}

// Helper method to get all meal slots from weekly meals
func (s *mealPlanService) getAllMealSlots(meals models.WeeklyMeals) []models.MealSlot {
	var allSlots []models.MealSlot
	
	days := [][]models.MealSlot{
		meals.Monday, meals.Tuesday, meals.Wednesday, meals.Thursday,
		meals.Friday, meals.Saturday, meals.Sunday,
	}
	
	for _, dayMeals := range days {
		allSlots = append(allSlots, dayMeals...)
	}
	
	return allSlots
}

// Helper method to restore meal state from change history
func (s *mealPlanService) restoreMealState(currentMeals *models.WeeklyMeals, beforeState *models.ChangeState, changeType string) error {
	// This is a simplified restoration - in production you'd want more sophisticated logic
	// For now, we'll handle the substitution case
	
	if changeType == "substitution" && beforeState.EntryID != nil && len(beforeState.MealSlots) > 0 {
		entryID := *beforeState.EntryID
		restoredSlot := beforeState.MealSlots[0]
		
		// Find and restore the meal slot
		days := map[string]*[]models.MealSlot{
			"monday":    &currentMeals.Monday,
			"tuesday":   &currentMeals.Tuesday,
			"wednesday": &currentMeals.Wednesday,
			"thursday":  &currentMeals.Thursday,
			"friday":    &currentMeals.Friday,
			"saturday":  &currentMeals.Saturday,
			"sunday":    &currentMeals.Sunday,
		}

		for _, dayMeals := range days {
			for i, slot := range *dayMeals {
				if slot.RecipeID != nil && *slot.RecipeID == entryID {
					(*dayMeals)[i] = restoredSlot
					return nil
				}
			}
		}
	}
	
	return nil
}
