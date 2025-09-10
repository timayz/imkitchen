package repositories

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
)

type MealPlanRepository interface {
	Create(mealPlan *models.MealPlan) error
	GetByID(id uuid.UUID, userID uuid.UUID) (*models.MealPlan, error)
	GetByUserID(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlan, error)
	GetByWeekStart(userID uuid.UUID, weekStart time.Time) (*models.MealPlan, error)
	Update(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlan, error)
	Delete(id uuid.UUID, userID uuid.UUID) error
	UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlan, error)
	DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlan, error)
}

type mealPlanRepository struct {
	db *gorm.DB
}

func NewMealPlanRepository(db *gorm.DB) MealPlanRepository {
	return &mealPlanRepository{db: db}
}

func (r *mealPlanRepository) Create(mealPlan *models.MealPlan) error {
	// Convert WeeklyMeals to JSONB format for database storage
	if err := r.convertMealsToJSONB(mealPlan); err != nil {
		return fmt.Errorf("failed to convert meals to JSONB: %w", err)
	}

	return r.db.Create(mealPlan).Error
}

func (r *mealPlanRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.MealPlan, error) {
	var mealPlan models.MealPlan
	err := r.db.Where("id = ? AND user_id = ? AND status != 'deleted'", id, userID).First(&mealPlan).Error
	if err != nil {
		return nil, err
	}

	// Convert JSONB meals back to structured format
	if err := r.convertJSONBToMeals(&mealPlan); err != nil {
		return nil, fmt.Errorf("failed to convert JSONB to meals: %w", err)
	}

	return &mealPlan, nil
}

func (r *mealPlanRepository) GetByUserID(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlan, error) {
	var mealPlans []models.MealPlan

	query := r.db.Where("user_id = ? AND status != 'deleted'", userID)

	if filters != nil {
		if filters.WeekStart != nil {
			query = query.Where("week_start >= ?", *filters.WeekStart)
		}
		if filters.WeekEnd != nil {
			query = query.Where("week_start <= ?", *filters.WeekEnd)
		}
		if filters.Status != nil {
			query = query.Where("status = ?", *filters.Status)
		}
	}

	err := query.Order("week_start DESC").Find(&mealPlans).Error
	if err != nil {
		return nil, err
	}

	// Convert JSONB meals back to structured format for each meal plan
	for i := range mealPlans {
		if err := r.convertJSONBToMeals(&mealPlans[i]); err != nil {
			return nil, fmt.Errorf("failed to convert JSONB to meals for meal plan %s: %w", mealPlans[i].ID, err)
		}
	}

	return mealPlans, nil
}

func (r *mealPlanRepository) GetByWeekStart(userID uuid.UUID, weekStart time.Time) (*models.MealPlan, error) {
	var mealPlan models.MealPlan
	err := r.db.Where("user_id = ? AND week_start = ? AND status != 'deleted'", userID, weekStart).First(&mealPlan).Error
	if err != nil {
		return nil, err
	}

	// Convert JSONB meals back to structured format
	if err := r.convertJSONBToMeals(&mealPlan); err != nil {
		return nil, fmt.Errorf("failed to convert JSONB to meals: %w", err)
	}

	return &mealPlan, nil
}

func (r *mealPlanRepository) Update(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlan, error) {
	var mealPlan models.MealPlan

	// First get the existing meal plan
	err := r.db.Where("id = ? AND user_id = ? AND status != 'deleted'", id, userID).First(&mealPlan).Error
	if err != nil {
		return nil, err
	}

	// Update fields
	updateData := make(map[string]interface{})
	updateData["updated_at"] = time.Now()

	if input.Status != nil {
		updateData["status"] = *input.Status
	}

	if input.CompletionPercentage != nil {
		updateData["completion_percentage"] = *input.CompletionPercentage
	}

	if input.UserFeedback != nil {
		updateData["user_feedback"] = *input.UserFeedback
	}

	if input.Meals != nil {
		// Convert meals to JSONB format
		mealsJSON, err := json.Marshal(input.Meals)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal meals: %w", err)
		}
		updateData["meals"] = mealsJSON
	}

	// Update the meal plan
	err = r.db.Model(&mealPlan).Where("id = ? AND user_id = ?", id, userID).Updates(updateData).Error
	if err != nil {
		return nil, err
	}

	// Get the updated meal plan
	return r.GetByID(id, userID)
}

func (r *mealPlanRepository) Delete(id uuid.UUID, userID uuid.UUID) error {
	return r.db.Model(&models.MealPlan{}).
		Where("id = ? AND user_id = ?", id, userID).
		Updates(map[string]interface{}{
			"status":      "deleted",
			"updated_at":  time.Now(),
			"archived_at": time.Now(),
		}).Error
}

func (r *mealPlanRepository) UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlan, error) {
	// Get the existing meal plan
	mealPlan, err := r.GetByID(mealPlanID, userID)
	if err != nil {
		return nil, err
	}

	// Convert JSONB to structured format to modify
	var weeklyMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &weeklyMeals); err != nil {
		return nil, fmt.Errorf("failed to unmarshal meals: %w", err)
	}

	// Get the day's meals
	dayMeals, err := r.getDayMeals(&weeklyMeals, day)
	if err != nil {
		return nil, err
	}

	// Find and update the specific meal slot
	found := false
	for i := range *dayMeals {
		if (*dayMeals)[i].MealType == mealType {
			if input.RecipeID != nil {
				(*dayMeals)[i].RecipeID = input.RecipeID
			}
			if input.Servings != nil {
				(*dayMeals)[i].Servings = *input.Servings
			}
			if input.Notes != nil {
				(*dayMeals)[i].Notes = input.Notes
			}
			if input.IsCompleted != nil {
				(*dayMeals)[i].IsCompleted = *input.IsCompleted
			}
			found = true
			break
		}
	}

	if !found {
		return nil, fmt.Errorf("meal slot not found for day %s, meal type %s", day, mealType)
	}

	// Convert back to JSONB and update
	mealsJSON, err := json.Marshal(weeklyMeals)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal updated meals: %w", err)
	}

	updateData := map[string]interface{}{
		"meals":      mealsJSON,
		"updated_at": time.Now(),
	}

	err = r.db.Model(&models.MealPlan{}).
		Where("id = ? AND user_id = ?", mealPlanID, userID).
		Updates(updateData).Error
	if err != nil {
		return nil, err
	}

	return r.GetByID(mealPlanID, userID)
}

func (r *mealPlanRepository) DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlan, error) {
	// Get the existing meal plan
	mealPlan, err := r.GetByID(mealPlanID, userID)
	if err != nil {
		return nil, err
	}

	// Convert JSONB to structured format to modify
	var weeklyMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &weeklyMeals); err != nil {
		return nil, fmt.Errorf("failed to unmarshal meals: %w", err)
	}

	// Get the day's meals
	dayMeals, err := r.getDayMeals(&weeklyMeals, day)
	if err != nil {
		return nil, err
	}

	// Find and remove the specific meal slot
	for i, meal := range *dayMeals {
		if meal.MealType == mealType {
			// Remove the meal slot by setting recipe to nil
			(*dayMeals)[i].RecipeID = nil
			(*dayMeals)[i].Recipe = nil
			(*dayMeals)[i].Notes = nil
			(*dayMeals)[i].IsCompleted = false
			break
		}
	}

	// Convert back to JSONB and update
	mealsJSON, err := json.Marshal(weeklyMeals)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal updated meals: %w", err)
	}

	updateData := map[string]interface{}{
		"meals":      mealsJSON,
		"updated_at": time.Now(),
	}

	err = r.db.Model(&models.MealPlan{}).
		Where("id = ? AND user_id = ?", mealPlanID, userID).
		Updates(updateData).Error
	if err != nil {
		return nil, err
	}

	return r.GetByID(mealPlanID, userID)
}

// Helper method to convert WeeklyMeals to JSONB format
func (r *mealPlanRepository) convertMealsToJSONB(mealPlan *models.MealPlan) error {
	if mealPlan.Entries == nil || len(mealPlan.Entries) == 0 {
		// If no entries provided, create empty weekly structure
		emptyWeek := models.WeeklyMeals{
			Monday:    []models.MealSlot{},
			Tuesday:   []models.MealSlot{},
			Wednesday: []models.MealSlot{},
			Thursday:  []models.MealSlot{},
			Friday:    []models.MealSlot{},
			Saturday:  []models.MealSlot{},
			Sunday:    []models.MealSlot{},
		}
		mealsJSON, err := json.Marshal(emptyWeek)
		if err != nil {
			return err
		}
		mealPlan.Meals = mealsJSON
		return nil
	}

	// Convert entries to weekly meals structure if provided
	// This would be implemented based on specific business logic
	return nil
}

// Helper method to convert JSONB back to structured meals format
func (r *mealPlanRepository) convertJSONBToMeals(mealPlan *models.MealPlan) error {
	if len(mealPlan.Meals) == 0 {
		return nil
	}

	var weeklyMeals models.WeeklyMeals
	if err := json.Unmarshal(mealPlan.Meals, &weeklyMeals); err != nil {
		return err
	}

	// For now, we keep the JSONB format as the primary storage
	// The Entries field can be populated if needed for specific use cases
	return nil
}

// Helper method to get meals for a specific day
func (r *mealPlanRepository) getDayMeals(weeklyMeals *models.WeeklyMeals, day string) (*[]models.MealSlot, error) {
	switch day {
	case "monday":
		return &weeklyMeals.Monday, nil
	case "tuesday":
		return &weeklyMeals.Tuesday, nil
	case "wednesday":
		return &weeklyMeals.Wednesday, nil
	case "thursday":
		return &weeklyMeals.Thursday, nil
	case "friday":
		return &weeklyMeals.Friday, nil
	case "saturday":
		return &weeklyMeals.Saturday, nil
	case "sunday":
		return &weeklyMeals.Sunday, nil
	default:
		return nil, fmt.Errorf("invalid day: %s", day)
	}
}
