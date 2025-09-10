package repositories

import (
	"database/sql"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"github.com/imkitchen/backend/internal/models"
)

func setupMockDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock, func()) {
	sqlDB, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("Failed to create mock database: %v", err)
	}

	gormDB, err := gorm.Open(postgres.New(postgres.Config{
		Conn: sqlDB,
	}), &gorm.Config{})
	if err != nil {
		t.Fatalf("Failed to create GORM instance: %v", err)
	}

	cleanup := func() {
		sqlDB.Close()
	}

	return gormDB, mock, cleanup
}

func TestOptimizedRecipeRepository_GetRecipesForMealPlanGeneration(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	filters := &OptimizedRecipeFilters{
		MealTypes:       []string{"breakfast", "lunch"},
		MaxComplexity:   "medium",
		MaxPrepTime:     30,
		DietaryLabels:   []string{"vegetarian"},
		CuisineTypes:    []string{"italian", "american"},
		ExcludeRecipes:  []uuid.UUID{uuid.New()},
		MinRating:       4.0,
		RequiredIngredients: []string{"eggs"},
		ForbiddenIngredients: []string{"nuts"},
	}

	// Mock the expected query
	rows := sqlmock.NewRows([]string{
		"id", "title", "meal_type", "complexity", "prep_time", "cook_time", 
		"total_time", "dietary_labels", "cuisine_type", "average_rating", 
		"ingredients", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Test Recipe", `{"breakfast"}`, "easy", 15, 20, 35,
		`{"vegetarian"}`, "italian", 4.5, `[{"name":"eggs","amount":"2"}]`,
		userID, time.Now(), time.Now(),
	)

	// Set up expectations for the complex query
	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg(), 
				sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg()).
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesForMealPlanGeneration(userID, filters)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "Test Recipe", recipes[0].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetRecipesByMealType(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	rows := sqlmock.NewRows([]string{
		"id", "title", "meal_type", "complexity", "prep_time", "cook_time",
		"total_time", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Breakfast Recipe", `{"breakfast"}`, "easy", 10, 15, 25,
		userID, time.Now(), time.Now(),
	).AddRow(
		uuid.New(), "Lunch Recipe", `{"lunch"}`, "medium", 20, 30, 50,
		userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, `{"breakfast","lunch"}`).
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesByMealType(userID, []string{"breakfast", "lunch"})

	assert.NoError(t, err)
	assert.Len(t, recipes, 2)
	assert.Equal(t, "Breakfast Recipe", recipes[0].Title)
	assert.Equal(t, "Lunch Recipe", recipes[1].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetRecipesByComplexityRange(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	rows := sqlmock.NewRows([]string{
		"id", "title", "complexity", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Easy Recipe", "easy", userID, time.Now(), time.Now(),
	).AddRow(
		uuid.New(), "Medium Recipe", "medium", userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, "easy", "medium").
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesByComplexityRange(userID, "easy", "medium")

	assert.NoError(t, err)
	assert.Len(t, recipes, 2)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetRecipesByTimeConstraints(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	timeConstraints := &TimeConstraints{
		MaxPrepTime:  30,
		MaxCookTime:  60,
		MaxTotalTime: 90,
	}

	rows := sqlmock.NewRows([]string{
		"id", "title", "prep_time", "cook_time", "total_time", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Quick Recipe", 20, 30, 50, userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, timeConstraints.MaxPrepTime, timeConstraints.MaxCookTime, timeConstraints.MaxTotalTime).
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesByTimeConstraints(userID, timeConstraints)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "Quick Recipe", recipes[0].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetRecipesByDietaryLabels(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	rows := sqlmock.NewRows([]string{
		"id", "title", "dietary_labels", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Vegetarian Recipe", `{"vegetarian","gluten-free"}`, userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, `{"vegetarian","gluten-free"}`).
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesByDietaryLabels(userID, []string{"vegetarian", "gluten-free"})

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "Vegetarian Recipe", recipes[0].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetRecipesByCuisine(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	rows := sqlmock.NewRows([]string{
		"id", "title", "cuisine_type", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Pasta Recipe", "italian", userID, time.Now(), time.Now(),
	).AddRow(
		uuid.New(), "Burger Recipe", "american", userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, "italian", "american").
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesByCuisine(userID, []string{"italian", "american"})

	assert.NoError(t, err)
	assert.Len(t, recipes, 2)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetHighlyRatedRecipes(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	rows := sqlmock.NewRows([]string{
		"id", "title", "average_rating", "total_ratings", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Highly Rated Recipe", 4.8, 50, userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, 4.0, 10).
		WillReturnRows(rows)

	recipes, err := repo.GetHighlyRatedRecipes(userID, 4.0, 10)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "Highly Rated Recipe", recipes[0].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_GetRecipesByIngredients(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	ingredientFilters := &IngredientFilters{
		RequiredIngredients:  []string{"chicken", "rice"},
		ForbiddenIngredients: []string{"nuts", "dairy"},
		AvailableIngredients: []string{"chicken", "rice", "vegetables"},
	}

	rows := sqlmock.NewRows([]string{
		"id", "title", "ingredients", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Chicken Rice Bowl", `[{"name":"chicken","amount":"1 lb"},{"name":"rice","amount":"2 cups"}]`, userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, sqlmock.AnyArg(), sqlmock.AnyArg()).
		WillReturnRows(rows)

	recipes, err := repo.GetRecipesByIngredients(userID, ingredientFilters)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "Chicken Rice Bowl", recipes[0].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_Performance(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	// Mock a large result set for performance testing
	rows := sqlmock.NewRows([]string{
		"id", "title", "meal_type", "complexity", "user_id", "created_at", "updated_at",
	})

	// Add 1000 mock rows
	for i := 0; i < 1000; i++ {
		rows.AddRow(
			uuid.New(), "Recipe "+string(rune(i)), `{"breakfast"}`, "easy", userID, time.Now(), time.Now(),
		)
	}

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, sqlmock.AnyArg()).
		WillReturnRows(rows)

	start := time.Now()
	recipes, err := repo.GetRecipesByMealType(userID, []string{"breakfast"})
	duration := time.Since(start)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1000)
	assert.Less(t, duration, 100*time.Millisecond, "Query should be fast with optimized indices")

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestOptimizedRecipeRepository_ComplexFilterCombination(t *testing.T) {
	db, mock, cleanup := setupMockDB(t)
	defer cleanup()

	repo := NewOptimizedRecipeRepository(db)
	userID := uuid.New()

	// Test complex filter combination that exercises multiple indices
	filters := &OptimizedRecipeFilters{
		MealTypes:       []string{"dinner"},
		MaxComplexity:   "medium",
		MaxPrepTime:     45,
		DietaryLabels:   []string{"vegetarian", "gluten-free"},
		CuisineTypes:    []string{"mediterranean"},
		MinRating:       4.2,
		RequiredIngredients: []string{"olive oil"},
	}

	rows := sqlmock.NewRows([]string{
		"id", "title", "meal_type", "complexity", "prep_time", "dietary_labels",
		"cuisine_type", "average_rating", "ingredients", "user_id", "created_at", "updated_at",
	}).AddRow(
		uuid.New(), "Mediterranean Veggie Bowl", `{"dinner"}`, "medium", 30,
		`{"vegetarian","gluten-free"}`, "mediterranean", 4.5,
		`[{"name":"olive oil","amount":"2 tbsp"}]`, userID, time.Now(), time.Now(),
	)

	mock.ExpectQuery(`SELECT \* FROM "recipes"`).
		WithArgs(userID, sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg(),
				sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg()).
		WillReturnRows(rows)

	start := time.Now()
	recipes, err := repo.GetRecipesForMealPlanGeneration(userID, filters)
	duration := time.Since(start)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "Mediterranean Veggie Bowl", recipes[0].Title)
	assert.Less(t, duration, 50*time.Millisecond, "Complex queries should still be fast")

	assert.NoError(t, mock.ExpectationsWereMet())
}