package services

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"sort"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// RotationState tracks recipe usage to prevent repeats with global persistence
type RotationState struct {
	UsedRecipes        map[string]time.Time `json:"usedRecipes"`        // Recipe ID -> last used date
	CycleCount         int                  `json:"cycleCount"`         // Number of complete rotations
	TotalRecipesUsed   int                  `json:"totalRecipesUsed"`   // Total unique recipes used
	LastResetDate      *time.Time           `json:"lastResetDate"`      // When the rotation was last reset
	MealTypeHistory    map[string][]string  `json:"mealTypeHistory"`    // Track recent meals by type
	ComplexityHistory  []string             `json:"complexityHistory"`  // Recent complexity levels
	
	// Enhanced global persistence tracking
	WeeklyHistory      []WeekRotationData   `json:"weeklyHistory"`      // Historical data per week
	GlobalRotationPool []string             `json:"globalRotationPool"` // All available recipe IDs for rotation
	LastUpdateWeek     string               `json:"lastUpdateWeek"`     // ISO week string (YYYY-Www)
	ConsecutiveWeeks   int                  `json:"consecutiveWeeks"`   // Weeks of continuous rotation tracking
}

// WeekRotationData captures rotation data for a specific week
type WeekRotationData struct {
	Week              string            `json:"week"`              // ISO week string (YYYY-Www)
	RecipesUsed       []string          `json:"recipesUsed"`       // Recipe IDs used this week
	ComplexityPattern []string          `json:"complexityPattern"` // Complexity progression through the week
	VarietyScore      float64           `json:"varietyScore"`      // Calculated variety score for this week
	GeneratedAt       time.Time         `json:"generatedAt"`       // When this week's plan was generated
}

// RecipeSelectionCriteria holds criteria for recipe selection
type RecipeSelectionCriteria struct {
	MealType           string
	Day                string  // monday, tuesday, etc.
	DayOfWeek          int     // 0=Sunday, 6=Saturday
	AvoidRecipeIDs     []string
	MaxPrepTime        *int
	PreferredComplexity []string
	DietaryRestrictions []string
	CuisinePreferences []string
	UsedThisWeek       map[string]bool
	WeeklyPattern      *models.UserWeeklyPattern // Pattern-specific constraints
}

// RotationService manages recipe rotation and intelligent meal plan generation
type RotationService interface {
	GetRotationState(userID uuid.UUID) (*RotationState, error)
	UpdateRotationState(userID uuid.UUID, state *RotationState) error
	SelectRecipesForWeek(userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, error)
	SelectRecipesForWeekWithConstraintHandling(userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, *RotationConstraintReport, error)
	SelectRecipesForWeekWithPatterns(userID uuid.UUID, preferences *models.UserPreferences, weeklyPatterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error)
	AssignRecipeForDay(day time.Weekday, userPatterns []models.UserWeeklyPattern, criteria *RecipeSelectionCriteria, recipes []models.Recipe, rotationState *RotationState) (*models.Recipe, error)
	ResetRotationCycle(userID uuid.UUID) error
	ResetRotationCycleWithOptions(userID uuid.UUID, req *models.RotationResetRequest) error
	GetVarietyScore(recipeIDs []string, userID uuid.UUID) (float64, error)
	GetRotationAnalytics(userID uuid.UUID, weeks int) (*models.RotationAnalytics, error)
	ExportRotationData(userID uuid.UUID, format string, startDate, endDate time.Time) ([]byte, error)
}

// RotationUserPreferences type alias for backward compatibility  
type RotationUserPreferences = models.UserPreferences

// CacheServiceInterface defines the interface expected by rotation service
type CacheServiceInterface interface {
	Get(key string) string
	Set(key, value string, duration time.Duration) error
	Delete(key string) error
	Exists(key string) bool
}

type rotationService struct {
	recipeRepo     repositories.RecipeRepository
	userRepo       repositories.UserRepository
	preferenceRepo *repositories.PreferenceRepository
	cache          CacheServiceInterface
}

func NewRotationService(
	recipeRepo repositories.RecipeRepository,
	userRepo repositories.UserRepository,
	preferenceRepo *repositories.PreferenceRepository,
	cache CacheServiceInterface,
) RotationService {
	return &rotationService{
		recipeRepo:     recipeRepo,
		userRepo:       userRepo,
		preferenceRepo: preferenceRepo,
		cache:          cache,
	}
}

// GetRotationState retrieves the current rotation state for a user
func (r *rotationService) GetRotationState(userID uuid.UUID) (*RotationState, error) {
	cacheKey := fmt.Sprintf("rotation_state:%s", userID.String())
	
	// Try to get from cache first
	if cached := r.cache.Get(cacheKey); cached != "" {
		var state RotationState
		if err := json.Unmarshal([]byte(cached), &state); err == nil {
			return &state, nil
		}
	}

	// Get user's preference data (stored in JSONB field)
	user, err := r.userRepo.GetByID(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user: %w", err)
	}

	// Initialize or parse existing rotation state
	state := &RotationState{
		UsedRecipes:       make(map[string]time.Time),
		CycleCount:        0,
		TotalRecipesUsed:  0,
		MealTypeHistory:   make(map[string][]string),
		ComplexityHistory: make([]string, 0),
		WeeklyHistory:     make([]WeekRotationData, 0),
		GlobalRotationPool: make([]string, 0),
		LastUpdateWeek:    getCurrentISOWeek(),
		ConsecutiveWeeks:  0,
	}

	// Parse rotation state from user's preference learning data
	if user.PreferenceLearningData != nil {
		var userData map[string]interface{}
		if err := json.Unmarshal(user.PreferenceLearningData, &userData); err == nil {
			if rotationData, exists := userData["rotationState"]; exists {
				if rotationJSON, err := json.Marshal(rotationData); err == nil {
					json.Unmarshal(rotationJSON, state)
				}
			}
		}
	}

	// Cache for 1 hour
	if stateJSON, err := json.Marshal(state); err == nil {
		r.cache.Set(cacheKey, string(stateJSON), time.Hour)
	}

	return state, nil
}

// UpdateRotationState saves the rotation state for a user
func (r *rotationService) UpdateRotationState(userID uuid.UUID, state *RotationState) error {
	// Update cache
	cacheKey := fmt.Sprintf("rotation_state:%s", userID.String())
	if stateJSON, err := json.Marshal(state); err == nil {
		r.cache.Set(cacheKey, string(stateJSON), time.Hour)
	}

	// Update user's preference learning data
	user, err := r.userRepo.GetByID(userID)
	if err != nil {
		return fmt.Errorf("failed to get user: %w", err)
	}

	// Parse existing preference data
	var userData map[string]interface{}
	if user.PreferenceLearningData != nil {
		json.Unmarshal(user.PreferenceLearningData, &userData)
	} else {
		userData = make(map[string]interface{})
	}

	// Update rotation state
	userData["rotationState"] = state

	// Save back to user
	updatedData, err := json.Marshal(userData)
	if err != nil {
		return fmt.Errorf("failed to marshal preference data: %w", err)
	}

	return r.userRepo.UpdatePreferenceLearningData(userID, updatedData)
}

// SelectRecipesForWeek generates a complete weekly meal plan with intelligent rotation
func (r *rotationService) SelectRecipesForWeek(userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, error) {
	// Get current rotation state
	rotationState, err := r.GetRotationState(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get rotation state: %w", err)
	}

	// Get user's available recipes
	filters := &models.RecipeFilters{
		DietaryLabels: preferences.DietaryRestrictions,
	}

	// Apply complexity preferences based on cooking skill level
	if preferences.CookingSkillLevel == "beginner" {
		filters.Complexity = []string{"simple"}
	} else if preferences.CookingSkillLevel == "advanced" {
		filters.Complexity = []string{"simple", "moderate", "complex"}
	} else {
		// Intermediate gets simple and moderate
		filters.Complexity = []string{"simple", "moderate"}
	}

	// Get available recipes
	searchParams := &models.RecipeSearchParams{
		RecipeFilters: *filters,
		Page:         1,
		Limit:        100, // Get a large pool for selection
		SortBy:       "average_rating",
		SortOrder:    "desc",
	}

	searchResult, err := r.recipeRepo.Search(userID, searchParams)
	if err != nil {
		return nil, fmt.Errorf("failed to search recipes: %w", err)
	}

	if len(searchResult.Recipes) < 21 {
		return nil, fmt.Errorf("insufficient recipes available (need at least 21, found %d)", len(searchResult.Recipes))
	}

	// Generate intelligent weekly meal plan
	weeklyMeals := &models.WeeklyMeals{
		Monday:    make([]models.MealSlot, 3),
		Tuesday:   make([]models.MealSlot, 3),
		Wednesday: make([]models.MealSlot, 3),
		Thursday:  make([]models.MealSlot, 3),
		Friday:    make([]models.MealSlot, 3),
		Saturday:  make([]models.MealSlot, 3),
		Sunday:    make([]models.MealSlot, 3),
	}

	usedRecipesThisWeek := make(map[string]bool)
	days := []string{"monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"}
	mealTypes := []string{"breakfast", "lunch", "dinner"}

	// Generate meals for each day
	for dayIndex, day := range days {
		dayMeals := make([]models.MealSlot, 3)
		
		for mealIndex, mealType := range mealTypes {
			criteria := &RecipeSelectionCriteria{
				MealType:            mealType,
				Day:                 day,
				MaxPrepTime:         &preferences.MaxPrepTimePerMeal,
				DietaryRestrictions: preferences.DietaryRestrictions,
				CuisinePreferences:  preferences.CuisinePreferences,
				UsedThisWeek:        usedRecipesThisWeek,
			}

			// Add used recipes to avoid list
			for recipeID := range rotationState.UsedRecipes {
				criteria.AvoidRecipeIDs = append(criteria.AvoidRecipeIDs, recipeID)
			}

			// Apply complexity balancing
			criteria.PreferredComplexity = r.getBalancedComplexity(rotationState, dayIndex, mealIndex, preferences.CookingSkillLevel)

			// Select recipe for this meal slot with fallback mechanisms
			selectedRecipe, err := r.selectBestRecipeWithFallbacks(searchResult.Recipes, criteria, rotationState)
			if err != nil {
				return nil, fmt.Errorf("failed to select recipe for %s %s: %w", day, mealType, err)
			}

			// Create meal slot
			recipeID := selectedRecipe.ID.String()
			dayMeals[mealIndex] = models.MealSlot{
				Day:      day,
				MealType: mealType,
				RecipeID: &recipeID,
				Servings: selectedRecipe.Servings,
			}

			// Track usage
			usedRecipesThisWeek[recipeID] = true
			rotationState.UsedRecipes[recipeID] = time.Now()
		}

		// Assign meals to the correct day
		switch day {
		case "monday":
			weeklyMeals.Monday = dayMeals
		case "tuesday":
			weeklyMeals.Tuesday = dayMeals
		case "wednesday":
			weeklyMeals.Wednesday = dayMeals
		case "thursday":
			weeklyMeals.Thursday = dayMeals
		case "friday":
			weeklyMeals.Friday = dayMeals
		case "saturday":
			weeklyMeals.Saturday = dayMeals
		case "sunday":
			weeklyMeals.Sunday = dayMeals
		}
	}

	// Enhanced rotation state management with cycle tracking and recipe history
	r.updateRotationStateWithHistory(rotationState, usedRecipesThisWeek, searchResult.Recipes)
	
	// Track complexity pattern for this week
	complexityPattern := r.extractComplexityPattern(weeklyMeals)
	
	// Update weekly history if this is a new week
	if r.isNewWeek(rotationState) || len(rotationState.WeeklyHistory) == 0 {
		r.updateWeeklyHistory(rotationState, usedRecipesThisWeek, complexityPattern)
	}
	
	// Update complexity history for balancing future selections
	for _, pattern := range complexityPattern {
		rotationState.ComplexityHistory = append(rotationState.ComplexityHistory, pattern)
	}
	
	// Maintain complexity history size (keep last 21 meals = 1 week)
	if len(rotationState.ComplexityHistory) > 21 {
		rotationState.ComplexityHistory = rotationState.ComplexityHistory[len(rotationState.ComplexityHistory)-21:]
	}

	// Save updated rotation state
	if err := r.UpdateRotationState(userID, rotationState); err != nil {
		// Log error but don't fail the operation
		fmt.Printf("Warning: failed to update rotation state: %v", err)
	}

	return weeklyMeals, nil
}

// selectBestRecipe chooses the best recipe based on criteria and rotation algorithm
func (r *rotationService) selectBestRecipe(recipes []models.Recipe, criteria *RecipeSelectionCriteria) (*models.Recipe, error) {
	// Filter recipes by criteria
	candidates := make([]models.Recipe, 0)

	for _, recipe := range recipes {
		// Check meal type compatibility
		if !r.containsMealType(recipe.MealType, criteria.MealType) {
			continue
		}

		// Skip if already used this week
		if criteria.UsedThisWeek[recipe.ID.String()] {
			continue
		}

		// Skip if in avoid list
		if r.contains(criteria.AvoidRecipeIDs, recipe.ID.String()) {
			continue
		}

		// Check prep time constraint
		if criteria.MaxPrepTime != nil && recipe.PrepTime > *criteria.MaxPrepTime {
			continue
		}

		// Check dietary restrictions
		if len(criteria.DietaryRestrictions) > 0 {
			if !r.matchesDietaryRestrictions(recipe.DietaryLabels, criteria.DietaryRestrictions) {
				continue
			}
		}

		// Check complexity preference (soft constraint)
		complexityScore := r.getComplexityScore(recipe.Complexity, criteria.PreferredComplexity)
		if complexityScore > 0 {
			candidates = append(candidates, recipe)
		}
	}

	if len(candidates) == 0 {
		return nil, fmt.Errorf("no suitable recipes found for %s", criteria.MealType)
	}

	// Score and sort candidates
	type scoredRecipe struct {
		recipe models.Recipe
		score  float64
	}

	scored := make([]scoredRecipe, len(candidates))
	for i, recipe := range candidates {
		score := r.calculateRecipeScore(recipe, criteria)
		scored[i] = scoredRecipe{recipe: recipe, score: score}
	}

	// Sort by score (highest first)
	sort.Slice(scored, func(i, j int) bool {
		return scored[i].score > scored[j].score
	})

	// Add some randomness to top candidates to prevent predictability
	topCount := min(5, len(scored))
	randomIndex := rand.Intn(topCount)

	return &scored[randomIndex].recipe, nil
}

// calculateRecipeScore assigns a score to a recipe based on selection criteria
func (r *rotationService) calculateRecipeScore(recipe models.Recipe, criteria *RecipeSelectionCriteria) float64 {
	score := 0.0

	// Base score from rating
	score += float64(recipe.AverageRating) * 10

	// Complexity preference bonus
	complexityScore := r.getComplexityScore(recipe.Complexity, criteria.PreferredComplexity)
	score += complexityScore * 5

	// Cuisine preference bonus
	if len(criteria.CuisinePreferences) > 0 && recipe.CuisineType != nil {
		if r.contains(criteria.CuisinePreferences, *recipe.CuisineType) {
			score += 15
		}
	}

	// Prep time bonus (shorter is better for busy days)
	if criteria.Day == "monday" || criteria.Day == "friday" {
		if recipe.PrepTime <= 30 {
			score += 10
		}
	}

	// Meal type compatibility bonus
	if r.containsMealType(recipe.MealType, criteria.MealType) {
		score += 20
	}

	return score
}

// Helper methods

func (r *rotationService) getBalancedComplexity(state *RotationState, dayIndex, mealIndex int, skillLevel string) []string {
	// Default complexity preferences based on skill level
	complexities := map[string][]string{
		"beginner":     {"simple"},
		"intermediate": {"simple", "moderate"},
		"advanced":     {"simple", "moderate", "complex"},
	}

	preferred := complexities[skillLevel]
	if len(preferred) == 0 {
		preferred = []string{"simple", "moderate"}
	}

	// Enhanced complexity balancing logic
	preferred = r.applyComplexityBalancing(state, dayIndex, mealIndex, preferred)

	return preferred
}

// applyComplexityBalancing applies advanced complexity balancing rules
func (r *rotationService) applyComplexityBalancing(state *RotationState, dayIndex, mealIndex int, preferred []string) []string {
	// Rule 1: Avoid back-to-back complex meals
	if len(state.ComplexityHistory) > 0 {
		lastComplexity := state.ComplexityHistory[len(state.ComplexityHistory)-1]
		if lastComplexity == "complex" {
			preferred = r.removeComplexity(preferred, "complex")
		}
	}

	// Rule 2: Avoid more than 2 complex meals in recent history (last 7 meals)
	recentComplexCount := r.countRecentComplexity(state.ComplexityHistory, "complex", 7)
	if recentComplexCount >= 2 {
		preferred = r.removeComplexity(preferred, "complex")
	}

	// Rule 3: Weekend complexity boost (Saturday/Sunday can handle more complex meals)
	isWeekend := dayIndex == 5 || dayIndex == 6 // Saturday (5) or Sunday (6)
	if isWeekend && r.contains(preferred, "complex") && len(preferred) > 1 {
		// Give complex meals higher preference on weekends by duplicating in array
		preferred = append(preferred, "complex")
	}

	// Rule 4: Weekday dinner complexity reduction (Monday-Thursday dinners prefer simpler)
	isWeekdayDinner := dayIndex < 4 && mealIndex == 2 // Monday-Thursday, dinner (index 2)
	if isWeekdayDinner {
		if r.contains(preferred, "complex") && len(preferred) > 1 {
			preferred = r.removeComplexity(preferred, "complex")
		}
		// Boost simple meals for weekday dinners
		if r.contains(preferred, "simple") {
			preferred = append(preferred, "simple")
		}
	}

	// Rule 5: Lunch complexity limitation (lunches generally should be simple-moderate)
	if mealIndex == 1 { // Lunch
		preferred = r.removeComplexity(preferred, "complex")
		if len(preferred) == 0 {
			preferred = []string{"simple", "moderate"}
		}
	}

	// Rule 6: Breakfast is almost always simple
	if mealIndex == 0 { // Breakfast
		preferred = []string{"simple"}
	}

	// Rule 7: Balance complexity across the week
	weekComplexityPattern := r.getWeekComplexityBalance(state, dayIndex)
	preferred = r.applyWeeklyComplexityBalance(preferred, weekComplexityPattern, dayIndex)

	// Ensure we always have at least one option
	if len(preferred) == 0 {
		preferred = []string{"simple"}
	}

	return preferred
}

// countRecentComplexity counts occurrences of a complexity level in recent history
func (r *rotationService) countRecentComplexity(history []string, complexity string, recentCount int) int {
	count := 0
	start := len(history) - recentCount
	if start < 0 {
		start = 0
	}
	
	for i := start; i < len(history); i++ {
		if history[i] == complexity {
			count++
		}
	}
	return count
}

// removeComplexity removes a complexity level from preferences
func (r *rotationService) removeComplexity(preferred []string, complexity string) []string {
	filtered := make([]string, 0)
	for _, c := range preferred {
		if c != complexity {
			filtered = append(filtered, c)
		}
	}
	return filtered
}

// getWeekComplexityBalance analyzes current week's complexity distribution
func (r *rotationService) getWeekComplexityBalance(state *RotationState, currentDayIndex int) map[string]int {
	balance := map[string]int{
		"simple":   0,
		"moderate": 0,
		"complex":  0,
	}
	
	// Count complexity levels used so far this week (estimate based on recent history)
	recentMeals := currentDayIndex * 3 // Approximate meals so far this week
	if recentMeals > len(state.ComplexityHistory) {
		recentMeals = len(state.ComplexityHistory)
	}
	
	start := len(state.ComplexityHistory) - recentMeals
	if start < 0 {
		start = 0
	}
	
	for i := start; i < len(state.ComplexityHistory); i++ {
		complexity := state.ComplexityHistory[i]
		balance[complexity]++
	}
	
	return balance
}

// applyWeeklyComplexityBalance adjusts preferences based on weekly complexity distribution
func (r *rotationService) applyWeeklyComplexityBalance(preferred []string, weekBalance map[string]int, dayIndex int) []string {
	totalMeals := weekBalance["simple"] + weekBalance["moderate"] + weekBalance["complex"]
	if totalMeals < 6 { // Not enough data to make adjustments
		return preferred
	}
	
	// Target ratios: 50% simple, 35% moderate, 15% complex for balanced approach
	simpleRatio := float64(weekBalance["simple"]) / float64(totalMeals)
	moderateRatio := float64(weekBalance["moderate"]) / float64(totalMeals)
	complexRatio := float64(weekBalance["complex"]) / float64(totalMeals)
	
	adjusted := make([]string, 0)
	
	for _, complexity := range preferred {
		switch complexity {
		case "simple":
			if simpleRatio < 0.6 { // Need more simple meals
				adjusted = append(adjusted, complexity, complexity) // Double weight
			} else {
				adjusted = append(adjusted, complexity)
			}
		case "moderate":
			if moderateRatio < 0.25 && simpleRatio > 0.4 { // Need more moderate
				adjusted = append(adjusted, complexity, complexity)
			} else {
				adjusted = append(adjusted, complexity)
			}
		case "complex":
			if complexRatio > 0.25 { // Too many complex meals
				// Skip adding complex to reduce its probability
			} else if complexRatio < 0.1 && dayIndex >= 4 { // Need complex, but only later in week
				adjusted = append(adjusted, complexity)
			}
		}
	}
	
	if len(adjusted) == 0 {
		return preferred // Fallback to original if filtering removed everything
	}
	
	return adjusted
}

func (r *rotationService) getComplexityScore(recipeComplexity string, preferredComplexities []string) float64 {
	if len(preferredComplexities) == 0 {
		return 1.0
	}

	for _, preferred := range preferredComplexities {
		if recipeComplexity == preferred {
			return 1.0
		}
	}

	return 0.3 // Partial credit for non-preferred complexity
}

func (r *rotationService) containsMealType(recipeMealTypes []string, targetMealType string) bool {
	for _, mealType := range recipeMealTypes {
		if mealType == targetMealType {
			return true
		}
	}
	return false
}

func (r *rotationService) contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}

func (r *rotationService) matchesDietaryRestrictions(recipeLabels []string, restrictions []string) bool {
	for _, restriction := range restrictions {
		found := false
		for _, label := range recipeLabels {
			if label == restriction {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}
	return true
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ResetRotationCycle manually resets the rotation cycle for a user
func (r *rotationService) ResetRotationCycle(userID uuid.UUID) error {
	state, err := r.GetRotationState(userID)
	if err != nil {
		return fmt.Errorf("failed to get rotation state: %w", err)
	}

	state.UsedRecipes = make(map[string]time.Time)
	state.CycleCount++
	now := time.Now()
	state.LastResetDate = &now

	return r.UpdateRotationState(userID, state)
}

// GetVarietyScore calculates how varied a set of recipes is (0-1 scale)
func (r *rotationService) GetVarietyScore(recipeIDs []string, userID uuid.UUID) (float64, error) {
	if len(recipeIDs) == 0 {
		return 0, nil
	}

	// Get recipe details
	recipes := make([]models.Recipe, 0)
	for _, recipeID := range recipeIDs {
		if id, err := uuid.Parse(recipeID); err == nil {
			if recipe, err := r.recipeRepo.GetByID(id, userID); err == nil {
				recipes = append(recipes, *recipe)
			}
		}
	}

	if len(recipes) == 0 {
		return 0, nil
	}

	// Calculate variety metrics
	cuisineTypes := make(map[string]int)
	complexities := make(map[string]int)
	totalTime := 0

	for _, recipe := range recipes {
		if recipe.CuisineType != nil {
			cuisineTypes[*recipe.CuisineType]++
		}
		complexities[recipe.Complexity]++
		totalTime += recipe.PrepTime + recipe.CookTime
	}

	// Calculate scores
	cuisineVariety := float64(len(cuisineTypes)) / float64(len(recipes))
	complexityVariety := float64(len(complexities)) / float64(len(recipes))
	timeVariety := 1.0 // Simplified - could analyze time distribution

	// Weighted average
	varietyScore := (cuisineVariety*0.4 + complexityVariety*0.3 + timeVariety*0.3)

	// Normalize to 0-1 scale
	if varietyScore > 1.0 {
		varietyScore = 1.0
	}

	return varietyScore, nil
}

// Helper functions for enhanced rotation management

// getCurrentISOWeek returns current ISO week string (YYYY-Www format)
func getCurrentISOWeek() string {
	year, week := time.Now().ISOWeek()
	return fmt.Sprintf("%d-W%02d", year, week)
}

// getISOWeekForTime returns ISO week string for a given time
func getISOWeekForTime(t time.Time) string {
	year, week := t.ISOWeek()
	return fmt.Sprintf("%d-W%02d", year, week)
}

// isNewWeek checks if we've moved to a new week since last rotation
func (r *rotationService) isNewWeek(state *RotationState) bool {
	currentWeek := getCurrentISOWeek()
	return state.LastUpdateWeek != currentWeek
}

// updateWeeklyHistory adds current week's data to rotation history
func (r *rotationService) updateWeeklyHistory(state *RotationState, usedRecipes map[string]bool, complexityPattern []string) error {
	currentWeek := getCurrentISOWeek()
	
	// Convert used recipes map to slice
	recipesUsed := make([]string, 0, len(usedRecipes))
	for recipeID := range usedRecipes {
		recipesUsed = append(recipesUsed, recipeID)
	}
	
	// Calculate variety score for this week
	varietyScore, err := r.GetVarietyScore(recipesUsed, uuid.Nil) // Use nil for calculation purposes
	if err != nil {
		varietyScore = 0.0 // Fallback to 0 if calculation fails
	}
	
	// Create week data
	weekData := WeekRotationData{
		Week:              currentWeek,
		RecipesUsed:       recipesUsed,
		ComplexityPattern: complexityPattern,
		VarietyScore:      varietyScore,
		GeneratedAt:       time.Now(),
	}
	
	// Add to history (maintain last 12 weeks)
	state.WeeklyHistory = append(state.WeeklyHistory, weekData)
	if len(state.WeeklyHistory) > 12 {
		state.WeeklyHistory = state.WeeklyHistory[len(state.WeeklyHistory)-12:]
	}
	
	// Update tracking fields
	state.LastUpdateWeek = currentWeek
	state.ConsecutiveWeeks++
	
	return nil
}

// getGlobalAvoidList returns recipes to avoid based on global rotation history
func (r *rotationService) getGlobalAvoidList(state *RotationState, targetVariety float64) []string {
	avoidList := make([]string, 0)
	
	// If we have recent weeks, avoid recipes from last 2-4 weeks based on variety target
	weeksToAvoid := 2
	if targetVariety > 0.8 && len(state.WeeklyHistory) >= 4 {
		weeksToAvoid = 4 // High variety users get longer avoidance
	} else if targetVariety > 0.6 && len(state.WeeklyHistory) >= 3 {
		weeksToAvoid = 3 // Medium variety users get moderate avoidance
	}
	
	// Get recent weeks to avoid
	recentWeeks := len(state.WeeklyHistory)
	if recentWeeks > weeksToAvoid {
		recentWeeks = weeksToAvoid
	}
	
	// Add recipes from recent weeks to avoid list
	for i := len(state.WeeklyHistory) - recentWeeks; i < len(state.WeeklyHistory); i++ {
		weekData := state.WeeklyHistory[i]
		avoidList = append(avoidList, weekData.RecipesUsed...)
	}
	
	// Also add recipes used in current rotation cycle
	for recipeID := range state.UsedRecipes {
		avoidList = append(avoidList, recipeID)
	}
	
	return avoidList
}

// updateRotationStateWithHistory manages rotation cycles with enhanced tracking
func (r *rotationService) updateRotationStateWithHistory(rotationState *RotationState, usedRecipes map[string]bool, availableRecipes []models.Recipe) {
	// Update total recipes used counter
	rotationState.TotalRecipesUsed += len(usedRecipes)
	
	// Update global rotation pool if empty or changed
	if len(rotationState.GlobalRotationPool) == 0 {
		rotationState.GlobalRotationPool = make([]string, len(availableRecipes))
		for i, recipe := range availableRecipes {
			rotationState.GlobalRotationPool[i] = recipe.ID.String()
		}
	}
	
	// Check if we need to reset the rotation cycle
	uniqueUsedCount := len(rotationState.UsedRecipes)
	totalAvailable := len(rotationState.GlobalRotationPool)
	
	// Reset cycle when we've used 80% of available recipes to maintain variety
	resetThreshold := int(float64(totalAvailable) * 0.8)
	if resetThreshold < 10 {
		resetThreshold = totalAvailable // For small recipe pools, use all
	}
	
	if uniqueUsedCount >= resetThreshold {
		r.resetRotationCycle(rotationState)
		
		// Re-add this week's recipes to the used list
		for recipeID := range usedRecipes {
			rotationState.UsedRecipes[recipeID] = time.Now()
		}
	} else {
		// Normal update - add new recipes to used list
		for recipeID := range usedRecipes {
			rotationState.UsedRecipes[recipeID] = time.Now()
		}
	}
}

// resetRotationCycle resets the rotation cycle with proper tracking
func (r *rotationService) resetRotationCycle(state *RotationState) {
	state.CycleCount++
	now := time.Now()
	state.LastResetDate = &now
	state.UsedRecipes = make(map[string]time.Time)
	
	// Archive current complexity history in weekly data if needed
	if len(state.ComplexityHistory) > 0 && len(state.WeeklyHistory) > 0 {
		lastWeek := &state.WeeklyHistory[len(state.WeeklyHistory)-1]
		if len(lastWeek.ComplexityPattern) == 0 {
			lastWeek.ComplexityPattern = make([]string, len(state.ComplexityHistory))
			copy(lastWeek.ComplexityPattern, state.ComplexityHistory)
		}
	}
}

// extractComplexityPattern extracts complexity pattern from weekly meals
func (r *rotationService) extractComplexityPattern(weeklyMeals *models.WeeklyMeals) []string {
	pattern := make([]string, 0, 21) // 7 days * 3 meals
	
	days := [][]models.MealSlot{
		weeklyMeals.Monday,
		weeklyMeals.Tuesday,
		weeklyMeals.Wednesday,
		weeklyMeals.Thursday,
		weeklyMeals.Friday,
		weeklyMeals.Saturday,
		weeklyMeals.Sunday,
	}
	
	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.Recipe != nil {
				pattern = append(pattern, meal.Recipe.Complexity)
			} else {
				// Fallback - estimate based on meal type
				if meal.MealType == "breakfast" {
					pattern = append(pattern, "simple")
				} else {
					pattern = append(pattern, "moderate")
				}
			}
		}
	}
	
	return pattern
}

// selectBestRecipeWithFallbacks attempts recipe selection with multiple fallback strategies
func (r *rotationService) selectBestRecipeWithFallbacks(recipes []models.Recipe, criteria *RecipeSelectionCriteria, rotationState *RotationState) (*models.Recipe, error) {
	// Primary attempt: Use original criteria
	recipe, err := r.selectBestRecipe(recipes, criteria)
	if err == nil {
		return recipe, nil
	}
	
	// Fallback 1: Relax complexity constraints
	fallback1Criteria := *criteria
	fallback1Criteria.PreferredComplexity = []string{"simple", "moderate", "complex"}
	recipe, err = r.selectBestRecipe(recipes, &fallback1Criteria)
	if err == nil {
		return recipe, nil
	}
	
	// Fallback 2: Relax prep time constraints
	fallback2Criteria := *criteria
	fallback2Criteria.MaxPrepTime = nil
	recipe, err = r.selectBestRecipe(recipes, &fallback2Criteria)
	if err == nil {
		return recipe, nil
	}
	
	// Fallback 3: Allow recently used recipes (relax rotation constraints)
	fallback3Criteria := *criteria
	fallback3Criteria.AvoidRecipeIDs = []string{} // Clear avoid list
	// Only avoid recipes used this week
	for recipeID := range fallback3Criteria.UsedThisWeek {
		fallback3Criteria.AvoidRecipeIDs = append(fallback3Criteria.AvoidRecipeIDs, recipeID)
	}
	recipe, err = r.selectBestRecipe(recipes, &fallback3Criteria)
	if err == nil {
		return recipe, nil
	}
	
	// Fallback 4: Ignore dietary restrictions (with user consent implied by constraint conflict)
	fallback4Criteria := *criteria
	fallback4Criteria.DietaryRestrictions = []string{}
	recipe, err = r.selectBestRecipe(recipes, &fallback4Criteria)
	if err == nil {
		return recipe, nil
	}
	
	// Fallback 5: Most permissive - only ensure meal type compatibility
	fallback5Criteria := RecipeSelectionCriteria{
		MealType:     criteria.MealType,
		Day:          criteria.Day,
		UsedThisWeek: make(map[string]bool), // Allow any recipe, even duplicates
	}
	recipe, err = r.selectBestRecipe(recipes, &fallback5Criteria)
	if err == nil {
		return recipe, nil
	}
	
	// Ultimate fallback: Return first recipe that matches meal type
	for _, recipe := range recipes {
		if r.containsMealType(recipe.MealType, criteria.MealType) {
			return &recipe, nil
		}
	}
	
	return nil, fmt.Errorf("no compatible recipes found for meal type %s after all fallback attempts", criteria.MealType)
}

// Enhanced rotation interface with fallback-aware selection
func (r *rotationService) SelectRecipesForWeekWithConstraintHandling(userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, *RotationConstraintReport, error) {
	weeklyMeals, err := r.SelectRecipesForWeek(userID, preferences)
	if err != nil {
		return nil, nil, err
	}
	
	// Generate constraint report
	report := r.generateConstraintReport(weeklyMeals, preferences)
	
	return weeklyMeals, report, nil
}

// RotationConstraintReport provides feedback on constraint handling
type RotationConstraintReport struct {
	TotalMeals           int                    `json:"totalMeals"`
	FallbacksUsed        int                    `json:"fallbacksUsed"`
	ConstraintViolations []ConstraintViolation  `json:"constraintViolations"`
	VarietyScore         float64                `json:"varietyScore"`
	ComplexityBalance    map[string]int         `json:"complexityBalance"`
}

// ConstraintViolation represents a constraint that couldn't be satisfied
type ConstraintViolation struct {
	MealDay          string `json:"mealDay"`
	MealType         string `json:"mealType"`
	ViolationType    string `json:"violationType"`    // "complexity", "prep_time", "dietary", "rotation"
	OriginalValue    string `json:"originalValue"`    // What was requested
	FallbackValue    string `json:"fallbackValue"`    // What was used instead
	Severity         string `json:"severity"`         // "low", "medium", "high"
}

// generateConstraintReport analyzes how well constraints were satisfied
func (r *rotationService) generateConstraintReport(weeklyMeals *models.WeeklyMeals, preferences *models.UserPreferences) *RotationConstraintReport {
	report := &RotationConstraintReport{
		TotalMeals:           21, // 7 days * 3 meals
		FallbacksUsed:        0,  // Would need tracking during selection
		ConstraintViolations: make([]ConstraintViolation, 0),
		ComplexityBalance:    make(map[string]int),
	}
	
	// Analyze meals and populate report
	allRecipeIDs := make([]string, 0, 21)
	
	days := []struct{
		name string
		meals []models.MealSlot
	}{
		{"monday", weeklyMeals.Monday},
		{"tuesday", weeklyMeals.Tuesday},
		{"wednesday", weeklyMeals.Wednesday},
		{"thursday", weeklyMeals.Thursday},
		{"friday", weeklyMeals.Friday},
		{"saturday", weeklyMeals.Saturday},
		{"sunday", weeklyMeals.Sunday},
	}
	
	for _, day := range days {
		for _, meal := range day.meals {
			if meal.Recipe != nil {
				allRecipeIDs = append(allRecipeIDs, meal.Recipe.ID.String())
				
				// Track complexity balance
				report.ComplexityBalance[meal.Recipe.Complexity]++
				
				// Check for constraint violations
				violations := r.checkMealConstraintViolations(meal, preferences, day.name)
				report.ConstraintViolations = append(report.ConstraintViolations, violations...)
			}
		}
	}
	
	// Calculate variety score
	if varietyScore, err := r.GetVarietyScore(allRecipeIDs, uuid.Nil); err == nil {
		report.VarietyScore = varietyScore
	}
	
	return report
}

// checkMealConstraintViolations checks if a meal violates user constraints
func (r *rotationService) checkMealConstraintViolations(meal models.MealSlot, preferences *models.UserPreferences, dayName string) []ConstraintViolation {
	violations := make([]ConstraintViolation, 0)
	
	if meal.Recipe == nil {
		return violations
	}
	
	// Check prep time violation
	if meal.Recipe.PrepTime > preferences.MaxPrepTimePerMeal {
		violations = append(violations, ConstraintViolation{
			MealDay:       dayName,
			MealType:      meal.MealType,
			ViolationType: "prep_time",
			OriginalValue: fmt.Sprintf("%d min", preferences.MaxPrepTimePerMeal),
			FallbackValue: fmt.Sprintf("%d min", meal.Recipe.PrepTime),
			Severity:      r.getPrepTimeSeverity(meal.Recipe.PrepTime, preferences.MaxPrepTimePerMeal),
		})
	}
	
	// Check dietary restrictions
	for _, restriction := range preferences.DietaryRestrictions {
		if !r.contains(meal.Recipe.DietaryLabels, restriction) {
			violations = append(violations, ConstraintViolation{
				MealDay:       dayName,
				MealType:      meal.MealType,
				ViolationType: "dietary",
				OriginalValue: restriction,
				FallbackValue: "not satisfied",
				Severity:      "high",
			})
		}
	}
	
	// Check complexity preference
	if preferences.PreferredMealComplexity != "" && meal.Recipe.Complexity != preferences.PreferredMealComplexity {
		violations = append(violations, ConstraintViolation{
			MealDay:       dayName,
			MealType:      meal.MealType,
			ViolationType: "complexity",
			OriginalValue: preferences.PreferredMealComplexity,
			FallbackValue: meal.Recipe.Complexity,
			Severity:      "low",
		})
	}
	
	return violations
}

// getPrepTimeSeverity determines severity of prep time constraint violation
func (r *rotationService) getPrepTimeSeverity(actualTime, maxTime int) string {
	overage := actualTime - maxTime
	if overage <= 10 {
		return "low"
	} else if overage <= 30 {
		return "medium"
	}
	return "high"
}

// SelectRecipesForWeekWithPatterns generates a weekly meal plan using user's weekly patterns
func (r *rotationService) SelectRecipesForWeekWithPatterns(userID uuid.UUID, preferences *models.UserPreferences, weeklyPatterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error) {
	// Get current rotation state
	rotationState, err := r.GetRotationState(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get rotation state: %w", err)
	}

	// Get user's available recipes
	filters := &models.RecipeFilters{
		DietaryLabels: preferences.DietaryRestrictions,
	}

	// Apply complexity preferences based on cooking skill level
	if preferences.CookingSkillLevel == "beginner" {
		filters.Complexity = []string{"simple"}
	} else if preferences.CookingSkillLevel == "advanced" {
		filters.Complexity = []string{"simple", "moderate", "complex"}
	} else {
		// Intermediate gets simple and moderate
		filters.Complexity = []string{"simple", "moderate"}
	}

	// Get available recipes
	searchParams := &models.RecipeSearchParams{
		RecipeFilters: *filters,
		Page:         1,
		Limit:        100, // Get a large pool for selection
		SortBy:       "average_rating",
		SortOrder:    "desc",
	}

	searchResult, err := r.recipeRepo.Search(userID, searchParams)
	if err != nil {
		return nil, fmt.Errorf("failed to search recipes: %w", err)
	}

	if len(searchResult.Recipes) < 21 {
		return nil, fmt.Errorf("insufficient recipes available (need at least 21, found %d)", len(searchResult.Recipes))
	}

	// Generate pattern-aware weekly meal plan
	weeklyMeals := &models.WeeklyMeals{
		Monday:    make([]models.MealSlot, 3),
		Tuesday:   make([]models.MealSlot, 3),
		Wednesday: make([]models.MealSlot, 3),
		Thursday:  make([]models.MealSlot, 3),
		Friday:    make([]models.MealSlot, 3),
		Saturday:  make([]models.MealSlot, 3),
		Sunday:    make([]models.MealSlot, 3),
	}

	usedRecipesThisWeek := make(map[string]bool)
	days := []string{"sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"}
	weekdays := []time.Weekday{time.Sunday, time.Monday, time.Tuesday, time.Wednesday, time.Thursday, time.Friday, time.Saturday}
	mealTypes := []string{"breakfast", "lunch", "dinner"}

	// Generate meals for each day using patterns
	for dayIndex, day := range days {
		dayMeals := make([]models.MealSlot, 3)
		weekday := weekdays[dayIndex]
		
		for mealIndex, mealType := range mealTypes {
			criteria := &RecipeSelectionCriteria{
				MealType:            mealType,
				Day:                 day,
				DayOfWeek:          int(weekday),
				DietaryRestrictions: preferences.DietaryRestrictions,
				CuisinePreferences:  preferences.CuisinePreferences,
				UsedThisWeek:        usedRecipesThisWeek,
			}

			// Add used recipes to avoid list
			for recipeID := range rotationState.UsedRecipes {
				criteria.AvoidRecipeIDs = append(criteria.AvoidRecipeIDs, recipeID)
			}

			// Select recipe using pattern-aware assignment
			selectedRecipe, err := r.AssignRecipeForDay(weekday, weeklyPatterns, criteria, searchResult.Recipes, rotationState)
			if err != nil {
				return nil, fmt.Errorf("failed to select recipe for %s %s: %w", day, mealType, err)
			}

			// Create meal slot
			recipeID := selectedRecipe.ID.String()
			dayMeals[mealIndex] = models.MealSlot{
				Day:      day,
				MealType: mealType,
				RecipeID: &recipeID,
				Recipe:   selectedRecipe, // Include recipe object for constraint checking
				Servings: selectedRecipe.Servings,
			}

			// Track usage
			usedRecipesThisWeek[recipeID] = true
			rotationState.UsedRecipes[recipeID] = time.Now()
		}

		// Assign meals to the correct day
		switch day {
		case "monday":
			weeklyMeals.Monday = dayMeals
		case "tuesday":
			weeklyMeals.Tuesday = dayMeals
		case "wednesday":
			weeklyMeals.Wednesday = dayMeals
		case "thursday":
			weeklyMeals.Thursday = dayMeals
		case "friday":
			weeklyMeals.Friday = dayMeals
		case "saturday":
			weeklyMeals.Saturday = dayMeals
		case "sunday":
			weeklyMeals.Sunday = dayMeals
		}
	}

	// Update rotation state with pattern-aware tracking
	r.updateRotationStateWithHistory(rotationState, usedRecipesThisWeek, searchResult.Recipes)
	
	// Track complexity pattern for this week
	complexityPattern := r.extractComplexityPattern(weeklyMeals)
	
	// Update weekly history if this is a new week
	if r.isNewWeek(rotationState) || len(rotationState.WeeklyHistory) == 0 {
		r.updateWeeklyHistory(rotationState, usedRecipesThisWeek, complexityPattern)
	}
	
	// Update complexity history for balancing future selections
	for _, pattern := range complexityPattern {
		rotationState.ComplexityHistory = append(rotationState.ComplexityHistory, pattern)
	}
	
	// Maintain complexity history size (keep last 21 meals = 1 week)
	if len(rotationState.ComplexityHistory) > 21 {
		rotationState.ComplexityHistory = rotationState.ComplexityHistory[len(rotationState.ComplexityHistory)-21:]
	}

	// Save updated rotation state
	if err := r.UpdateRotationState(userID, rotationState); err != nil {
		// Log error but don't fail the operation
		fmt.Printf("Warning: failed to update rotation state: %v", err)
	}

	return weeklyMeals, nil
}

// AssignRecipeForDay assigns a recipe for a specific day using pattern recognition
func (r *rotationService) AssignRecipeForDay(day time.Weekday, userPatterns []models.UserWeeklyPattern, criteria *RecipeSelectionCriteria, recipes []models.Recipe, rotationState *RotationState) (*models.Recipe, error) {
	// Find pattern for this day
	pattern := r.getPatternForDay(day, userPatterns)
	if pattern != nil {
		criteria.WeeklyPattern = pattern
		criteria.MaxPrepTime = &pattern.MaxPrepTime
	}

	// Apply pattern-aware complexity preferences
	preferredComplexity := r.getPatternAwareComplexity(day, pattern, rotationState)
	criteria.PreferredComplexity = preferredComplexity

	// Select recipe with pattern awareness and fallbacks
	return r.selectWithPatternWeighting(recipes, criteria, rotationState)
}

// getPatternForDay finds the weekly pattern for a specific day
func (r *rotationService) getPatternForDay(day time.Weekday, patterns []models.UserWeeklyPattern) *models.UserWeeklyPattern {
	dayInt := int(day) // 0=Sunday, 6=Saturday
	
	// First, look for exact day match
	for _, pattern := range patterns {
		if pattern.DayOfWeek == dayInt {
			return &pattern
		}
	}
	
	// If no exact match, look for weekend/weekday pattern
	isWeekend := day == time.Saturday || day == time.Sunday
	
	for _, pattern := range patterns {
		if pattern.IsWeekendPattern == isWeekend {
			return &pattern
		}
	}
	
	return nil // No pattern found, use defaults
}

// getPatternAwareComplexity determines complexity preferences based on day patterns
func (r *rotationService) getPatternAwareComplexity(day time.Weekday, pattern *models.UserWeeklyPattern, rotationState *RotationState) []string {
	// Default complexity based on day type
	isWeekend := day == time.Saturday || day == time.Sunday
	defaultComplexity := []string{"simple", "moderate"}
	
	if isWeekend {
		defaultComplexity = []string{"simple", "moderate", "complex"} // Weekends allow complex meals
	}
	
	// Apply user's pattern preferences if available
	if pattern != nil {
		switch pattern.PreferredComplexity {
		case "simple":
			return []string{"simple"}
		case "moderate":
			return []string{"simple", "moderate"}
		case "complex":
			if isWeekend {
				return []string{"moderate", "complex"} // Complex preference on weekends
			} else {
				return []string{"simple", "moderate"} // Limit complex on weekdays
			}
		}
	}
	
	// Apply existing complexity balancing logic
	dayIndex := int(day)
	if day == time.Sunday {
		dayIndex = 0
	} else {
		dayIndex = int(day)
	}
	
	mealIndex := 1 // Default to lunch for complexity calculation
	return r.applyComplexityBalancing(rotationState, dayIndex, mealIndex, defaultComplexity)
}

// selectWithPatternWeighting selects recipes with pattern-aware weighting
func (r *rotationService) selectWithPatternWeighting(recipes []models.Recipe, criteria *RecipeSelectionCriteria, rotationState *RotationState) (*models.Recipe, error) {
	// Filter recipes by basic criteria first
	candidates := make([]models.Recipe, 0)

	for _, recipe := range recipes {
		// Check meal type compatibility
		if !r.containsMealType(recipe.MealType, criteria.MealType) {
			continue
		}

		// Skip if already used this week
		if criteria.UsedThisWeek[recipe.ID.String()] {
			continue
		}

		// Skip if in avoid list
		if r.contains(criteria.AvoidRecipeIDs, recipe.ID.String()) {
			continue
		}

		// Check prep time constraint (pattern-aware)
		if criteria.MaxPrepTime != nil && recipe.PrepTime > *criteria.MaxPrepTime {
			continue
		}

		// Check dietary restrictions
		if len(criteria.DietaryRestrictions) > 0 {
			if !r.matchesDietaryRestrictions(recipe.DietaryLabels, criteria.DietaryRestrictions) {
				continue
			}
		}

		// Check complexity preference (soft constraint)
		complexityScore := r.getComplexityScore(recipe.Complexity, criteria.PreferredComplexity)
		if complexityScore > 0 {
			candidates = append(candidates, recipe)
		}
	}

	if len(candidates) == 0 {
		// Use fallback selection if pattern-aware selection fails
		return r.selectBestRecipeWithFallbacks(recipes, criteria, rotationState)
	}

	// Score and sort candidates with pattern weighting
	type scoredRecipe struct {
		recipe models.Recipe
		score  float64
	}

	scored := make([]scoredRecipe, len(candidates))
	for i, recipe := range candidates {
		score := r.calculatePatternAwareScore(recipe, criteria)
		scored[i] = scoredRecipe{recipe: recipe, score: score}
	}

	// Sort by score (highest first)
	sort.Slice(scored, func(i, j int) bool {
		return scored[i].score > scored[j].score
	})

	// Add some randomness to top candidates to prevent predictability
	topCount := min(5, len(scored))
	randomIndex := rand.Intn(topCount)

	return &scored[randomIndex].recipe, nil
}

// calculatePatternAwareScore calculates recipe score with pattern awareness
func (r *rotationService) calculatePatternAwareScore(recipe models.Recipe, criteria *RecipeSelectionCriteria) float64 {
	score := 0.0

	// Base score from rating
	score += float64(recipe.AverageRating) * 10

	// Pattern-aware complexity bonus
	if criteria.WeeklyPattern != nil {
		if recipe.Complexity == criteria.WeeklyPattern.PreferredComplexity {
			score += 20 // Strong bonus for matching pattern complexity
		}
	}

	// Standard complexity preference bonus
	complexityScore := r.getComplexityScore(recipe.Complexity, criteria.PreferredComplexity)
	score += complexityScore * 5

	// Cuisine preference bonus
	if len(criteria.CuisinePreferences) > 0 && recipe.CuisineType != nil {
		if r.contains(criteria.CuisinePreferences, *recipe.CuisineType) {
			score += 15
		}
	}

	// Pattern-aware prep time bonus
	if criteria.WeeklyPattern != nil {
		prepTimeRatio := float64(recipe.PrepTime) / float64(criteria.WeeklyPattern.MaxPrepTime)
		if prepTimeRatio <= 0.8 { // Recipe is well within time limits
			score += 10
		}
	}

	// Weekend vs weekday bonus
	isWeekend := criteria.DayOfWeek == 0 || criteria.DayOfWeek == 6 // Sunday or Saturday
	if isWeekend && recipe.Complexity == "complex" {
		score += 15 // Bonus for complex meals on weekends
	}

	// Weekday efficiency bonus
	if !isWeekend && recipe.PrepTime <= 30 && criteria.MealType != "breakfast" {
		score += 10 // Bonus for quick weekday meals
	}

	// Meal type compatibility bonus
	if r.containsMealType(recipe.MealType, criteria.MealType) {
		score += 20
	}

	return score
}// selectWithFavoritesWeighting applies favorites weighting to recipe selection
func (r *rotationService) selectWithFavoritesWeighting(recipes []models.Recipe, criteria *RecipeSelectionCriteria, rotationState *RotationState, userID uuid.UUID) (*models.Recipe, error) {
	// Get user's favorite recipe IDs
	favoriteIDs, err := r.preferenceRepo.GetFavoriteRecipeIDs(userID)
	if err != nil {
		// Log warning but continue without favorites weighting
		fmt.Printf("Warning: failed to get user favorites: %v", err)
		return r.selectWithPatternWeighting(recipes, criteria, rotationState)
	}
	
	// Create a map for quick lookup
	favoriteMap := make(map[string]float64)
	if len(favoriteIDs) > 0 {
		// Get detailed favorite data with multipliers
		favorites, _, err := r.preferenceRepo.GetUserFavorites(userID, 1, 1000) // Get all favorites
		if err != nil {
			fmt.Printf("Warning: failed to get favorite details: %v", err)
		} else {
			for _, fav := range favorites {
				favoriteMap[fav.RecipeID.String()] = fav.WeightMultiplier
			}
		}
	}
	
	// Filter recipes by basic criteria first
	candidates := make([]models.Recipe, 0)

	for _, recipe := range recipes {
		// Check meal type compatibility
		if !r.containsMealType(recipe.MealType, criteria.MealType) {
			continue
		}

		// Skip if already used this week
		if criteria.UsedThisWeek[recipe.ID.String()] {
			continue
		}

		// Skip if in avoid list
		if r.contains(criteria.AvoidRecipeIDs, recipe.ID.String()) {
			continue
		}

		// Check prep time constraint (pattern-aware)
		if criteria.MaxPrepTime != nil && recipe.PrepTime > *criteria.MaxPrepTime {
			continue
		}

		// Check dietary restrictions
		if len(criteria.DietaryRestrictions) > 0 {
			if !r.matchesDietaryRestrictions(recipe.DietaryLabels, criteria.DietaryRestrictions) {
				continue
			}
		}

		// Check complexity preference (soft constraint)
		complexityScore := r.getComplexityScore(recipe.Complexity, criteria.PreferredComplexity)
		if complexityScore > 0 {
			candidates = append(candidates, recipe)
		}
	}

	if len(candidates) == 0 {
		// Use fallback selection if pattern-aware selection fails
		return r.selectBestRecipeWithFallbacks(recipes, criteria, rotationState)
	}

	// Score and sort candidates with favorites weighting
	type scoredRecipe struct {
		recipe models.Recipe
		score  float64
	}

	scored := make([]scoredRecipe, len(candidates))
	for i, recipe := range candidates {
		score := r.calculateFavoritesWeightedScore(recipe, criteria, favoriteMap)
		scored[i] = scoredRecipe{recipe: recipe, score: score}
	}

	// Sort by score (highest first)
	sort.Slice(scored, func(i, j int) bool {
		return scored[i].score > scored[j].score
	})

	// Apply favorites probability boost - give favorites higher chance of selection
	if len(scored) > 0 {
		topCount := min(10, len(scored))
		
		// Create weighted selection pool
		weightedCandidates := make([]models.Recipe, 0)
		for i := 0; i < topCount; i++ {
			recipe := scored[i].recipe
			recipeID := recipe.ID.String()
			
			// Add multiple copies based on favorite status and score ranking
			copies := 1
			if multiplier, isFavorite := favoriteMap[recipeID]; isFavorite {
				// Favorites get more copies based on their multiplier
				copies = int(multiplier * 2) // 1.5x -> 3 copies, 2.0x -> 4 copies
			}
			
			// Higher ranked recipes get more copies
			if i < 3 {
				copies += 2 // Top 3 get bonus copies
			} else if i < 5 {
				copies += 1 // Top 5 get some bonus
			}
			
			for j := 0; j < copies; j++ {
				weightedCandidates = append(weightedCandidates, recipe)
			}
		}
		
		// Random selection from weighted pool
		if len(weightedCandidates) > 0 {
			randomIndex := rand.Intn(len(weightedCandidates))
			return &weightedCandidates[randomIndex], nil
		}
	}

	// Fallback to basic selection
	topCount := min(5, len(scored))
	randomIndex := rand.Intn(topCount)
	return &scored[randomIndex].recipe, nil
}

// calculateFavoritesWeightedScore calculates recipe score with favorites weighting
func (r *rotationService) calculateFavoritesWeightedScore(recipe models.Recipe, criteria *RecipeSelectionCriteria, favoriteMap map[string]float64) float64 {
	// Start with pattern-aware score
	score := r.calculatePatternAwareScore(recipe, criteria)
	
	// Apply favorites multiplier
	recipeID := recipe.ID.String()
	if multiplier, isFavorite := favoriteMap[recipeID]; isFavorite {
		score *= multiplier // Apply the user's favorite multiplier (default 1.5x)
		
		// Additional favorite bonus to ensure strong preference
		score += 25 // Significant bonus for being a favorite
		
		// Scale bonus based on multiplier strength
		if multiplier >= 2.0 {
			score += 15 // Extra bonus for high-multiplier favorites
		}
	}
	
	return score
}

// ResetRotationCycleWithOptions provides enhanced reset functionality with options
func (r *rotationService) ResetRotationCycleWithOptions(userID uuid.UUID, req *models.RotationResetRequest) error {
	if !req.ConfirmReset {
		return fmt.Errorf("reset confirmation required")
	}

	state, err := r.GetRotationState(userID)
	if err != nil {
		return fmt.Errorf("failed to get rotation state: %w", err)
	}

	// Count weeks being cleared
	weeksCleared := len(state.WeeklyHistory)
	
	// Clear rotation history but preserve preferences based on request
	state.UsedRecipes = make(map[string]time.Time)
	state.CycleCount++
	state.MealTypeHistory = make(map[string][]string)
	state.ComplexityHistory = []string{}
	state.WeeklyHistory = []WeekRotationData{}
	state.GlobalRotationPool = []string{}
	state.LastUpdateWeek = ""
	state.ConsecutiveWeeks = 0
	
	now := time.Now()
	state.LastResetDate = &now

	// Update rotation state
	if err := r.UpdateRotationState(userID, state); err != nil {
		return fmt.Errorf("failed to update rotation state: %w", err)
	}

	// Log reset event for analytics
	resetLog := models.RotationResetLog{
		UserID:             userID,
		ResetAt:            now,
		PreservedPatterns:  req.PreservePatterns,
		PreservedFavorites: req.PreserveFavorites,
		WeeksCleared:       weeksCleared,
	}

	// Store reset log in database via preference repository
	if err := r.preferenceRepo.CreateResetLog(&resetLog); err != nil {
		// Log error but don't fail the reset - this is analytics data
		fmt.Printf("Warning: failed to log reset event: %v\n", err)
	}

	return nil
}

// GetRotationAnalytics provides comprehensive rotation analytics
func (r *rotationService) GetRotationAnalytics(userID uuid.UUID, weeks int) (*models.RotationAnalytics, error) {
	state, err := r.GetRotationState(userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get rotation state: %w", err)
	}

	now := time.Now()
	analytics := &models.RotationAnalytics{
		UserID:       userID,
		CalculatedAt: now,
		WeeksAnalyzed: weeks,
	}

	// Limit analysis to requested weeks or available history
	historyToAnalyze := state.WeeklyHistory
	if len(historyToAnalyze) > weeks {
		historyToAnalyze = historyToAnalyze[len(historyToAnalyze)-weeks:]
	}
	analytics.WeeksAnalyzed = len(historyToAnalyze)

	// Calculate variety score (0-100 scale)
	if len(historyToAnalyze) > 0 {
		totalVariety := 0.0
		for _, week := range historyToAnalyze {
			totalVariety += week.VarietyScore * 100 // Convert 0-1 to 0-100 scale
		}
		analytics.VarietyScore = totalVariety / float64(len(historyToAnalyze))
	}

	// Calculate complexity distribution
	complexityCount := map[string]int{
		"simple":   0,
		"moderate": 0,
		"complex":  0,
	}
	totalMeals := 0

	for _, week := range historyToAnalyze {
		for _, complexity := range week.ComplexityPattern {
			if count, exists := complexityCount[complexity]; exists {
				complexityCount[complexity] = count + 1
			}
			totalMeals++
		}
	}

	analytics.ComplexityDistribution = make(map[string]float64)
	if totalMeals > 0 {
		for complexity, count := range complexityCount {
			analytics.ComplexityDistribution[complexity] = float64(count) / float64(totalMeals) * 100
		}
	}

	// Generate complexity trends by week
	analytics.ComplexityTrends = make([]models.ComplexityTrendData, 0, len(historyToAnalyze))
	for _, week := range historyToAnalyze {
		weekComplexity := map[string]float64{
			"simple":   0,
			"moderate": 0,
			"complex":  0,
		}
		
		for _, complexity := range week.ComplexityPattern {
			if _, exists := weekComplexity[complexity]; exists {
				weekComplexity[complexity]++
			}
		}
		
		// Convert to percentages
		total := float64(len(week.ComplexityPattern))
		if total > 0 {
			for complexity := range weekComplexity {
				weekComplexity[complexity] = weekComplexity[complexity] / total * 100
			}
		}

		analytics.ComplexityTrends = append(analytics.ComplexityTrends, models.ComplexityTrendData{
			Week:       week.Week,
			Complexity: weekComplexity,
		})
	}

	// Get favorites frequency (requires preference repository)
	favorites, _, err := r.preferenceRepo.GetUserFavorites(userID, 1, 100)
	if err == nil {
		analytics.FavoritesFrequency = make(map[string]int)
		for _, favorite := range favorites {
			// Count usage in recent weeks - simplified for now
			analytics.FavoritesFrequency[favorite.RecipeID.String()] = 0
			for _, week := range historyToAnalyze {
				for _, recipeID := range week.RecipesUsed {
					if recipeID == favorite.RecipeID.String() {
						analytics.FavoritesFrequency[favorite.RecipeID.String()]++
					}
				}
			}
		}
	}

	// Generate weekly pattern analysis
	analytics.WeeklyPatterns = make([]models.WeeklyAnalysisData, 0, len(historyToAnalyze))
	for _, week := range historyToAnalyze {
		weekAnalysis := models.WeeklyAnalysisData{
			Week:            week.Week,
			PatternAdherence: 85.0, // Placeholder - would need pattern matching logic
			VarietyScore:    week.VarietyScore * 100,
			FavoritesRatio:  0.0, // Placeholder - would need favorites calculation
		}
		analytics.WeeklyPatterns = append(analytics.WeeklyPatterns, weekAnalysis)
	}

	// Calculate rotation efficiency (algorithm performance metric)
	analytics.RotationEfficiency = 85.0 // Placeholder - would need performance tracking

	// Calculate favorites impact on variety
	analytics.FavoritesImpact = 15.0 // Placeholder - would need statistical analysis

	return analytics, nil
}

// ExportRotationData exports rotation data in specified format
func (r *rotationService) ExportRotationData(userID uuid.UUID, format string, startDate, endDate time.Time) ([]byte, error) {
	analytics, err := r.GetRotationAnalytics(userID, 52) // Get up to 1 year
	if err != nil {
		return nil, fmt.Errorf("failed to get analytics: %w", err)
	}

	switch format {
	case "json":
		return json.MarshalIndent(analytics, "", "  ")
	case "csv":
		return r.exportToCSV(analytics)
	default:
		return nil, fmt.Errorf("unsupported format: %s", format)
	}
}

// exportToCSV converts analytics to CSV format
func (r *rotationService) exportToCSV(analytics *models.RotationAnalytics) ([]byte, error) {
	var csv []string
	csv = append(csv, "Week,VarietyScore,PatternAdherence,FavoritesRatio")
	
	for _, week := range analytics.WeeklyPatterns {
		line := fmt.Sprintf("%s,%.2f,%.2f,%.2f",
			week.Week,
			week.VarietyScore,
			week.PatternAdherence,
			week.FavoritesRatio,
		)
		csv = append(csv, line)
	}
	
	csvContent := ""
	for i, line := range csv {
		csvContent += line
		if i < len(csv)-1 {
			csvContent += "\n"
		}
	}
	
	return []byte(csvContent), nil
}