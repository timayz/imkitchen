package repositories

import (
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/models"
)

// ShoppingListRepository interface defines shopping list data access methods
type ShoppingListRepository interface {
	Create(list *models.ShoppingList) error
	CreateItems(items []models.ShoppingItem) error
	GetByID(id uuid.UUID, userID uuid.UUID) (*models.ShoppingList, error)
	GetByUserID(userID uuid.UUID, status string, sortBy string) ([]models.ShoppingList, error)
	GetItemsByListID(listID uuid.UUID) ([]models.ShoppingItem, error)
	GetWithItems(id uuid.UUID, userID uuid.UUID) (*models.ShoppingList, []models.ShoppingItem, error)
	UpdateItem(itemID uuid.UUID, updates *models.ShoppingItemUpdateRequest) error
	UpdateStatus(listID uuid.UUID, userID uuid.UUID, status string) error
	Delete(listID uuid.UUID, userID uuid.UUID) error
	GetByMealPlanID(mealPlanID uuid.UUID) ([]models.ShoppingList, error)
	GetActiveByUserID(userID uuid.UUID) ([]models.ShoppingList, error)
}

// shoppingListRepository implements ShoppingListRepository using GORM
type shoppingListRepository struct {
	db *gorm.DB
}

// NewShoppingListRepository creates a new shopping list repository
func NewShoppingListRepository(db *gorm.DB) ShoppingListRepository {
	return &shoppingListRepository{
		db: db,
	}
}

// Create creates a new shopping list
func (r *shoppingListRepository) Create(list *models.ShoppingList) error {
	return r.db.Create(list).Error
}

// CreateItems creates shopping list items in batch
func (r *shoppingListRepository) CreateItems(items []models.ShoppingItem) error {
	if len(items) == 0 {
		return nil
	}

	return r.db.Transaction(func(tx *gorm.DB) error {
		for _, item := range items {
			if err := tx.Create(&item).Error; err != nil {
				return err
			}
		}
		return nil
	})
}

// GetByID retrieves a shopping list by ID for a specific user
func (r *shoppingListRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.ShoppingList, error) {
	var list models.ShoppingList
	err := r.db.Where("id = ? AND user_id = ?", id, userID).First(&list).Error
	if err != nil {
		return nil, err
	}
	return &list, nil
}

// GetByUserID retrieves all shopping lists for a user with optional filtering
func (r *shoppingListRepository) GetByUserID(userID uuid.UUID, status string, sortBy string) ([]models.ShoppingList, error) {
	query := r.db.Where("user_id = ?", userID)

	if status != "" {
		query = query.Where("status = ?", status)
	}

	// Add sorting
	switch sortBy {
	case "name":
		query = query.Order("name ASC")
	case "updated":
		query = query.Order("updated_at DESC")
	case "created":
		fallthrough
	default:
		query = query.Order("created_at DESC")
	}

	var lists []models.ShoppingList
	err := query.Find(&lists).Error
	if err != nil {
		return nil, err
	}

	return lists, nil
}

// GetItemsByListID retrieves all items for a shopping list
func (r *shoppingListRepository) GetItemsByListID(listID uuid.UUID) ([]models.ShoppingItem, error) {
	var items []models.ShoppingItem
	err := r.db.Where("shopping_list_id = ?", listID).
		Order("category, ingredient_name").
		Find(&items).Error
	if err != nil {
		return nil, err
	}

	return items, nil
}

// GetWithItems retrieves a shopping list with all its items
func (r *shoppingListRepository) GetWithItems(id uuid.UUID, userID uuid.UUID) (*models.ShoppingList, []models.ShoppingItem, error) {
	list, err := r.GetByID(id, userID)
	if err != nil {
		return nil, nil, err
	}

	items, err := r.GetItemsByListID(id)
	if err != nil {
		return nil, nil, err
	}

	return list, items, nil
}

// UpdateItem updates a shopping item
func (r *shoppingListRepository) UpdateItem(itemID uuid.UUID, updates *models.ShoppingItemUpdateRequest) error {
	updateData := map[string]interface{}{
		"is_completed": updates.IsCompleted,
		"notes":        updates.Notes,
		"updated_at":   time.Now(),
	}

	if updates.IsCompleted {
		updateData["completed_at"] = time.Now()
	} else {
		updateData["completed_at"] = nil
	}

	return r.db.Model(&models.ShoppingItem{}).
		Where("id = ?", itemID).
		Updates(updateData).Error
}

// UpdateStatus updates a shopping list status
func (r *shoppingListRepository) UpdateStatus(listID uuid.UUID, userID uuid.UUID, status string) error {
	updateData := map[string]interface{}{
		"status":     status,
		"updated_at": time.Now(),
	}

	if status == models.ShoppingListStatusCompleted {
		updateData["completed_at"] = time.Now()
	}

	return r.db.Model(&models.ShoppingList{}).
		Where("id = ? AND user_id = ?", listID, userID).
		Updates(updateData).Error
}

// Delete deletes a shopping list and its items
func (r *shoppingListRepository) Delete(listID uuid.UUID, userID uuid.UUID) error {
	return r.db.Transaction(func(tx *gorm.DB) error {
		// Delete items first
		if err := tx.Where("shopping_list_id = ?", listID).Delete(&models.ShoppingItem{}).Error; err != nil {
			return err
		}

		// Delete list
		if err := tx.Where("id = ? AND user_id = ?", listID, userID).Delete(&models.ShoppingList{}).Error; err != nil {
			return err
		}

		return nil
	})
}

// GetByMealPlanID retrieves shopping lists associated with a meal plan
func (r *shoppingListRepository) GetByMealPlanID(mealPlanID uuid.UUID) ([]models.ShoppingList, error) {
	var lists []models.ShoppingList
	err := r.db.Where("meal_plan_id = ?", mealPlanID).
		Order("created_at DESC").
		Find(&lists).Error
	if err != nil {
		return nil, err
	}

	return lists, nil
}

// GetActiveByUserID retrieves active shopping lists for a user
func (r *shoppingListRepository) GetActiveByUserID(userID uuid.UUID) ([]models.ShoppingList, error) {
	return r.GetByUserID(userID, models.ShoppingListStatusActive, "created")
}