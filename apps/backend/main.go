package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/middleware"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
)

func main() {
	// Load configuration from environment or defaults
	port := getEnv("PORT", "8080")
	
	// Initialize dependencies
	deps := initializeDependencies()
	
	// Setup Gin router
	router := setupRouter(deps)
	
	// Create HTTP server
	server := &http.Server{
		Addr:         fmt.Sprintf(":%s", port),
		Handler:      router,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// Start server in goroutine
	go func() {
		log.Printf("Starting server on port %s", port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server failed to start: %v", err)
		}
	}()

	// Graceful shutdown
	waitForShutdown(server, 30*time.Second)
}

func setupRouter(deps *Dependencies) *gin.Engine {
	// Set release mode in production
	if getEnv("ENVIRONMENT", "development") == "production" {
		gin.SetMode(gin.ReleaseMode)
	}

	router := gin.New()

	// Global middleware
	router.Use(
		middleware.Logger(),
		middleware.Recovery(),
		middleware.CORS(),
		middleware.RequestID(),
		middleware.Security(),
	)

	// Health check endpoints
	router.GET("/health", handlers.HealthCheck(deps))
	router.GET("/readiness", handlers.ReadinessCheck(deps))

	// API version 1
	v1 := router.Group("/api/v1")
	{
		// Authentication endpoints with rate limiting
		auth := v1.Group("/auth")
		auth.Use(middleware.AuthRateLimit(deps.CacheService))
		{
			auth.POST("/register", handlers.Register(deps))
			auth.POST("/login", handlers.Login(deps))
			auth.POST("/refresh", handlers.RefreshToken(deps))
			auth.POST("/logout", middleware.RequireAuth(), handlers.Logout(deps))
			auth.POST("/forgot-password", handlers.ForgotPassword(deps))
			auth.POST("/reset-password", handlers.ResetPassword(deps))
		}

		// Protected routes
		protected := v1.Group("/")
		protected.Use(middleware.RequireAuth())
		{
			// User management
			users := protected.Group("/users")
			{
				users.GET("/me", handlers.GetCurrentUser(deps))
				users.PUT("/me", handlers.UpdateCurrentUser(deps))
				users.DELETE("/me", handlers.DeleteCurrentUser(deps))
			}
			
			// Recipe management
			recipeHandler := handlers.NewRecipeHandler(deps.RecipeService)
			handlers.RegisterRecipeRoutes(protected, recipeHandler)
			
			// Meal plan management
			mealPlanHandler := handlers.NewMealPlanHandler(deps.MealPlanService)
			handlers.RegisterMealPlanRoutes(protected, mealPlanHandler)
			
			// Preference management
			preferenceHandler := handlers.NewPreferenceHandler(deps.PreferenceService)
			handlers.RegisterPreferenceRoutes(protected, preferenceHandler)
			
			// Shopping list management
			shoppingListHandler := handlers.NewShoppingListHandlers(deps.ShoppingListService)
			handlers.RegisterShoppingListRoutes(protected, shoppingListHandler)
			
			// Photo management (if photo service is available)
			if deps.PhotoService != nil {
				photoHandler := handlers.NewPhotoHandler(deps.PhotoService, deps.RecipeService)
				handlers.RegisterPhotoRoutes(protected, photoHandler)
			}
		}
	}

	return router
}

// Dependencies represents all application dependencies
type Dependencies struct {
	CacheService        *services.CacheService
	DatabaseService     *services.DatabaseService
	RecipeService       services.RecipeService
	MealPlanService     services.MealPlanService
	PreferenceService   *services.PreferenceService
	ShoppingListService *services.ShoppingListService
	PhotoService        services.PhotoService
	RedisClient         *redis.Client
}

func initializeDependencies() *Dependencies {
	// Initialize database connection
	databaseService, err := services.NewDatabaseService()
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	
	// Initialize Redis client
	redisAddr := getEnv("REDIS_ADDR", "localhost:6379")
	redisPassword := getEnv("REDIS_PASSWORD", "")
	redisDB := 0 // use default DB
	
	rdb := redis.NewClient(&redis.Options{
		Addr:     redisAddr,
		Password: redisPassword,
		DB:       redisDB,
	})
	
	// Test Redis connection
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	
	if err := rdb.Ping(ctx).Err(); err != nil {
		log.Printf("Redis connection failed: %v", err)
		// Don't fail the application, but log the error
	} else {
		log.Println("Redis connected successfully")
	}
	
	// Initialize cache service
	cacheService := services.NewCacheService(rdb)
	
	// Run database migrations
	migrationService := services.NewMigrationService(databaseService.DB)
	if err := migrationService.RunMigrations(); err != nil {
		log.Fatalf("Failed to run database migrations: %v", err)
	}
	
	// Initialize repositories and services
	recipeRepo := repositories.NewRecipeRepository(databaseService.DB)
	recipeService := services.NewRecipeService(recipeRepo)
	
	mealPlanRepo := repositories.NewMealPlanRepository(databaseService.DB)
	mealPlanService := services.NewMealPlanService(mealPlanRepo, recipeRepo)
	
	preferenceRepo := repositories.NewPreferenceRepository(databaseService.DB)
	preferenceService := services.NewPreferenceService(preferenceRepo)
	
	// Initialize shopping list service
	shoppingListRepo := repositories.NewShoppingListRepository(databaseService.DB)
	shoppingListService := services.NewShoppingListService(shoppingListRepo, mealPlanRepo, recipeRepo, cacheService)
	
	// Initialize photo service
	photoService, err := services.NewPhotoService()
	if err != nil {
		log.Printf("Warning: Failed to initialize photo service: %v", err)
		// Don't fail the application, just log the warning
	}
	
	return &Dependencies{
		CacheService:        cacheService,
		DatabaseService:     databaseService,
		RecipeService:       recipeService,
		MealPlanService:     mealPlanService,
		PreferenceService:   preferenceService,
		ShoppingListService: shoppingListService,
		PhotoService:        photoService,
		RedisClient:         rdb,
	}
}

func waitForShutdown(server *http.Server, timeout time.Duration) {
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Println("Shutting down server...")

	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		log.Fatalf("Server forced to shutdown: %v", err)
	}

	log.Println("Server exited")
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}