package services

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// ShoppingListService handles shopping list business logic
type ShoppingListService struct {
	shoppingRepo    repositories.ShoppingListRepository
	mealPlanRepo    repositories.MealPlanRepository
	recipeRepo      repositories.RecipeRepository
	unitConverter   *UnitConversionService
	cacheService    *CacheService
}

// NewShoppingListService creates a new shopping list service
func NewShoppingListService(
	shoppingRepo repositories.ShoppingListRepository,
	mealPlanRepo repositories.MealPlanRepository,
	recipeRepo repositories.RecipeRepository,
	cacheService *CacheService,
) *ShoppingListService {
	return &ShoppingListService{
		shoppingRepo:  shoppingRepo,
		mealPlanRepo:  mealPlanRepo,
		recipeRepo:    recipeRepo,
		unitConverter: NewUnitConversionService(),
		cacheService:  cacheService,
	}
}

// GenerateFromMealPlan generates a shopping list from a meal plan
func (s *ShoppingListService) GenerateFromMealPlan(userID, mealPlanID uuid.UUID, mergeExisting bool) (*models.ShoppingListResponse, error) {
	// 1. Retrieve meal plan with recipe details
	mealPlan, err := s.mealPlanRepo.GetByID(mealPlanID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get meal plan: %w", err)
	}

	if mealPlan == nil {
		return nil, fmt.Errorf("meal plan not found")
	}

	// 2. Extract recipe IDs and fetch recipes
	recipeIDs := s.extractRecipeIDs(mealPlan)
	if len(recipeIDs) == 0 {
		return nil, fmt.Errorf("no recipes found in meal plan")
	}

	recipes, err := s.fetchRecipes(recipeIDs, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch recipes: %w", err)
	}

	// 3. Aggregate ingredients with quantity combining
	aggregatedIngredients := s.aggregateIngredients(recipes)

	// 4. Create shopping list
	shoppingList := &models.ShoppingList{
		ID:          uuid.New(),
		UserID:      userID,
		MealPlanID:  &mealPlanID,
		Name:        s.generateShoppingListName(mealPlan),
		Status:      models.ShoppingListStatusActive,
		GeneratedAt: time.Now(),
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
	}

	// 5. Create shopping items
	items := s.createShoppingItems(shoppingList.ID, aggregatedIngredients)

	// 6. Persist to database
	if err := s.shoppingRepo.Create(shoppingList); err != nil {
		return nil, fmt.Errorf("failed to create shopping list: %w", err)
	}

	if err := s.shoppingRepo.CreateItems(items); err != nil {
		return nil, fmt.Errorf("failed to create shopping items: %w", err)
	}

	// 7. Return response
	return shoppingList.ToResponse(items), nil
}

// GetShoppingList retrieves a shopping list with its items
func (s *ShoppingListService) GetShoppingList(userID, listID uuid.UUID) (*models.ShoppingListResponse, error) {
	list, items, err := s.shoppingRepo.GetWithItems(listID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get shopping list: %w", err)
	}

	if list == nil {
		return nil, fmt.Errorf("shopping list not found")
	}

	return list.ToResponse(items), nil
}

// GetUserShoppingLists retrieves all shopping lists for a user
func (s *ShoppingListService) GetUserShoppingLists(userID uuid.UUID, status, sortBy string) ([]models.ShoppingListResponse, error) {
	lists, err := s.shoppingRepo.GetByUserID(userID, status, sortBy)
	if err != nil {
		return nil, fmt.Errorf("failed to get shopping lists: %w", err)
	}

	var responses []models.ShoppingListResponse
	for _, list := range lists {
		items, err := s.shoppingRepo.GetItemsByListID(list.ID)
		if err != nil {
			return nil, fmt.Errorf("failed to get items for list %s: %w", list.ID, err)
		}
		responses = append(responses, *list.ToResponse(items))
	}

	return responses, nil
}

// UpdateItem updates a shopping list item
func (s *ShoppingListService) UpdateItem(userID, listID, itemID uuid.UUID, updates *models.ShoppingItemUpdateRequest) error {
	// Verify ownership
	_, err := s.shoppingRepo.GetByID(listID, userID)
	if err != nil {
		return fmt.Errorf("shopping list not found or access denied: %w", err)
	}

	return s.shoppingRepo.UpdateItem(itemID, updates)
}

// ExportShoppingList exports a shopping list in the specified format
func (s *ShoppingListService) ExportShoppingList(userID, listID uuid.UUID, format string, includeRecipeSources bool) ([]byte, string, error) {
	list, items, err := s.shoppingRepo.GetWithItems(listID, userID)
	if err != nil {
		return nil, "", fmt.Errorf("failed to get shopping list: %w", err)
	}

	if list == nil {
		return nil, "", fmt.Errorf("shopping list not found")
	}

	switch strings.ToLower(format) {
	case "json":
		return s.exportAsJSON(list, items, includeRecipeSources)
	case "csv":
		return s.exportAsCSV(list, items, includeRecipeSources)
	case "txt":
		return s.exportAsText(list, items, includeRecipeSources)
	default:
		return nil, "", fmt.Errorf("unsupported export format: %s", format)
	}
}

// DeleteShoppingList deletes a shopping list
func (s *ShoppingListService) DeleteShoppingList(userID, listID uuid.UUID) error {
	return s.shoppingRepo.Delete(listID, userID)
}

// extractRecipeIDs extracts unique recipe IDs from a meal plan
func (s *ShoppingListService) extractRecipeIDs(mealPlan *models.MealPlan) []uuid.UUID {
	recipeIDMap := make(map[uuid.UUID]bool)
	
	// First, try to extract from structured Entries
	if len(mealPlan.Entries) > 0 {
		for _, entry := range mealPlan.Entries {
			if recipeID, err := uuid.Parse(entry.RecipeID); err == nil {
				recipeIDMap[recipeID] = true
			}
		}
	} else if len(mealPlan.Meals) > 0 {
		// Fallback: Parse meals from JSONB structure
		var mealsData map[string]interface{}
		if err := json.Unmarshal(mealPlan.Meals, &mealsData); err == nil {
			for _, dayData := range mealsData {
				if dayMap, ok := dayData.(map[string]interface{}); ok {
					for _, mealData := range dayMap {
						if mealMap, ok := mealData.(map[string]interface{}); ok {
							if recipeIDStr, ok := mealMap["recipeId"].(string); ok {
								if recipeID, err := uuid.Parse(recipeIDStr); err == nil {
									recipeIDMap[recipeID] = true
								}
							}
						}
					}
				}
			}
		}
	}
	
	// Convert map keys to slice
	var recipeIDs []uuid.UUID
	for id := range recipeIDMap {
		recipeIDs = append(recipeIDs, id)
	}
	
	return recipeIDs
}

// fetchRecipes fetches multiple recipes by IDs
func (s *ShoppingListService) fetchRecipes(recipeIDs []uuid.UUID, userID uuid.UUID) ([]*models.Recipe, error) {
	var recipes []*models.Recipe
	
	for _, recipeID := range recipeIDs {
		recipe, err := s.recipeRepo.GetByID(recipeID, userID)
		if err != nil {
			continue // Skip recipes that can't be found
		}
		if recipe != nil {
			recipes = append(recipes, recipe)
		}
	}

	return recipes, nil
}

// aggregateIngredients combines ingredients with quantity aggregation
func (s *ShoppingListService) aggregateIngredients(recipes []*models.Recipe) map[string]*models.AggregatedIngredient {
	ingredientMap := make(map[string]*models.AggregatedIngredient)

	for _, recipe := range recipes {
		// Parse ingredients from JSON
		var ingredients []models.RecipeIngredient
		if err := json.Unmarshal(recipe.Ingredients, &ingredients); err != nil {
			continue // Skip recipes with invalid ingredient data
		}

		for _, ingredient := range ingredients {
			key := s.unitConverter.GenerateIngredientKey(ingredient.Name, ingredient.Unit)

			if existing, exists := ingredientMap[key]; exists {
				// Combine quantities with unit conversion
				compatibleUnit := s.unitConverter.GetCompatibleUnit(existing.Unit, ingredient.Unit)
				existingAmount := s.unitConverter.Convert(existing.Amount, existing.Unit, compatibleUnit)
				newAmount := s.unitConverter.Convert(ingredient.Amount, ingredient.Unit, compatibleUnit)

				existing.Amount = existingAmount + newAmount
				existing.Unit = compatibleUnit
				existing.RecipeSources = append(existing.RecipeSources, recipe.ID)
			} else {
				ingredientMap[key] = &models.AggregatedIngredient{
					Name:          ingredient.Name,
					Amount:        ingredient.Amount,
					Unit:          ingredient.Unit,
					Category:      ingredient.Category,
					RecipeSources: []uuid.UUID{recipe.ID},
				}
			}
		}
	}

	return ingredientMap
}

// createShoppingItems converts aggregated ingredients to shopping items
func (s *ShoppingListService) createShoppingItems(listID uuid.UUID, aggregatedIngredients map[string]*models.AggregatedIngredient) []models.ShoppingItem {
	var items []models.ShoppingItem

	for _, ingredient := range aggregatedIngredients {
		// Ensure category is valid
		category := ingredient.Category
		if !models.IsValidCategory(category) {
			category = models.CategoryOther
		}

		item := models.ShoppingItem{
			ID:               uuid.New(),
			ShoppingListID:   listID,
			IngredientName:   ingredient.Name,
			Amount:           ingredient.Amount,
			Unit:             ingredient.Unit,
			Category:         category,
			IsCompleted:      false,
			RecipeSources:    models.UUIDArray(ingredient.RecipeSources),
			CreatedAt:        time.Now(),
			UpdatedAt:        time.Now(),
		}

		items = append(items, item)
	}

	return items
}

// generateShoppingListName creates a descriptive name for the shopping list
func (s *ShoppingListService) generateShoppingListName(mealPlan *models.MealPlan) string {
	if mealPlan.Name != "" {
		return fmt.Sprintf("Shopping List - %s", mealPlan.Name)
	}
	return fmt.Sprintf("Shopping List - %s", mealPlan.WeekStart.Format("Jan 2, 2006"))
}

// exportAsJSON exports the shopping list as JSON
func (s *ShoppingListService) exportAsJSON(list *models.ShoppingList, items []models.ShoppingItem, includeRecipeSources bool) ([]byte, string, error) {
	response := list.ToResponse(items)
	
	if !includeRecipeSources {
		// Remove recipe sources from items
		for categoryName, categoryItems := range response.Categories {
			for i := range categoryItems {
				categoryItems[i].RecipeSources = nil
			}
			response.Categories[categoryName] = categoryItems
		}
	}

	data, err := json.Marshal(response)
	if err != nil {
		return nil, "", fmt.Errorf("failed to marshal JSON: %w", err)
	}

	filename := fmt.Sprintf("shopping_list_%s.json", list.ID.String()[:8])
	return data, filename, nil
}

// exportAsCSV exports the shopping list as CSV
func (s *ShoppingListService) exportAsCSV(list *models.ShoppingList, items []models.ShoppingItem, includeRecipeSources bool) ([]byte, string, error) {
	var csvData strings.Builder
	
	// Header
	if includeRecipeSources {
		csvData.WriteString("Category,Item,Amount,Unit,Completed,Notes,Recipe Sources\n")
	} else {
		csvData.WriteString("Category,Item,Amount,Unit,Completed,Notes\n")
	}

	// Data rows
	for _, item := range items {
		completedStr := "No"
		if item.IsCompleted {
			completedStr = "Yes"
		}

		notes := ""
		if item.Notes != nil {
			notes = *item.Notes
		}

		if includeRecipeSources {
			recipeSourcesStr := ""
			if len(item.RecipeSources) > 0 {
				var sourceStrs []string
				for _, source := range item.RecipeSources {
					sourceStrs = append(sourceStrs, source.String())
				}
				recipeSourcesStr = strings.Join(sourceStrs, "; ")
			}

			csvData.WriteString(fmt.Sprintf("%s,\"%s\",%.2f,%s,%s,\"%s\",\"%s\"\n",
				item.Category, item.IngredientName, item.Amount, item.Unit,
				completedStr, notes, recipeSourcesStr))
		} else {
			csvData.WriteString(fmt.Sprintf("%s,\"%s\",%.2f,%s,%s,\"%s\"\n",
				item.Category, item.IngredientName, item.Amount, item.Unit,
				completedStr, notes))
		}
	}

	filename := fmt.Sprintf("shopping_list_%s.csv", list.ID.String()[:8])
	return []byte(csvData.String()), filename, nil
}

// exportAsText exports the shopping list as plain text
func (s *ShoppingListService) exportAsText(list *models.ShoppingList, items []models.ShoppingItem, includeRecipeSources bool) ([]byte, string, error) {
	var textData strings.Builder
	
	textData.WriteString(fmt.Sprintf("Shopping List: %s\n", list.Name))
	textData.WriteString(fmt.Sprintf("Generated: %s\n\n", list.GeneratedAt.Format("January 2, 2006")))

	// Group by category
	categoryItems := make(map[string][]models.ShoppingItem)
	for _, item := range items {
		categoryItems[item.Category] = append(categoryItems[item.Category], item)
	}

	// Write each category
	categories := []string{
		models.CategoryProduce,
		models.CategoryDairy,
		models.CategoryPantry,
		models.CategoryProtein,
		models.CategoryOther,
	}

	for _, category := range categories {
		if items, exists := categoryItems[category]; exists && len(items) > 0 {
			textData.WriteString(fmt.Sprintf("%s:\n", strings.Title(category)))
			for _, item := range items {
				checkbox := "☐"
				if item.IsCompleted {
					checkbox = "☑"
				}

				textData.WriteString(fmt.Sprintf("  %s %.2f %s %s\n", 
					checkbox, item.Amount, item.Unit, item.IngredientName))

				if item.Notes != nil && *item.Notes != "" {
					textData.WriteString(fmt.Sprintf("      Note: %s\n", *item.Notes))
				}
			}
			textData.WriteString("\n")
		}
	}

	filename := fmt.Sprintf("shopping_list_%s.txt", list.ID.String()[:8])
	return []byte(textData.String()), filename, nil
}