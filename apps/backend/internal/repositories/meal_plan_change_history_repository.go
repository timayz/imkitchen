package repositories

import (
	"fmt"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/models"
)

type MealPlanChangeHistoryRepository interface {
	Create(change *models.MealPlanChangeHistory) error
	GetByMealPlanID(mealPlanID uuid.UUID, userID uuid.UUID, filters *models.ChangeHistoryFilters) ([]models.MealPlanChangeHistory, error)
	GetLatestChanges(mealPlanID uuid.UUID, userID uuid.UUID, limit int) ([]models.MealPlanChangeHistory, error)
	DeleteOldChanges(mealPlanID uuid.UUID, userID uuid.UUID, keepCount int) error
	CountChanges(mealPlanID uuid.UUID, userID uuid.UUID) (int64, error)
}

type mealPlanChangeHistoryRepository struct {
	db *gorm.DB
}

func NewMealPlanChangeHistoryRepository(db *gorm.DB) MealPlanChangeHistoryRepository {
	return &mealPlanChangeHistoryRepository{db: db}
}

func (r *mealPlanChangeHistoryRepository) Create(change *models.MealPlanChangeHistory) error {
	if change.ID == uuid.Nil {
		change.ID = uuid.New()
	}
	if change.CreatedAt.IsZero() {
		change.CreatedAt = time.Now()
	}

	return r.db.Create(change).Error
}

func (r *mealPlanChangeHistoryRepository) GetByMealPlanID(mealPlanID uuid.UUID, userID uuid.UUID, filters *models.ChangeHistoryFilters) ([]models.MealPlanChangeHistory, error) {
	var changes []models.MealPlanChangeHistory

	query := r.db.Where("meal_plan_id = ? AND user_id = ?", mealPlanID, userID)

	// Apply filters
	if filters != nil {
		if filters.ChangeType != nil {
			query = query.Where("change_type = ?", *filters.ChangeType)
		}
		if filters.StartDate != nil {
			query = query.Where("created_at >= ?", *filters.StartDate)
		}
		if filters.EndDate != nil {
			query = query.Where("created_at <= ?", *filters.EndDate)
		}
	}

	// Apply limit
	limit := 20 // default limit
	if filters != nil && filters.Limit != nil {
		limit = *filters.Limit
	}

	err := query.Order("created_at DESC").Limit(limit).Find(&changes).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get change history: %w", err)
	}

	return changes, nil
}

func (r *mealPlanChangeHistoryRepository) GetLatestChanges(mealPlanID uuid.UUID, userID uuid.UUID, limit int) ([]models.MealPlanChangeHistory, error) {
	var changes []models.MealPlanChangeHistory

	err := r.db.Where("meal_plan_id = ? AND user_id = ?", mealPlanID, userID).
		Order("created_at DESC").
		Limit(limit).
		Find(&changes).Error

	if err != nil {
		return nil, fmt.Errorf("failed to get latest changes: %w", err)
	}

	return changes, nil
}

func (r *mealPlanChangeHistoryRepository) DeleteOldChanges(mealPlanID uuid.UUID, userID uuid.UUID, keepCount int) error {
	// Get the timestamp of the Nth most recent change
	var cutoffTime time.Time
	subquery := r.db.Model(&models.MealPlanChangeHistory{}).
		Select("created_at").
		Where("meal_plan_id = ? AND user_id = ?", mealPlanID, userID).
		Order("created_at DESC").
		Limit(1).
		Offset(keepCount)

	err := subquery.Scan(&cutoffTime).Error
	if err != nil {
		// If no cutoff time found (less than keepCount records), don't delete anything
		return nil
	}

	// Delete changes older than the cutoff time
	err = r.db.Where("meal_plan_id = ? AND user_id = ? AND created_at < ?", 
		mealPlanID, userID, cutoffTime).
		Delete(&models.MealPlanChangeHistory{}).Error

	if err != nil {
		return fmt.Errorf("failed to delete old changes: %w", err)
	}

	return nil
}

func (r *mealPlanChangeHistoryRepository) CountChanges(mealPlanID uuid.UUID, userID uuid.UUID) (int64, error) {
	var count int64
	err := r.db.Model(&models.MealPlanChangeHistory{}).
		Where("meal_plan_id = ? AND user_id = ?", mealPlanID, userID).
		Count(&count).Error

	if err != nil {
		return 0, fmt.Errorf("failed to count changes: %w", err)
	}

	return count, nil
}